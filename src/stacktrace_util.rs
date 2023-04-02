use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::thread::ThreadId;
use base58::ToBase58;

pub struct Stracktrace {
    pub frames: Vec<Frame>,
    // simple tagging of stacktrace e.g. 'JuCPL' - use for grepping
    pub hash: String,
}

pub struct Frame {
    pub method: String,
    pub filename: String,
    pub line_no: u32,
}

pub struct ThreadInfo {
    pub thread_id: ThreadId,
    pub name: String,
}

#[derive(Debug)]
pub enum BacktrackError {
    NoStartFrame,
    NoDebugSymbols,
}

impl Display for ThreadInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO fix format "main:ThreadId(1)" -> how to deal with numeric thread id?
        write!(f, "{}:{:?}", self.name, self.thread_id)
    }
}

impl Display for BacktrackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BacktrackError::NoStartFrame =>
                write!(f, "Start Frame not found!"),
            BacktrackError::NoDebugSymbols => {
                write!(f, "No debug symbols! Did you build in release mode?")
            }
        }
    }
}


impl std::error::Error for BacktrackError {}

const HASH_ALPHABET: &[u8] = b"0123456789abcdef";

/// Returns a list of stack frames starting with innermost frame.
///
/// # Examples
///
/// ```
/// use rust_debugging_locks::stacktrace_util::backtrack_frame;
/// let frames = backtrack_frame(|symbol_name| symbol_name.starts_with("rust_basics::debugging_locks::"));
/// ```
pub fn backtrack_frame(fn_skip_frame: fn(&str) -> bool) -> Result<Stracktrace, BacktrackError> {

    const FRAMES_LIMIT: usize = 99;

    let mut started = false;
    let mut stop = false;
    let mut symbols = 0;
    let mut hash: String = String::from("no_hash");

    // ordering: inside out
    let mut frames: Vec<Frame> = vec![];

    backtrace::trace(|frame| {
        backtrace::resolve_frame(frame, |symbol| {
            // note: values are None for release build
            // sample output:
            // Symbol { name: backtrace::backtrace::trace_unsynchronized::hc02a5cecd085adce,
            //   addr: 0x100001b2a, filename: ".../.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.67/src/backtrace/mod.rs", lineno: 66 }

            if stop {
                return;
            }

            if symbol.filename().is_none() {
                return;
            }

            symbols += 1;

            if frames.len() > FRAMES_LIMIT {
                stop = true;
                return;
            }

            // /rustc/69f9c33d71c871fc16ac445211281c6e7a340943/library/std/src/rt.rs
            if symbol.filename().unwrap().starts_with(PathBuf::from("/rustc")) {
                stop = true;
                return;
            }

            // symbol.name looks like this "rust_basics::debugging_lock_newtype::backtrack::h1cb6032f9b10548c"
            let symbol_name = symbol.name().unwrap().to_string();
            // module_path is "rust_debugging_locks::stacktrace_util"

            if !symbol_name.starts_with("backtrace::backtrace::")
                && !fn_skip_frame(symbol_name.as_str()) {
                assert_eq!(started, false);
                let addr_instruction_pointer = frame.ip() as u32;
                hash = addr_instruction_pointer.to_be_bytes().to_base58();

                started = true;
                // do not return to catch the current frame
            }

            // note: started may just get flagged for the current frame
            if started {
                frames.push(Frame {
                    method: symbol.name().unwrap().to_string(),
                    filename: symbol.filename().unwrap().file_name().unwrap().to_str().unwrap().to_string(),
                    line_no: symbol.lineno().unwrap() });
                return;
            }

        });

        !stop
    });

    if started == false {
        if symbols == 0 {
            // detected implicitly by checking frames
            return Err(BacktrackError::NoDebugSymbols);
        } else {
            return Err(BacktrackError::NoStartFrame);
        }
    } else {
        return Ok(Stracktrace { frames, hash });
    }

}

fn debug_frames(frames: &Result<Vec<Frame>, BacktrackError>) {
    for frame in frames.as_ref().unwrap() {
        println!("\t>{}:{}:{}", frame.filename, frame.method, frame.line_no);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stacktrace_from_method() {
        let stacktrace = caller_function().unwrap();
        // debug_frames(&frames);
        assert!(stacktrace.frames.get(0).unwrap().method.starts_with("rust_debugging_locks::stacktrace_util::tests::caller_function::h"));
    }

    fn caller_function() -> Result<Stracktrace, BacktrackError> {
        backtrack_frame(|symbol_name| !symbol_name.contains("::caller_function"))
    }
}
