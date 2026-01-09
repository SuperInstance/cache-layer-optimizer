//! Predictive caching and warming example
//!
//! Run with: cargo run --example predictive_warming

use cache_layer_optimizer::{
    AccessPattern, FrequencyPredictor, OnDemandWarmer, Predictor, Warmer, WarmingStrategy,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Predictive Caching and Warming ===\n");

    // Create predictor with frequency model
    let predictor = Predictor::new().add_model(Box::new(FrequencyPredictor::new(
        5.0, // min frequency
        100, // window size
    )));

    println!("✓ Created predictor (frequency-based)\n");

    // Generate realistic access patterns
    println!("--- Generating Access Patterns ---");
    let patterns = generate_realistic_patterns(1000);
    println!("Generated {} patterns", patterns.len());

    // Analyze patterns
    let high_freq_count = patterns.iter().filter(|p| p.frequency > 10.0).count();
    let avg_frequency: f64 = patterns.iter().map(|p| p.frequency).sum::<f64>() / patterns.len() as f64;

    println!("High-frequency keys: {}", high_freq_count);
    println!("Average frequency: {:.2}", avg_frequency);
    println!();

    // Get predictions
    println!("--- Running Prediction ---");
    let start = std::time::Instant::now();
    let predictions = predictor.get_predictions(&patterns).await?;
    let duration = start.elapsed();

    println!("✓ Generated {} predictions in {:?}", predictions.len(), duration);
    println!("Top predicted keys:");
    for (i, key) in predictions.iter().take(10).enumerate() {
        let pattern = patterns.iter().find(|p| &p.key == key).unwrap();
        println!("  {}. {} (freq: {:.2}, priority: {:.2})",
            i + 1, key, pattern.frequency, pattern.priority_score());
    }
    println!();

    // Warm cache
    println!("--- Warming Cache ---");
    let warmer = Box::new(OnDemandWarmer::new(100)); // Warm top 100 keys
    let mut warner = Warmer::new(warmer);

    let start = std::time::Instant::now();
    let warmed_keys = warner.warm(&patterns).await?;
    let duration = start.elapsed();

    println!("✓ Warmed {} keys in {:?}", warmed_keys.len(), duration);
    println!("Warmed keys: {}", warmed_keys.iter().take(10).cloned().collect::<Vec<_>>().join(", "));
    println!();

    // Metrics
    println!("--- Metrics ---");
    let metrics = warner.metrics();
    println!("Warming cycles: {}", metrics.warming_cycles);
    println!("Keys warmed: {}", metrics.keys_warmed);
    println!("Average warming time: {:?}", metrics.avg_warming_time);
    println!();

    println!("=== Example Complete ===");
    Ok(())
}

fn generate_realistic_patterns(count: usize) -> Vec<AccessPattern> {
    // Simulate realistic access pattern:
    // - 10% hot keys (high frequency)
    // - 30% warm keys (medium frequency)
    // - 60% cold keys (low frequency)

    (0..count)
        .map(|i| {
            let mut pattern = AccessPattern::new(format!("key_{}", i));

            if i < count / 10 {
                // Hot keys
                pattern.frequency = 50.0 + (i as f64 * 0.5);
                pattern.regularity = 0.9;
            } else if i < count / 10 * 4 {
                // Warm keys
                pattern.frequency = 10.0 + (i as f64 * 0.1);
                pattern.regularity = 0.6;
            } else {
                // Cold keys
                pattern.frequency = (i % 5) as f64;
                pattern.regularity = 0.2;
            }

            pattern.recency = Duration::from_secs((i % 3600) as u64);
            pattern
        })
        .collect()
}
