use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cmhash::*;

#[allow(dead_code)]
pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("stateful_hash_single_thread", |b| b.iter(|| 
        (black_box(TLCoreHasher::new().fast_hash(0xDEADBEEF)))
    ));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
