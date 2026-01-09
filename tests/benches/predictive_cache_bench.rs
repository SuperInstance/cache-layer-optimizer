//! Predictive caching benchmarks
//!
//! Benchmarks for ML-based cache prediction performance

use cache_layer_optimizer::{
    AccessPattern, FrequencyPredictor, Predictor, RecencyPredictor,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_access_pattern_priority(c: &mut Criterion) {
    let mut group = c.benchmark_group("access_pattern_priority");

    for num_patterns in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_patterns),
            num_patterns,
            |b, &num_patterns| {
                let patterns: Vec<AccessPattern> = (0..num_patterns)
                    .map(|i| {
                        let mut p = AccessPattern::new(format!("key_{}", i));
                        p.frequency = (i % 100) as f64;
                        p.regularity = (i % 10) as f64 / 10.0;
                        p
                    })
                    .collect();

                b.iter(|| {
                    for pattern in &patterns {
                        black_box(pattern.priority_score());
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_frequency_predictor(c: &mut Criterion) {
    let mut group = c.benchmark_group("frequency_predictor");

    for num_patterns in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_patterns),
            num_patterns,
            |b, &num_patterns| {
                let predictor = FrequencyPredictor::new(5.0, 100);
                let patterns: Vec<AccessPattern> = (0..num_patterns)
                    .map(|i| {
                        let mut p = AccessPattern::new(format!("key_{}", i));
                        p.frequency = (i % 100) as f64;
                        p
                    })
                    .collect();

                b.iter(|| {
                    black_box(predictor.get_high_frequency_keys(black_box(&patterns)));
                });
            },
        );
    }

    group.finish();
}

fn bench_recency_predictor(c: &mut Criterion) {
    let mut group = c.benchmark_group("recency_predictor");

    for num_patterns in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_patterns),
            num_patterns,
            |b, &num_patterns| {
                let predictor = RecencyPredictor::new(std::time::Duration::from_secs(300));
                let patterns: Vec<AccessPattern> = (0..num_patterns)
                    .map(|i| {
                        let mut p = AccessPattern::new(format!("key_{}", i));
                        p.last_access = chrono::Utc::now() - chrono::Duration::seconds(i as i64);
                        p
                    })
                    .collect();

                b.iter(|| {
                    black_box(predictor.get_recent_keys(black_box(&patterns)));
                });
            },
        );
    }

    group.finish();
}

fn bench_predictor(c: &mut Criterion) {
    let mut group = c.benchmark_group("predictor");

    for num_patterns in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_patterns),
            num_patterns,
            |b, &num_patterns| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                let predictor = Predictor::new()
                    .add_model(Box::new(FrequencyPredictor::new(5.0, 100)))
                    .add_model(Box::new(RecencyPredictor::new(std::time::Duration::from_secs(
                        300,
                    ))));

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
                        black_box(predictor.get_predictions(black_box(&patterns)).await.unwrap());
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_predictor_train(c: &mut Criterion) {
    let mut group = c.benchmark_group("predictor_train");

    for num_samples in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_samples),
            num_samples,
            |b, &num_samples| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let mut predictor = FrequencyPredictor::new(5.0, 100);

                let patterns: Vec<AccessPattern> = (0..num_samples)
                    .map(|i| {
                        let mut p = AccessPattern::new(format!("key_{}", i));
                        p.frequency = (i % 100) as f64;
                        p
                    })
                    .collect();

                b.iter(|| {
                    rt.block_on(async {
                        black_box(predictor.train(black_box(&patterns)).await.unwrap());
                    })
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_access_pattern_priority,
    bench_frequency_predictor,
    bench_recency_predictor,
    bench_predictor,
    bench_predictor_train
);

criterion_main!(benches);
