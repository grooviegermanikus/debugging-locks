use std::fmt;
use std::fmt::Display;
use std::path::PathBuf;
use std::thread::ThreadId;

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

/// Returns a list of stack frames starting with innermost frame.
///
/// # Examples
///
/// ```
/// let frames = backtrack_frame(|symbol_name| symbol_name.starts_with("rust_basics::debugging_locks::"))
/// ```
pub fn backtrack_frame(fn_skip_frame: fn(&str) -> bool) -> Result<Vec<Frame>, BacktrackError> {

    const FRAMES_LIMIT: usize = 99;

    let mut started = false;
    let mut stop = false;
    let mut symbols = 0;

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
            // module_path is "rust_basics::stacktrace_util"

            if !symbol_name.starts_with("backtrace::backtrace::")
                && !symbol_name.starts_with(module_path!())
                && !fn_skip_frame(symbol_name.as_str()) {
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
        return Ok(frames)
    }

}

