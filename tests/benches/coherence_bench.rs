//! Cache coherence protocol benchmarks
//!
//! Benchmarks for cache coherence strategies

use cache_layer_optimizer::{CoherenceProtocol, WriteAround, WriteBack, WriteThrough};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_write_through_invalidate(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_through_invalidate");

    for num_ops in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_ops),
            num_ops,
            |b, &num_ops| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                b.iter(|| {
                    rt.block_on(async {
                        let mut protocol = WriteThrough::new();
                        for i in 0..num_ops {
                            black_box(protocol.invalidate(&format!("key{}", i)).await.unwrap());
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_write_through_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_through_update");

    for num_ops in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_ops),
            num_ops,
            |b, &num_ops| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                b.iter(|| {
                    rt.block_on(async {
                        let mut protocol = WriteThrough::new();
                        for i in 0..num_ops {
                            black_box(
                                protocol
                                    .update(&format!("key{}", i), b"value".to_vec())
                                    .await
                                    .unwrap(),
                            );
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_write_back_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_back_update");

    for num_ops in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_ops),
            num_ops,
            |b, &num_ops| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                b.iter(|| {
                    rt.block_on(async {
                        let mut protocol = WriteBack::new();
                        for i in 0..num_ops {
                            black_box(
                                protocol
                                    .update(&format!("key{}", i), b"value".to_vec())
                                    .await
                                    .unwrap(),
                            );
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_write_around_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_around_update");

    for num_ops in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_ops),
            num_ops,
            |b, &num_ops| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                b.iter(|| {
                    rt.block_on(async {
                        let mut protocol = WriteAround::new();
                        for i in 0..num_ops {
                            black_box(
                                protocol
                                    .update(&format!("key{}", i), b"value".to_vec())
                                    .await
                                    .unwrap(),
                            );
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_protocol_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_comparison");

    for num_ops in [1000, 10000, 100000].iter() {
        // Write-through
        group.bench_with_input(
            BenchmarkId::new("write_through", num_ops),
            num_ops,
            |b, &num_ops| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                b.iter(|| {
                    rt.block_on(async {
                        let mut protocol = WriteThrough::new();
                        for i in 0..num_ops {
                            protocol.invalidate(&format!("key{}", i)).await.unwrap();
                        }
                    })
                });
            },
        );

        // Write-back
        group.bench_with_input(
            BenchmarkId::new("write_back", num_ops),
            num_ops,
            |b, &num_ops| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                b.iter(|| {
                    rt.block_on(async {
                        let mut protocol = WriteBack::new();
                        for i in 0..num_ops {
                            protocol.invalidate(&format!("key{}", i)).await.unwrap();
                        }
                    })
                });
            },
        );

        // Write-around
        group.bench_with_input(
            BenchmarkId::new("write_around", num_ops),
            num_ops,
            |b, &num_ops| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                b.iter(|| {
                    rt.block_on(async {
                        let mut protocol = WriteAround::new();
                        for i in 0..num_ops {
                            protocol.invalidate(&format!("key{}", i)).await.unwrap();
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_mixed_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_operations");

    for num_ops in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_ops),
            num_ops,
            |b, &num_ops| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                b.iter(|| {
                    rt.block_on(async {
                        let mut protocol = WriteThrough::new();
                        for i in 0..num_ops {
                            if i % 2 == 0 {
                                black_box(protocol.invalidate(&format!("key{}", i)).await.unwrap());
                            } else {
                                black_box(
                                    protocol
                                        .update(&format!("key{}", i), b"value".to_vec())
                                        .await
                                        .unwrap(),
                                );
                            }
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_write_through_invalidate,
    bench_write_through_update,
    bench_write_back_update,
    bench_write_around_update,
    bench_protocol_comparison,
    bench_mixed_operations
);

criterion_main!(benches);
