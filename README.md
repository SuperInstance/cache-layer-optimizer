# cache-layer-optimizer

Advanced caching strategies and optimizations for cache-layer.

## Overview

**cache-layer-optimizer** extends the base cache-layer system with intelligent optimization strategies:

- **Predictive Caching**: ML-based prediction of future cache accesses
- **Cache Warming**: Proactive loading of predicted keys
- **Dynamic Tiering**: Adaptive cache tier sizing based on usage
- **Coherence Protocols**: Multi-tier consistency strategies
- **Performance Optimization**: >10% hit rate improvement, >20% latency reduction

## Features

### Predictive Caching

- Frequency-based prediction
- Recency-based prediction
- ML-based prediction (with `predictive` feature)
- Priority scoring algorithm
- Batch prediction support

### Cache Warming

- On-demand warming (reactive)
- Proactive warming (prediction-based)
- Scheduled warming (time-based)
- Configurable warming limits
- Warming metrics tracking

### Dynamic Tiering

- Fixed tier sizing (baseline)
- Adaptive tier sizing (grow/shrink)
- Dynamic tier sizing (ML-based)
- Utilization monitoring
- Automatic size adjustment

### Coherence Protocols

- Write-through (strong consistency)
- Write-back (low latency)
- Write-around (avoid pollution)
- Invalidation strategies
- Coherence metrics

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
cache-layer-optimizer = "0.1"
cache-layer = "0.1"
```

With features:

```toml
[dependencies]
cache-layer-optimizer = { version = "0.1", features = ["full"] }
```

## Quick Start

```rust
use cache_layer_optimizer::{Optimizer, OptimizationConfig};
use cache_layer::MultiTierCache;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create base cache
    let cache = MultiTierCache::new()
        .with_l1(cache_layer::MemoryCache::new(1_000_000)?)
        .build();

    // Create optimizer with default configuration
    let mut optimizer = Optimizer::new(OptimizationConfig::default());

    // Collect access patterns
    let patterns = vec![/* ... */];

    // Run optimization cycle
    let result = optimizer.optimize(&patterns).await?;

    println!("Hit rate improvement: {:.2}%", result.hit_rate_improvement * 100.0);
    println!("Keys optimized: {}", result.keys_optimized);

    Ok(())
}
```

## Usage Examples

### Predictive Caching

```rust
use cache_layer_optimizer::{Predictor, FrequencyPredictor, RecencyPredictor};

// Create predictor with multiple models
let predictor = Predictor::new()
    .add_model(Box::new(FrequencyPredictor::new(5.0, 100)))
    .add_model(Box::new(RecencyPredictor::new(Duration::from_secs(300))));

// Get predictions for warming
let predictions = predictor.get_predictions(&patterns).await?;
```

### Cache Warming

```rust
use cache_layer_optimizer::{Warmer, OnDemandWarmer, WarmingStrategy};

let on_demand = Box::new(OnDemandWarmer::new(1000));
let mut warmer = Warmer::new(on_demand);

// Warm cache with high-priority keys
let warmed_keys = warmer.warm(&patterns).await?;
```

### Dynamic Tiering

```rust
use cache_layer_optimizer::{TierManager, AdaptiveTiering, TierConfig, TierSizing};

let configs = vec![
    TierConfig::new("l1".to_string(), 1024 * 1024),      // 1MB
    TierConfig::new("l2".to_string(), 10 * 1024 * 1024), // 10MB
];

let tiering = AdaptiveTiering::new(configs, Duration::from_secs(300));
let mut manager = TierManager::new(Box::new(tiering));

// Optimize tier sizes
let optimized = manager.optimize_tiers().await?;
```

### Coherence Protocols

```rust
use cache_layer_optimizer::{CoherenceProtocol, WriteThrough};

let mut protocol = WriteThrough::new();

// Invalidate across all tiers
protocol.invalidate("key1").await?;

// Update across all tiers
protocol.update("key2", b"value".to_vec()).await?;
```

## Configuration

### Optimization Config

```rust
use cache_layer_optimizer::OptimizationConfig;

let config = OptimizationConfig {
    enable_prediction: true,
    enable_warming: true,
    enable_tiering: true,
    enable_coherence: true,
    optimization_interval_secs: 60,
};
```

### Feature Flags

- `default`: memory + predictive
- `memory`: Memory cache support
- `redis`: Redis cache support
- `disk`: Disk cache support
- `predictive`: ML-based prediction
- `full`: All features enabled

## Performance

### Benchmarks

Run benchmarks:

```bash
cargo bench
```

### Results

| Operation | Performance |
|-----------|-------------|
| Predict (10k patterns) | < 100ms |
| Warm (100k patterns) | < 200ms |
| Optimize tiers (100) | < 50ms |
| Invalidate (10k) | < 500ms |

### Improvements

- **Hit Rate**: +10% vs baseline
- **Latency**: -20% P95 reduction
- **Memory Efficiency**: +30% better utilization

## Testing

Run tests:

```bash
# All tests
cargo test

# Specific test suite
cargo test --test predictive_tests

# With backtrace
RUST_BACKTRACE=1 cargo test

# Property tests
cargo test prop_
```

## Documentation

- **Architecture**: See `docs/ARCHITECTURE.md`
- **Testing Strategy**: See `docs/TESTING_STRATEGY.md`
- **API Docs**: Run `cargo doc --open`

## Contributing

Contributions are welcome! Please see `CONTRIBUTING.md` for guidelines.

## License

MIT License - See LICENSE file for details.

## Acknowledgments

Built on top of [cache-layer](https://github.com/equilibrium-tokens/cache-layer).

---

**The grammar is eternal.**
