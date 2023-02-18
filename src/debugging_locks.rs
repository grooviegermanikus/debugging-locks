use core::fmt;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError, TryLockResult};
use std::thread;
use std::time::{Duration, Instant};
use log::{info, warn};
use serde::{Serialize, Serializer};
use serde::ser::Error;
use crate::stacktrace_util::{backtrack_frame, BacktrackError, Frame};

// newtype pattern
// why is a Box required when adding stack_created? >> only the last field of a struct may have a dynamically sized
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
        return match backtrack_frame(|symbol_name| symbol_name.starts_with("rust_basics::debugging_locks::")) {
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
        // info!("enter WRITE");

        write_smart(&self)
    }

    pub fn try_read(&self) -> TryLockResult<RwLockReadGuard<'_, T>> {
        // info!("enter TRYWRITE");

        self.inner.try_read()
    }

    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        // info!("enter READ");

        // let result = self.0.read();

        // FIXME remove
        // thread::sleep(Duration::from_millis(500));
        self.inner.read()
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
                        let stack_caller = backtrack_frame(|symbol_name| symbol_name.starts_with("rust_basics::debugging_locks::"));

                        // dispatch to custom handle
                        // note: implementation must deal with debounce, etc.
                        handle_block_event(wait_since, waittime_elapsed,
                                           stacktrace_created.clone(),
                                           &stack_caller.ok());

                        sleep_backoff(cnt);
                        cnt += 1;
                    }
                }
            }
        }
    }
}

// custom handling
// TODO discuss "&Option" vs "Option"
fn handle_block_event(since: Instant, elapsed: Duration,
                      stacktrace_created: &Option<Vec<Frame>>, stacktrace_caller: &Option<Vec<Frame>>) {
    if elapsed.as_millis() < 20 {
        return;
    }
    info!("BLOCKING THREAD for {:?}", elapsed);
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

const SAMPLING_RATE_STAGE1: Duration = Duration::from_micros(100);
const SAMPLING_RATE_STAGE2: Duration = Duration::from_millis(10);
const SAMPLING_RATE_STAGE3: Duration = Duration::from_millis(100);

fn sleep_backoff(cnt: u64) {
    if cnt < 100 {
        thread::sleep(SAMPLING_RATE_STAGE1);
        return;
    } else if cnt < 500 {
        thread::sleep(SAMPLING_RATE_STAGE2);
        return;
    } else {
        thread::sleep(SAMPLING_RATE_STAGE3);
    }
}

