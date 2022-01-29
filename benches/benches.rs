use std::{
    hash::Hasher,
    sync::{Arc, Barrier},
    thread,
    time::Instant,
};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

#[allow(dead_code)]
pub fn atomic_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("Threaded Hashing with Atomic");
    for threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            &threads,
            |b, &threads| {
                b.iter_custom(|iters| {
                    let barrier = Arc::new(Barrier::new(threads + 1));
                    let hasher = Arc::new(cmhash::CoreHasher::new());
                    let threads: Vec<_> = (0..threads)
                        .map(|_tid| {
                            let barrier = Arc::clone(&barrier);
                            let hasher = hasher.clone();
                            thread::spawn(move || {
                                barrier.wait();
                                barrier.wait();
                                for _ in 0..(iters / threads as u64) {
                                    black_box(hasher.hash_word(0xDEADBEEF));
                                }
                            })
                        })
                        .collect();
                    barrier.wait();
                    let start = Instant::now();
                    barrier.wait();
                    for thread in threads {
                        thread.join().unwrap();
                    }
                    start.elapsed()
                })
            },
        );
    }
}

#[allow(dead_code)]
pub fn tl_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("Threaded Hashing with Thread-Local");
    for threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            &threads,
            |b, &threads| {
                b.iter_custom(|iters| {
                    let barrier = Arc::new(Barrier::new(threads + 1));
                    let threads: Vec<_> = (0..threads)
                        .map(|_tid| {
                            let barrier = Arc::clone(&barrier);
                            let hasher = cmhash::TLCoreHasher::new();
                            thread::spawn(move || {
                                barrier.wait();
                                barrier.wait();
                                for _ in 0..(iters / threads as u64) {
                                    black_box(hasher.hash_word(0xDEADBEEF));
                                }
                            })
                        })
                        .collect();
                    barrier.wait();
                    let start = Instant::now();
                    barrier.wait();
                    for thread in threads {
                        thread.join().unwrap();
                    }
                    start.elapsed()
                })
            },
        );
    }
}

#[allow(dead_code)]
pub fn stateless_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("Stateless Threaded Hashing");
    for threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            &threads,
            |b, &threads| {
                b.iter_custom(|iters| {
                    let barrier = Arc::new(Barrier::new(threads + 1));
                    let threads: Vec<_> = (0..threads)
                        .map(|_tid| {
                            let barrier = Arc::clone(&barrier);
                            thread::spawn(move || {
                                barrier.wait();
                                barrier.wait();
                                for _ in 0..(iters / threads as u64) {
                                    black_box(cmhash::hash_word_stateless(0xDEADBEEF));
                                }
                            })
                        })
                        .collect();
                    barrier.wait();
                    let start = Instant::now();
                    barrier.wait();
                    for thread in threads {
                        thread.join().unwrap();
                    }
                    start.elapsed()
                })
            },
        );
    }
}

#[allow(dead_code)]
pub fn tl_build_hasher_threaded(c: &mut Criterion) {
    use std::hash::BuildHasher;
    let mut group = c.benchmark_group("Threaded Hashing with Thread-Local BuildHasher");
    for threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            &threads,
            |b, &threads| {
                b.iter_custom(|iters| {
                    let barrier = Arc::new(Barrier::new(threads + 1));
                    let threads: Vec<_> = (0..threads)
                        .map(|_tid| {
                            let barrier = Arc::clone(&barrier);
                            let build_hasher = cmhash::hasher::CMBuildHasher::new();
                            thread::spawn(move || {
                                barrier.wait();
                                barrier.wait();
                                for _ in 0..(iters / threads as u64) {
                                    let mut hasher = build_hasher.build_hasher();
                                    hasher.write_u64(0xDEADBEEF);
                                    black_box(hasher.finish());
                                }
                            })
                        })
                        .collect();
                    barrier.wait();
                    let start = Instant::now();
                    barrier.wait();
                    for thread in threads {
                        thread.join().unwrap();
                    }
                    start.elapsed()
                })
            },
        );
    }
}

#[allow(dead_code)]
pub fn stateless_build_hasher_threaded(c: &mut Criterion) {
    use std::hash::BuildHasher;
    let mut group = c.benchmark_group("Threaded Hashing with Stateless BuildHasher");
    for threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            &threads,
            |b, &threads| {
                b.iter_custom(|iters| {
                    let barrier = Arc::new(Barrier::new(threads + 1));
                    let threads: Vec<_> = (0..threads)
                        .map(|_tid| {
                            let barrier = Arc::clone(&barrier);
                            let build_hasher = cmhash::hasher::StatelessBuildHasher;
                            thread::spawn(move || {
                                barrier.wait();
                                barrier.wait();
                                for _ in 0..(iters / threads as u64) {
                                    let mut hasher = build_hasher.build_hasher();
                                    hasher.write_u64(0xDEADBEEF);
                                    black_box(hasher.finish());
                                }
                            })
                        })
                        .collect();
                    barrier.wait();
                    let start = Instant::now();
                    barrier.wait();
                    for thread in threads {
                        thread.join().unwrap();
                    }
                    start.elapsed()
                })
            },
        );
    }
}

criterion_group!(
    benches,
    stateless_threaded,
    tl_threaded,
    atomic_threaded,
    tl_build_hasher_threaded,
    stateless_build_hasher_threaded
);
criterion_main!(benches);
