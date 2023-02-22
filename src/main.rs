use env_logger::Env;

mod debugging_locks;
mod stacktrace_util;
mod debugging_locks_run;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    debugging_locks_run::runit();

}

