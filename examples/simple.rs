use env_logger::Env;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    // FIXME
    // debugging_locks_run::runit();
//
}

