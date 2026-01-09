//! Cache warming benchmarks
//!
//! Benchmarks for cache warming strategies

use cache_layer_optimizer::{
    AccessPattern, OnDemandWarmer, ProactiveWarmer, Warmer, WarmingStrategy,
};
use cache_layer_optimizer::Predictor;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

fn bench_on_demand_warming(c: &mut Criterion) {
    let mut group = c.benchmark_group("on_demand_warming");

    for num_patterns in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_patterns),
            num_patterns,
            |b, &num_patterns| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let warmer = OnDemandWarmer::new(100);

                let patterns: Vec<AccessPattern> = (0..num_patterns)
                    .map(|i| {
                        let mut p = AccessPattern::new(format!("key_{}", i));
                        p.frequency = (i % 100) as f64;
                        p.regularity = (i % 10) as f64 / 10.0;
                        p
                    })
                    .collect();

                b.iter(|| {
                    rt.block_on(async {
                        black_box(warmer.select_keys(black_box(&patterns)).await.unwrap());
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_warmer_warm(c: &mut Criterion) {
    let mut group = c.benchmark_group("warmer_warm");

    for num_patterns in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_patterns),
            num_patterns,
            |b, &num_patterns| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let on_demand = Box::new(OnDemandWarmer::new(1000));
                let mut warmer = Warmer::new(on_demand);

                let patterns: Vec<AccessPattern> = (0..num_patterns)
                    .map(|i| {
                        let mut p = AccessPattern::new(format!("key_{}", i));
                        p.frequency = (i % 100) as f64;
                        p.regularity = (i % 10) as f64 / 10.0;
                        p
                    })
                    .collect();

                b.iter(|| {
                    rt.block_on(async {
                        black_box(warmer.warm(black_box(&patterns)).await.unwrap());
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_proactive_warming(c: &mut Criterion) {
    let mut group = c.benchmark_group("proactive_warming");

    for num_patterns in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_patterns),
            num_patterns,
            |b, &num_patterns| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                let predictor = Predictor::new();
                let warmer = ProactiveWarmer::new(predictor, Duration::from_secs(60));

                let patterns: Vec<AccessPattern> = (0..num_patterns)
                    .map(|i| {
                        let mut p = AccessPattern::new(format!("key_{}", i));
                        p.frequency = (i % 100) as f64;
                        p.regularity = (i % 10) as f64 / 10.0;
                        p
                    })
                    .collect();

                b.iter(|| {
                    rt.block_on(async {
                        black_box(warmer.select_keys(black_box(&patterns)).await.unwrap());
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_warming_cycles(c: &mut Criterion) {
    let mut group = c.benchmark_group("warming_cycles");

    for num_cycles in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_cycles),
            num_cycles,
            |b, &num_cycles| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                let patterns: Vec<AccessPattern> = (0..1000)
                    .map(|i| {
                        let mut p = AccessPattern::new(format!("key_{}", i));
                        p.frequency = (i % 100) as f64;
                        p.regularity = (i % 10) as f64 / 10.0;
                        p
                    })
                    .collect();

                b.iter(|| {
                    rt.block_on(async {
                        let on_demand = Box::new(OnDemandWarmer::new(100));
                        let mut warmer = Warmer::new(on_demand);

                        for _ in 0..num_cycles {
                            black_box(warmer.warm(black_box(&patterns)).await.unwrap());
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
    bench_on_demand_warming,
    bench_warmer_warm,
    bench_proactive_warming,
    bench_warming_cycles
);

criterion_main!(benches);
