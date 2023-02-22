This utility provides a thin wrapper around <code>RwLock</code> (and <code>Mutex</code> in the future) for debugging lock stalls.

The wrapper keeps track of the callers' and creators' stackframes to provide debugging context.


### Usage
* see _debugging_locks_run.rs_ for reference
* !!! You need to include debug symbols to see the stacktraces !!!


### What's missing?
* intercept _.read_ (currently only _.write_) gets tracked
* define interface for callbacks
* remove hex from method name "debugging_locks_run.rs:rust_basics::debugging_locks_run::runit::hbcf42217d721148f"


### Sample output
	[2023-02-18T11:39:32Z INFO  rust_basics::debugging_locks] WRITER WAS BLOCKED on thread main:ThreadId(1) for 1.490548125s
	[2023-02-18T11:39:32Z INFO  rust_basics::debugging_locks] Accessed here:
	[2023-02-18T11:39:32Z INFO  rust_basics::debugging_locks] 	>debugging_locks_run.rs:rust_basics::debugging_locks_run::runit::hbcf42217d721148f:26
	[2023-02-18T11:39:32Z INFO  rust_basics::debugging_locks] 	>main.rs:rust_basics::main::h7b144dc665faa5e5:45
	[2023-02-18T11:39:32Z INFO  rust_basics::debugging_locks] Lock defined here:
	[2023-02-18T11:39:32Z INFO  rust_basics::debugging_locks] 	>debugging_locks_run.rs:rust_basics::debugging_locks_run::runit::hbcf42217d721148f:11
	[2023-02-18T11:39:32Z INFO  rust_basics::debugging_locks] 	>main.rs:rust_basics::main::h7b144dc665faa5e5:45

