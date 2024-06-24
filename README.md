This utility provides a thin wrapper around <code>RwLock</code> (and <code>Mutex</code> in the future) for debugging lock stalls.

The wrapper keeps track of the callers' and creators' stackframes to provide debugging context.


### Usage
* see _debugging_locks_run.rs_ for reference
* you need to include debug symbols to see the stacktraces

#### enable debug symbols for release

using Env:

     RUSTFLAGS=-g cargo build --release

or add to Cargo.toml:

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
    [2023-05-03T09:33:26Z INFO  rust_debugging_locks::debugging_locks] READER BLOCKED on thread main:ThreadId(1) for 7.597502ms (locktag xFxiD)
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>     blocking call:
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::writer_blocks_reader::h90b32e8be4ee69f9:60
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::main::h51d8a2c7c463da66:12
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>     current lock holder:
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::writer_blocks_reader::{{closure}}::h56d46ee0d6ad82da:51
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>     rwlock constructed here:
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::writer_blocks_reader::h90b32e8be4ee69f9:47
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::main::h51d8a2c7c463da66:12
    [2023-05-03T09:33:26Z INFO  rust_debugging_locks::debugging_locks] READER BLOCKED on thread main:ThreadId(1) for 9.658057ms (locktag xFxiD)
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>     blocking call:
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::writer_blocks_reader::h90b32e8be4ee69f9:60
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::main::h51d8a2c7c463da66:12
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>     current lock holder:
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::writer_blocks_reader::{{closure}}::h56d46ee0d6ad82da:51
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>     rwlock constructed here:
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::writer_blocks_reader::h90b32e8be4ee69f9:47
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::main::h51d8a2c7c463da66:12
    [2023-05-03T09:33:26Z INFO  rust_debugging_locks::debugging_locks] READER BLOCKED on thread main:ThreadId(1) for 20.825162ms (locktag xFxiD)
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>     blocking call:
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::writer_blocks_reader::h90b32e8be4ee69f9:60
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::main::h51d8a2c7c463da66:12
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>     current lock holder:
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::writer_blocks_reader::{{closure}}::h56d46ee0d6ad82da:51
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>     rwlock constructed here:
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::writer_blocks_reader::h90b32e8be4ee69f9:47
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::main::h51d8a2c7c463da66:12
    [2023-05-03T09:33:26Z INFO  rust_debugging_locks::debugging_locks] READER BLOCKED on thread main:ThreadId(1) for 24.683994ms (locktag xFxiD)
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>     blocking call:
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::writer_blocks_reader::h90b32e8be4ee69f9:60
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::main::h51d8a2c7c463da66:12
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>     current lock holder:
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::writer_blocks_reader::{{closure}}::h56d46ee0d6ad82da:51
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>     rwlock constructed here:
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::writer_blocks_reader::h90b32e8be4ee69f9:47
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::main::h51d8a2c7c463da66:12
    [2023-05-03T09:33:26Z INFO  rust_debugging_locks::debugging_locks] READER BLOCKED on thread main:ThreadId(1) for 38.673215ms (locktag xFxiD)
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>     blocking call:
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::writer_blocks_reader::h90b32e8be4ee69f9:60
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::main::h51d8a2c7c463da66:12
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>     current lock holder:
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::writer_blocks_reader::{{closure}}::h56d46ee0d6ad82da:51
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>     rwlock constructed here:
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::writer_blocks_reader::h90b32e8be4ee69f9:47
    [2023-05-03T09:33:26Z DEBUG rust_debugging_locks::debugging_locks]  |xFxiD>       simple.rs!simple::main::h51d8a2c7c463da66:12

### locktag
A _locktag_ is assigned a __RwLock__ instance when it is created. The _locktag_ is used to group log lines together. The _locktag_ is a hash of the stacktrace of the caller of the __RwLock::new()__ method.