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
* make thresholds configurable (e.g. via env variables)

### Startup info (how to figure out if it's working)
    [2023-05-02T18:17:53Z INFO  rust_debugging_locks::debugging_locks] NEW WRAPPED RWLOCK (v0.0.0)


### Sample output
    [2023-04-02T21:02:13Z INFO  rust_debugging_locks::debugging_locks] READER BLOCKED on thread main:ThreadId(1) for 10.183237ms (locktag ebDRt)
    [2023-04-02T21:02:13Z DEBUG rust_debugging_locks::debugging_locks]  |ebDRt>     blocking call:
    [2023-04-02T21:02:13Z DEBUG rust_debugging_locks::debugging_locks]  |ebDRt>       simple.rs:simple::writer_blocks_reader::h72064aa0acdf6155:60
    [2023-04-02T21:02:13Z DEBUG rust_debugging_locks::debugging_locks]  |ebDRt>       simple.rs:simple::main::h821b9ad0f7379986:12
    [2023-04-02T21:02:13Z DEBUG rust_debugging_locks::debugging_locks]  |ebDRt>     current lock holder:
    [2023-04-02T21:02:13Z DEBUG rust_debugging_locks::debugging_locks]  |ebDRt>       simple.rs:simple::writer_blocks_reader::{{closure}}::hba167631f187bd26:51
    [2023-04-02T21:02:13Z DEBUG rust_debugging_locks::debugging_locks]  |ebDRt>     rwlock constructed here:
    [2023-04-02T21:02:13Z DEBUG rust_debugging_locks::debugging_locks]  |ebDRt>       simple.rs:simple::writer_blocks_reader::h72064aa0acdf6155:47
    [2023-04-02T21:02:13Z DEBUG rust_debugging_locks::debugging_locks]  |ebDRt>       simple.rs:simple::main::h821b9ad0f7379986:12

### logtags
A _logtag_ is assigned a __RwLock__ instance when it is created. The logtag is used to group log lines together. The logtag is a hash of the stacktrace of the caller of the __RwLock::new()__ method.