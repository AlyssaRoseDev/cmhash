
use std::{sync::{Arc, Barrier}, thread, time::Instant};

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

#[allow(dead_code)]
pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Threaded Hashing");
    for threads in [1, 2, 4, 8] {
        group.bench_with_input(BenchmarkId::from_parameter(threads), &threads, |b, &threads| {
            b.iter_custom(|iters| {
                let barrier = Arc::new(Barrier::new(threads + 1));
                let hasher = Arc::new(cmhash::CoreHasher::new());
                let threads: Vec<_> = (0..threads).map(|_tid| {
                    let barrier = Arc::clone(&barrier);
                    let hasher = hasher.clone();
                    thread::spawn(move || {
                        barrier.wait();
                        barrier.wait();
                        for _ in 0..(iters / threads as u64) {
                            black_box(hasher.fast_hash(0xDEADBEEF));
                        }
                    })
                }).collect();
                barrier.wait();
                let start = Instant::now();
                barrier.wait();
                for thread in threads {
                    thread.join().unwrap();
                }
                start.elapsed()
            })
        });
    };
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
