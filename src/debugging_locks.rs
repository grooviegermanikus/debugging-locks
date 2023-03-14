use core::fmt;
use std::sync::{LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError, TryLockResult};
use std::thread;
use std::time::{Duration, Instant};
use log::{info, warn};
use serde::{Serialize, Serializer};
use serde::ser::Error;
use crate::stacktrace_util::{backtrack_frame, Frame, ThreadInfo};
use crate::thresholds_config;

const OMIT_FRAME_NAME: &str = "rust_debugging_locks::debugging_locks::";

// newtype pattern
pub struct RwLockWrapped<T: ?Sized> {
    stack_created: Option<Vec<Frame>>,
    // RwLock must be last element in struct
    inner: RwLock<T>,
}

// TODO !!! Send+Sync are only required in transaction_status_service.rs
unsafe impl<T: ?Sized + Send> Send for RwLockWrapped<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for RwLockWrapped<T> {}

// from https://rust-random.github.io/rand/src/serde/ser/impls.rs.html#594-607
impl<T: ?Sized> Serialize for RwLockWrapped<T>
    where
        T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        match self.inner.read() {
            Ok(locked) => locked.serialize(serializer),
            Err(_) => Err(Error::custom("lock poison error while serializing")),
        }
    }
}


impl<T: ?Sized + fmt::Debug> fmt::Debug for RwLockWrapped<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> RwLockWrapped<T> {
    pub fn new(t: T) -> RwLockWrapped<T> {
        info!("SETUP RWLOCK WRAPPER(i)");
        return match backtrack_frame(|symbol_name| symbol_name.starts_with(OMIT_FRAME_NAME)) {
            Ok(frames) => {
                RwLockWrapped { inner: RwLock::new(t), stack_created: Option::from(frames) }
            }
            Err(backtrack_error) => {
                warn!("Unable to determine stacktrace - continue without! (error: {})", backtrack_error);
                RwLockWrapped { inner: RwLock::new(t), stack_created: None }
            }
        }
    }

    pub fn to_rwlock(&self) -> &RwLock<T> {
        &self.inner
    }

    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, T>> {
        write_smart(&self)
    }

    pub fn try_read(&self) -> TryLockResult<RwLockReadGuard<'_, T>> {
        self.inner.try_read()
    }

    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        read_smart(&self)
    }
}

impl<T: Default> Default for RwLockWrapped<T> {
    /// Creates a new `RwLock<T>`, with the `Default` value for T.
    fn default() -> RwLockWrapped<T> {
        RwLockWrapped::new(Default::default())
    }
}

// impl<T: ?Sized> Deref for RwLockWrapped<T> {
//     type Target = RwLock<T>;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }


fn write_smart<T>(rwlock_wrapped: &RwLockWrapped<T>) -> LockResult<RwLockWriteGuard<'_, T>> {
    // info!("ENTER WRITELOCK");
    let rwlock = &rwlock_wrapped.inner;
    let stacktrace_created = &rwlock_wrapped.stack_created;

    let mut cnt: u64 = 0;
    // consider using SystemTime here
    let wait_since = Instant::now();
    loop {
        match rwlock.try_write() {
            Ok(guard) => {
                return Ok(guard);
            }
            Err(err) => {
                match err {
                    TryLockError::Poisoned(poison) => {
                        return Err(poison);
                    }
                    TryLockError::WouldBlock => {
                        let waittime_elapsed = wait_since.elapsed();
                        if thresholds_config::should_inspect_lock(cnt) {
                            let stack_caller = backtrack_frame(|symbol_name| symbol_name.starts_with(OMIT_FRAME_NAME));
                            let thread = thread::current();
                            let thread_info = ThreadInfo { thread_id: thread.id(), name: thread.name().unwrap_or("no_thread").to_string() };

                            // dispatch to custom handle
                            // note: implementation must deal with debounce, etc.
                            handle_blocked_writer_event(wait_since, waittime_elapsed, cnt,
                                                        thread_info,
                                                        stacktrace_created.clone(),
                                                        &stack_caller.ok());
                        }

                        thresholds_config::sleep_backoff(cnt);
                        cnt += 1;
                    }
                }
            }
        }
    }
}


fn read_smart<T>(rwlock_wrapped: &RwLockWrapped<T>) -> LockResult<RwLockReadGuard<'_, T>> {
    // info!("ENTER READLOCK");
    let rwlock = &rwlock_wrapped.inner;
    let stacktrace_created = &rwlock_wrapped.stack_created;

    let mut cnt: u64 = 0;
    // consider using SystemTime here
    let wait_since = Instant::now();
    loop {
        match rwlock.try_read() {
            Ok(guard) => {
                return Ok(guard);
            }
            Err(err) => {
                match err {
                    TryLockError::Poisoned(poison) => {
                        return Err(poison);
                    }
                    TryLockError::WouldBlock => {
                        let waittime_elapsed = wait_since.elapsed();
                        if thresholds_config::should_inspect_lock(cnt) {
                            let stack_caller = backtrack_frame(|symbol_name| symbol_name.starts_with(OMIT_FRAME_NAME));
                            let thread = thread::current();
                            let thread_info = ThreadInfo { thread_id: thread.id(), name: thread.name().unwrap_or("no_thread").to_string() };

                            // dispatch to custom handle
                            // note: implementation must deal with debounce, etc.
                            handle_blocked_reader_event(wait_since, waittime_elapsed, cnt,
                                                        thread_info,
                                                        stacktrace_created.clone(),
                                                        &stack_caller.ok());
                        }

                        thresholds_config::sleep_backoff(cnt);
                        cnt += 1;
                    }
                }
            }
        }
    }
}

// custom handling
// TODO discuss "&Option" vs "Option"
fn handle_blocked_writer_event(_since: Instant, elapsed: Duration,
                               cnt: u64,
                               thread: ThreadInfo,
                               stacktrace_created: &Option<Vec<Frame>>, stacktrace_caller: &Option<Vec<Frame>>) {
    info!("WRITER WAS BLOCKED on thread {} for {:?}", thread, elapsed);
    match stacktrace_caller {
        None => {}
        Some(frames) => {
            info!("Accessed here:");
            for frame in frames {
                info!("\t>{}:{}:{}", frame.filename, frame.method, frame.line_no);
            }
        }
    }
    match stacktrace_created {
        None => {}
        Some(frames) => {
            info!("Lock defined here:");
            for frame in frames {
                info!("\t>{}:{}:{}", frame.filename, frame.method, frame.line_no);
            }
        }
    }
}

fn handle_blocked_reader_event(_since: Instant, elapsed: Duration,
                               cnt: u64,
                               thread: ThreadInfo,
                               stacktrace_created: &Option<Vec<Frame>>, stacktrace_caller: &Option<Vec<Frame>>) {
    info!("READER WAS BLOCKED on thread {} for {:?}", thread, elapsed);
    match stacktrace_caller {
        None => {}
        Some(frames) => {
            info!("Accessed here:");
            for frame in frames {
                info!("\t>{}:{}:{}", frame.filename, frame.method, frame.line_no);
            }
        }
    }
    match stacktrace_created {
        None => {}
        Some(frames) => {
            info!("Lock defined here:");
            for frame in frames {
                info!("\t>{}:{}:{}", frame.filename, frame.method, frame.line_no);
            }
        }
    }
}

