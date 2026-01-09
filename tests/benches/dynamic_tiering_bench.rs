//! Dynamic tier sizing benchmarks
//!
//! Benchmarks for adaptive cache tier sizing

use cache_layer_optimizer::{
    AdaptiveTiering, DynamicTiering, FixedTiering, TierConfig, TierManager, TierSizing,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

fn bench_tier_config_utilization(c: &mut Criterion) {
    let mut group = c.benchmark_group("tier_config_utilization");

    for num_configs in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_configs),
            num_configs,
            |b, &num_configs| {
                let configs: Vec<TierConfig> = (0..num_configs)
                    .map(|i| {
                        let mut config = TierConfig::new(format!("tier_{}", i), 1024 * 1024);
                        config.current_size = (i * 1024) as u64;
                        config
                    })
                    .collect();

                b.iter(|| {
                    for config in &configs {
                        black_box(config.utilization());
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_fixed_tiering(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixed_tiering");

    for num_tiers in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tiers),
            num_tiers,
            |b, &num_tiers| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                let configs: Vec<TierConfig> = (0..num_tiers)
                    .map(|i| TierConfig::new(format!("tier_{}", i), 1024 * 1024))
                    .collect();

                let tiering = FixedTiering::new(configs);

                b.iter(|| {
                    rt.block_on(async {
                        black_box(tiering.calculate_sizes(black_box(&tiering.metrics)).await.unwrap());
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_adaptive_tiering_calculate(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_tiering_calculate");

    for num_tiers in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tiers),
            num_tiers,
            |b, &num_tiers| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                let configs: Vec<TierConfig> = (0..num_tiers)
                    .map(|i| TierConfig::new(format!("tier_{}", i), 1024 * 1024))
                    .collect();

                let tiering = AdaptiveTiering::new(configs, Duration::from_secs(60));

                b.iter(|| {
                    rt.block_on(async {
                        black_box(tiering.calculate_sizes(black_box(&tiering.metrics)).await.unwrap());
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_adaptive_tiering_adjust(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_tiering_adjust");

    for num_tiers in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tiers),
            num_tiers,
            |b, &num_tiers| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                let configs: Vec<TierConfig> = (0..num_tiers)
                    .map(|i| TierConfig::new(format!("tier_{}", i), 1024 * 1024))
                    .collect();

                let mut tiering = AdaptiveTiering::new(configs, Duration::from_secs(60));

                b.iter(|| {
                    rt.block_on(async {
                        let configs = tiering.calculate_sizes(black_box(&tiering.metrics)).await.unwrap();
                        black_box(tiering.adjust_sizes(black_box(&configs)).await.unwrap());
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_tier_manager_optimize(c: &mut Criterion) {
    let mut group = c.benchmark_group("tier_manager_optimize");

    for num_tiers in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tiers),
            num_tiers,
            |b, &num_tiers| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                let configs: Vec<TierConfig> = (0..num_tiers)
                    .map(|i| TierConfig::new(format!("tier_{}", i), 1024 * 1024))
                    .collect();

                let tiering = FixedTiering::new(configs);
                let mut manager = TierManager::new(Box::new(tiering));

                b.iter(|| {
                    rt.block_on(async {
                        black_box(manager.optimize_tiers().await.unwrap());
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_adaptive_grow_shrink(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_grow_shrink");

    for num_ops in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_ops),
            num_ops,
            |b, &num_ops| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                let configs = vec![TierConfig::new("l1".to_string(), 1024 * 1024 * 1024)];
                let mut tiering = AdaptiveTiering::new(configs, Duration::from_secs(60));

                b.iter(|| {
                    for i in 0..num_ops {
                        if i % 2 == 0 {
                            black_box(tiering.grow_tier(0).unwrap());
                        } else {
                            black_box(tiering.shrink_tier(0).unwrap());
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_tier_config_utilization,
    bench_fixed_tiering,
    bench_adaptive_tiering_calculate,
    bench_adaptive_tiering_adjust,
    bench_tier_manager_optimize,
    bench_adaptive_grow_shrink
);

criterion_main!(benches);
