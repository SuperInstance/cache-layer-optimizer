//! Basic optimizer usage example
//!
//! Run with: cargo run --example basic_optimizer

use cache_layer_optimizer::{
    AccessPattern, Optimizer, OptimizationConfig, Predictor, Warmer, WarmingStrategy,
};
use cache_layer::MultiTierCache;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== cache-layer-optimizer Basic Usage ===\n");

    // Create base cache
    let cache = MultiTierCache::new()
        .with_l1(cache_layer::MemoryCache::new(10_000_000)?) // 10MB
        .build();

    println!("✓ Created base cache (10MB memory)\n");

    // Create optimizer
    let config = OptimizationConfig {
        enable_prediction: true,
        enable_warming: true,
        enable_tiering: false,
        enable_coherence: false,
        optimization_interval_secs: 60,
    };

    let mut optimizer = Optimizer::new(config);
    println!("✓ Created optimizer\n");

    // Simulate access patterns
    println!("--- Simulating Access Patterns ---");
    let patterns = generate_test_patterns(1000);
    println!("Generated {} access patterns", patterns.len());
    println!();

    // Run optimization
    println!("--- Running Optimization ---");
    let start = std::time::Instant::now();
    let result = optimizer.optimize(&patterns).await?;
    let duration = start.elapsed();

    println!("✓ Optimization completed in {:?}", duration);
    println!("  - Hit rate improvement: {:.2}%", result.hit_rate_improvement * 100.0);
    println!("  - Keys optimized: {}", result.keys_optimized);
    println!("  - Latency reduction: {:.2}%", result.latency_reduction * 100.0);
    println!("  - Memory efficiency: {:.2}%", result.memory_efficiency * 100.0);
    println!();

    // Display metrics
    println!("--- Optimizer Metrics ---");
    let metrics = optimizer.metrics();
    println!("Prediction requests: {}", metrics.prediction.prediction_requests);
    println!("Predictions made: {}", metrics.prediction.predictions_made);
    println!("Prediction accuracy: {:.2}%", metrics.prediction.prediction_accuracy * 100.0);
    println!();

    println!("=== Example Complete ===");
    Ok(())
}

fn generate_test_patterns(count: usize) -> Vec<AccessPattern> {
    (0..count)
        .map(|i| {
            let mut pattern = AccessPattern::new(format!("key_{}", i));
            pattern.frequency = (i % 100) as f64;
            pattern.regularity = (i % 10) as f64 / 10.0;
            pattern.recency = Duration::from_secs((i % 3600) as u64);
            pattern
        })
        .collect()
}
