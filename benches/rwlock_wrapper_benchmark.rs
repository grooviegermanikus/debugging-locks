use criterion::{criterion_group, criterion_main, Criterion};
use rust_debugging_locks::debugging_locks::RwLockWrapped;


fn rwlock_creation_and_use(c: &mut Criterion) {
    c.bench_function("create rwlock", |b| b.iter(|| {
            let _lock = RwLockWrapped::new(());
    }));
}

criterion_group!(benches, rwlock_creation_and_use);
criterion_main!(benches);
