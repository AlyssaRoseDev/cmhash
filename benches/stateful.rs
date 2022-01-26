use cmhash::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[allow(dead_code)]
pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("stateful_hash_single_thread", |b| {
        let h = TLCoreHasher::new();
        b.iter(|| (black_box(h.hash_word(0xDEADBEEF))))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
