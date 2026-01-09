//! Overall throughput benchmarks
//!
//! End-to-end performance benchmarks for the optimizer

use cache_layer_optimizer::{
    AccessPattern, Optimizer, OptimizationConfig, Predictor, Warmer, WarmingStrategy,
};
use cache_layer_optimizer::OnDemandWarmer;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_optimizer_no_warming(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimizer_no_warming");

    for num_patterns in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_patterns),
            num_patterns,
            |b, &num_patterns| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                let config = OptimizationConfig {
                    enable_prediction: true,
                    enable_warming: false,
                    enable_tiering: false,
                    enable_coherence: false,
                    ..Default::default()
                };

                let mut optimizer = Optimizer::new(config);

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
                        black_box(optimizer.optimize(black_box(&patterns)).await.unwrap());
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_optimizer_with_warming(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimizer_with_warming");

    for num_patterns in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_patterns),
            num_patterns,
            |b, &num_patterns| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                let config = OptimizationConfig {
                    enable_prediction: true,
                    enable_warming: true,
                    enable_tiering: false,
                    enable_coherence: false,
                    ..Default::default()
                };

                let mut optimizer = Optimizer::new(config);
                // Note: In actual implementation, we'd add a warmer here

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
                        black_box(optimizer.optimize(black_box(&patterns)).await.unwrap());
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_optimizer_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimizer_full");

    for num_patterns in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_patterns),
            num_patterns,
            |b, &num_patterns| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                let config = OptimizationConfig::default();
                let mut optimizer = Optimizer::new(config);

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
                        black_box(optimizer.optimize(black_box(&patterns)).await.unwrap());
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_optimizer_consecutive_runs(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimizer_consecutive_runs");

    for num_runs in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_runs),
            num_runs,
            |b, &num_runs| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                let config = OptimizationConfig::default();
                let mut optimizer = Optimizer::new(config);

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
                        for _ in 0..num_runs {
                            black_box(optimizer.optimize(black_box(&patterns)).await.unwrap());
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_predictor_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("predictor_scalability");

    for num_patterns in [1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_patterns),
            num_patterns,
            |b, &num_patterns| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                let predictor = Predictor::new();

                let patterns: Vec<AccessPattern> = (0..*num_patterns)
                    .map(|i| {
                        let mut p = AccessPattern::new(format!("key_{}", i));
                        p.frequency = (i % 100) as f64;
                        p.regularity = (i % 10) as f64 / 10.0;
                        p
                    })
                    .collect();

                b.iter(|| {
                    rt.block_on(async {
                        black_box(predictor.get_predictions(black_box(&patterns)).await.unwrap());
                    })
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_optimizer_no_warming,
    bench_optimizer_with_warming,
    bench_optimizer_full,
    bench_optimizer_consecutive_runs,
    bench_predictor_scalability
);

criterion_main!(benches);
