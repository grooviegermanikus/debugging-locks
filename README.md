This utility provides a thin wrapper around <code>RwLock</code> (and <code>Mutex</code> in the future) for debugging lock stalls.

The wrapper keeps track of the callers' and creators' stackframes to provide debugging context.


### Usage
* see _debugging_locks_run.rs_ for reference
* you need to include debug symbols to see the stacktraces

#### enable debug symbols for release
    [profile.release]
    debug = true
(this will increase the binary size by 20x; performance/optimization of machine code is NOT affected)

#### enable logger
there are two levels of information available:
1. basic information about the lock (e.g. who got blocked and how log)
2. stacktraces of the callers, the lock creator and the lock holder

```rust
 env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
```

```bash
RUST_LOG=rust_debugging_locks::debugging_locks=info the_binary
```

### What's missing?
* detect if debug symbols are available and warn/fail if not
* define interface for callbacks
* remove hex from method name "debugging_locks_run.rs:rust_basics::debugging_locks_run::runit::hbcf42217d721148f"
* add string (e.g. hash) to each log line to allow grouping (using grep)
* enhance benchmark for rwlock wrapper
* symbolize stacktraces lazy; keep only the instruction pointer/symbol address
* check if __last_returned_lock_from__ is expensive; if yes make it an optional feature

### Startup info (how to figure out if it's working)
    [2023-05-02T18:17:53Z INFO  rust_debugging_locks::debugging_locks] SETUP RWLOCK WRAPPER (v0.0.0)


### Sample output
	[2023-05-18T11:39:32Z INFO  rust_basics::debugging_locks] WRITER WAS BLOCKED on thread main:ThreadId(1) for 1.490548125s
	[2023-05-18T11:39:32Z DEBUG  rust_basics::debugging_locks] Accessed here:
	[2023-05-18T11:39:32Z DEBUG  rust_basics::debugging_locks] 	>debugging_locks_run.rs:rust_basics::debugging_locks_run::runit::hbcf42217d721148f:26
	[2023-05-18T11:39:32Z DEBUG  rust_basics::debugging_locks] 	>main.rs:rust_basics::main::h7b144dc665faa5e5:45
	[2023-05-18T11:39:32Z DEBUG  rust_basics::debugging_locks] Lock defined here:
	[2023-05-18T11:39:32Z DEBUG  rust_basics::debugging_locks] 	>debugging_locks_run.rs:rust_basics::debugging_locks_run::runit::hbcf42217d721148f:11
	[2023-05-18T11:39:32Z DEBUG  rust_basics::debugging_locks] 	>main.rs:rust_basics::main::h7b144dc665faa5e5:45

