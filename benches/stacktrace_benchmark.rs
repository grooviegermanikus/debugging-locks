use criterion::{criterion_group, criterion_main, Criterion};
use rust_debugging_locks::stacktrace_util::backtrack_frame;

fn dummy_start_frame() {
    let stacktrace = backtrack_frame(|symbol_name| symbol_name.contains("no_skip"));
    assert_eq!(3, stacktrace.unwrap().len());
}

fn backtrace_benchmark(c: &mut Criterion) {
    c.bench_function("backtrace full stack", |b| b.iter(||
        {
            backtrace::trace(|_frame| {
                true // continue
            });
        }
    ));
    c.bench_function("backtrace not walking stack", |b| b.iter(||
        {
            backtrace::trace(|_frame| {
                false // do not continue
            });
        }
    ));
    c.bench_function("stacktrace", |b| b.iter(||
        dummy_start_frame()
    ));
}

criterion_group!(benches, backtrace_benchmark);
criterion_main!(benches);
