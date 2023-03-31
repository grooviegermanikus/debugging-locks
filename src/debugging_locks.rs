use core::fmt;
use std::sync::{Arc, LockResult, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError, TryLockResult};
use std::{ptr, thread};
use std::cell::{Cell, RefCell};
use std::sync::atomic::AtomicPtr;
use std::time::{Duration, Instant};
use log::{info, warn};
use serde::{Serialize, Serializer};
use serde::ser::Error;
use crate::stacktrace_util::{backtrack_frame, Frame, ThreadInfo};
use crate::thresholds_config;

// covers:
// rust_debugging_locks::debugging_locks::
// rust_debugging_locks::stacktrace_util::
const OMIT_FRAME_NAME: &str = "rust_debugging_locks::";

// newtype pattern
pub struct RwLockWrapped<T: ?Sized> {
    stack_created: Option<Vec<Frame>>,
    // note: this does NOT reflect a currently acquired lock
    last_returned_lock_from: Arc<Mutex<Option<Vec<Frame>>>>,
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
        let version = env!("CARGO_PKG_VERSION");
        info!("SETUP RWLOCK WRAPPER (v{})", version);
        return match backtrack_frame(|symbol_name| symbol_name.starts_with(OMIT_FRAME_NAME)) {
            Ok(frames) => {
                RwLockWrapped {
                    inner: RwLock::new(t),
                    stack_created: Option::from(frames),
                    last_returned_lock_from: Arc::new(Mutex::new(None)) }
            }
            Err(backtrack_error) => {
                warn!("Unable to determine stacktrace - continue without! (error: {})", backtrack_error);
                RwLockWrapped {
                    inner: RwLock::new(t),
                    stack_created: None,
                    last_returned_lock_from: Arc::new(Mutex::new(None)) }
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
    let rwlock = &rwlock_wrapped.inner;
    let mut last_returned = &rwlock_wrapped.last_returned_lock_from;

    let mut cnt: u64 = 0;
    // consider using SystemTime here
    let wait_since = Instant::now();
    loop {
        match rwlock.try_write() {
            Ok(guard) => {
                let stack_caller = backtrack_frame(|symbol_name| symbol_name.starts_with(OMIT_FRAME_NAME));
                *last_returned.lock().unwrap() = Some(stack_caller.expect("stacktrace should be available"));
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
                            let stacktrace_created = &rwlock_wrapped.stack_created;
                            let last_lock_from = &rwlock_wrapped.last_returned_lock_from;

                            // dispatch to custom handle
                            // note: implementation must deal with debounce, etc.
                            handle_blocked_writer_event(wait_since, waittime_elapsed, cnt,
                                                        thread_info,
                                                        stacktrace_created.clone(),
                                                        last_lock_from.clone(),
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
    let rwlock = &rwlock_wrapped.inner;
    let mut last_returned = &rwlock_wrapped.last_returned_lock_from;

    let mut cnt: u64 = 0;
    // consider using SystemTime here
    let wait_since = Instant::now();
    loop {
        match rwlock.try_read() {
            Ok(guard) => {
                let stack_caller = backtrack_frame(|symbol_name| symbol_name.starts_with(OMIT_FRAME_NAME));
                *last_returned.lock().unwrap() = Some(stack_caller.expect("stacktrace should be available"));
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
                            let stacktrace_created = &rwlock_wrapped.stack_created;
                            let last_lock_from = &rwlock_wrapped.last_returned_lock_from;

                            // dispatch to custom handle
                            // note: implementation must deal with debounce, etc.
                            handle_blocked_reader_event(wait_since, waittime_elapsed, cnt,
                                                        thread_info,
                                                        stacktrace_created.clone(),
                                                        last_lock_from.clone(),
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
                               stacktrace_created: &Option<Vec<Frame>>,
                               last_returned_lock_from: Arc<Mutex<Option<Vec<Frame>>>>,
                               stacktrace_caller: &Option<Vec<Frame>>) {
    info!("WRITER BLOCKED on thread {} for {:?} - details:", thread, elapsed);

    match stacktrace_caller {
        None => {}
        Some(frames) => {
            info!("  >blocking call:");
            for frame in frames {
                info!("\t>{}:{}:{}", frame.filename, frame.method, frame.line_no);
            }
        }
    }

    match last_returned_lock_from.lock().unwrap().as_ref() {
        None => {}
        Some(frames) => {
            info!("  >concurrent lock acquired here:");
            for frame in frames {
                info!("\t>{}:{}:{}", frame.filename, frame.method, frame.line_no);
            }
        }
    }

    match stacktrace_created {
        None => {}
        Some(frames) => {
            info!("  >RwLock constructed here:");
            for frame in frames {
                info!("\t>{}:{}:{}", frame.filename, frame.method, frame.line_no);
            }
        }
    }
}

fn handle_blocked_reader_event(_since: Instant, elapsed: Duration,
                               cnt: u64,
                               thread: ThreadInfo,
                               stacktrace_created: &Option<Vec<Frame>>,
                               last_returned_lock_from: Arc<Mutex<Option<Vec<Frame>>>>,
                               stacktrace_caller: &Option<Vec<Frame>>) {
    info!("READER BLOCKED on thread {} for {:?} - details:", thread, elapsed);

    match stacktrace_caller {
        None => {}
        Some(frames) => {
            info!("  >blocking here:");
            for frame in frames {
                info!("\t>{}:{}:{}", frame.filename, frame.method, frame.line_no);
            }
        }
    }

    match last_returned_lock_from.lock().unwrap().as_ref() {
        None => {}
        Some(frames) => {
            info!("  >concurrent lock acquired here:");
            for frame in frames {
                info!("\t>{}:{}:{}", frame.filename, frame.method, frame.line_no);
            }
        }
    }

    match stacktrace_created {
        None => {}
        Some(frames) => {
            info!("  >RwLock constructed here:");
            for frame in frames {
                info!("\t>{}:{}:{}", frame.filename, frame.method, frame.line_no);
            }
        }
    }
}

