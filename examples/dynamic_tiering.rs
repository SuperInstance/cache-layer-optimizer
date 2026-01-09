//! Dynamic tier sizing example
//!
//! Run with: cargo run --example dynamic_tiering

use cache_layer_optimizer::{
    AdaptiveTiering, TierConfig, TierManager, TierSizing,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Dynamic Tier Sizing ===\n");

    // Create initial tier configuration
    let configs = vec![
        TierConfig::new("l1".to_string(), 100 * 1024 * 1024),   // 100MB L1
        TierConfig::new("l2".to_string(), 1024 * 1024 * 1024),  // 1GB L2
        TierConfig::new("l3".to_string(), 10 * 1024 * 1024 * 1024), // 10GB L3
    ];

    println!("--- Initial Tier Configuration ---");
    print_tier_config(&configs);

    // Create adaptive tiering
    let mut tiering = AdaptiveTiering::new(configs, Duration::from_secs(300));

    println!("\n--- Simulating Load ---");

    // Simulate high load on L1
    println!("Simulating high load on L1...");
    tiering.grow_tier(0)?;
    tiering.grow_tier(0)?;
    println!("L1 grown twice");

    print_tier_config(&tiering.configs);

    // Simulate low load on L2
    println!("\nSimulating low load on L2...");
    tiering.shrink_tier(1)?;
    println!("L2 shrunk once");

    print_tier_config(&tiering.configs);

    // Create tier manager and optimize
    println!("\n--- Optimizing Tiers ---");
    let manager = TierManager::new(Box::new(tiering));
    let mut manager_ref = manager; // Note: In real code, you'd keep this mutable

    let start = std::time::Instant::now();
    let optimized = manager_ref.optimize_tiers().await?;
    let duration = start.elapsed();

    println!("✓ Optimization completed in {:?}", duration);

    print_tier_config(&optimized);

    // Metrics
    println!("\n--- Tiering Metrics ---");
    let metrics = manager_ref.metrics();
    println!("Sizing adjustments: {}", metrics.sizing_adjustments);
    println!("Total bytes moved: {}", metrics.total_bytes_moved);
    println!("Avg adjustment time: {:?}", metrics.avg_adjustment_time);
    println!("Hit rate improvement: {:.2}%", metrics.hit_rate_improvement * 100.0);
    println!("Memory efficiency: {:.2}%", metrics.memory_efficiency * 100.0);

    println!("\n=== Example Complete ===");
    Ok(())
}

fn print_tier_config(configs: &[TierConfig]) {
    for config in configs {
        println!(
            "  {} - Current: {} MB, Target: {} MB, Max: {} MB, Utilization: {:.1}%",
            config.tier_name,
            config.current_size / (1024 * 1024),
            config.target_size / (1024 * 1024),
            config.max_size / (1024 * 1024),
            config.utilization() * 100.0
        );
    }
}
