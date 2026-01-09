# Testing Strategy for cache-layer-optimizer

## Overview

This document outlines the comprehensive testing strategy for **cache-layer-optimizer**, a Rust library providing advanced caching strategies and optimizations. The testing approach ensures reliability, performance, and correctness across all optimizer components.

## Testing Philosophy

Our testing strategy follows these principles:

1. **Test Pyramid**: Unit tests → Integration tests → Performance benchmarks
2. **Property-Based Testing**: Use proptest for invariant verification
3. **Performance Validation**: Ensure performance targets are met
4. **Concurrent Safety**: Test async operations under high concurrency
5. **Edge Case Coverage**: Validate boundary conditions and error paths

## Test Categories

### 1. Predictive Caching Tests

**Location**: `/tests/predictive_tests.rs`

**Coverage**:
- Access pattern creation and priority scoring
- Frequency-based prediction
- Recency-based prediction
- ML-based prediction (with `predictive` feature)
- Predictor composition and integration

**Key Test Scenarios**:
```rust
// Unit tests
- test_access_pattern_creation
- test_access_pattern_priority_score
- test_frequency_predictor_high_frequency_keys
- test_recency_predictor_recent_keys
- test_predictor_disabled

// Property-based tests
- prop_access_pattern_priority_score
- prop_frequency_predictor_filters

// Integration tests
- test_predictor_with_multiple_models
- test_predictor_metrics_tracking

// Performance tests
- test_predictor_large_dataset (10k items < 100ms)
- test_predictor_concurrent_predictions
```

**Performance Targets**:
- Prediction on 10k patterns: < 100ms
- Concurrent predictions (10 threads): < 200ms
- Memory overhead: < 10MB for 100k patterns

### 2. Cache Warming Tests

**Location**: `/tests/warming_tests.rs`

**Coverage**:
- On-demand warming strategies
- Proactive warming with predictions
- Scheduled warming
- Warmer integration and metrics

**Key Test Scenarios**:
```rust
// Unit tests
- test_on_demand_warmer_select_keys
- test_on_demand_warmer_limited_capacity
- test_proactive_warmer_select_keys
- test_scheduled_warmer_select_keys

// Property-based tests
- prop_on_demand_warmer_respects_limit
- prop_on_demand_warmer_priority_ordering

// Integration tests
- test_warmer_integration
- test_warmer_multiple_cycles
- test_warmer_with_predictor

// Performance tests
- test_warming_large_dataset (100k items < 200ms)
- test_warming_concurrent (10 concurrent < 500ms)
```

**Performance Targets**:
- Warming 100k patterns: < 200ms
- Concurrent warming (10 threads): < 500ms
- Keys warmed per cycle: Configurable (default 100-1000)

### 3. Dynamic Tiering Tests

**Location**: `/tests/tiering_tests.rs`

**Coverage**:
- Tier configuration and utilization
- Fixed tier sizing
- Adaptive tier sizing (grow/shrink)
- Dynamic tier sizing
- Tier manager integration

**Key Test Scenarios**:
```rust
// Unit tests
- test_tier_config_utilization
- test_adaptive_tiering_grow
- test_adaptive_tiering_shrink
- test_adaptive_tiering_multiple_adjustments

// Property-based tests
- prop_tier_config_utilization
- prop_adaptive_tiering_grow_factor
- prop_adaptive_tiering_shrink_factor

// Integration tests
- test_tier_manager_with_fixed
- test_tier_manager_with_adaptive
- test_tier_manager_multiple_optimizations

// Performance tests
- test_tiering_many_tiers (100 tiers < 50ms)
- test_tiering_concurrent_optimizations
```

**Performance Targets**:
- Optimize 100 tiers: < 50ms
- Grow/shrink operations: < 1ms each
- Memory efficiency: > 30% improvement over static sizing

### 4. Coherence Protocol Tests

**Location**: `/tests/coherence_tests.rs`

**Coverage**:
- Write-through protocol
- Write-back protocol
- Write-around protocol
- Coherence metrics tracking
- Dirty key management (write-back)

**Key Test Scenarios**:
```rust
// Unit tests
- test_write_through_invalidate
- test_write_back_update
- test_write_around_invalidate
- test_write_back_dirty_tracking

// Property-based tests
- prop_write_through_invalidation_count
- prop_write_back_dirty_tracking
- prop_coherence_timing

// Integration tests
- test_coherence_metrics_tracking
- test_coherence_invalidation_timing
- test_write_through_vs_write_back

// Performance tests
- test_coherence_high_throughput (10k ops < 500ms)
- test_coherence_concurrent_stress (10k ops < 1s)
```

**Performance Targets**:
- 10k invalidations: < 500ms
- 10k updates: < 500ms
- Concurrent operations (100 threads × 100 ops): < 1s
- Coherence overhead: < 5%

### 5. Integration Tests

**Location**: `/tests/integration_tests.rs` (if needed)

**Coverage**:
- End-to-end optimizer workflows
- Combined predictor + warmer + tiering
- Real-world usage patterns
- Metrics aggregation

## Benchmark Suite

**Location**: `/tests/benches/`

### Benchmark Categories

1. **predictive_cache_bench.rs**
   - Access pattern priority scoring
   - Frequency/Recency predictor performance
   - Predictor training throughput

2. **cache_warming_bench.rs**
   - On-demand warming throughput
   - Proactive warming performance
   - Multi-cycle warming

3. **dynamic_tiering_bench.rs**
   - Tier configuration calculations
   - Adaptive grow/shrink operations
   - Tier manager optimization

4. **coherence_bench.rs**
   - Protocol comparison (write-through/write-back/write-around)
   - Mixed operation throughput
   - Concurrent invalidation/update

5. **throughput_bench.rs**
   - End-to-end optimizer performance
   - Scalability testing (1K - 1M patterns)
   - Consecutive optimization runs

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench predictive_cache_bench

# Generate flamegraph
cargo bench --bench throughput_bench -- --profile-time=5
```

## Test Execution

### Running All Tests

```bash
# Run all tests (default features)
cargo test

# Run tests with full features
cargo test --features full

# Run tests in release mode (faster)
cargo test --release

# Run tests with output
cargo test -- --nocapture --test-threads=1
```

### Running Specific Test Categories

```bash
# Predictive caching tests
cargo test --test predictive_tests

# Warming tests
cargo test --test warming_tests

# Tiering tests
cargo test --test tiering_tests

# Coherence tests
cargo test --test coherence_tests
```

### Running Property-Based Tests

```bash
# Run property tests with more iterations
cargo test --test predictive_tests prop_ -- --test-threads=1

# Run with custom test case count
PROPTEST_CASES=10000 cargo test --test predictive_tests prop_
```

## Continuous Integration

### CI Test Matrix

```yaml
test_matrix:
  - rust: stable
    features: default
    test_args: --all-targets
  - rust: nightly
    features: full
    test_args: --all-targets
  - rust: stable
    features: predictive
    test_args: --test predictive_tests
  - rust: stable
    mode: release
    test_args: --benches
```

### CI Performance Regression Detection

```yaml
benchmarks:
  - predictor_10k: < 100ms
  - warming_100k: < 200ms
  - tiering_100: < 50ms
  - coherence_10k: < 500ms
  - optimizer_full_10k: < 200ms
```

## Performance Validation

### Target Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Hit Rate Improvement | > 10% | Cache hit rate comparison |
| Latency Reduction | > 20% | P95 latency measurement |
| Memory Efficiency | > 30% | Memory usage comparison |
| Prediction Accuracy | > 70% | Correct predictions / total predictions |
| Warming Hit Rate | > 60% | Warmed keys hit / total warmed |

### Measuring Performance

```bash
# Run performance validation
cargo test --test performance_tests --release -- --nocapture

# Generate performance report
cargo bench -- | tee benchmark_results.txt
```

## Code Coverage

### Coverage Goals

- Overall line coverage: **> 80%**
- Branch coverage: **> 70%**
- Critical path coverage: **> 90%**

### Generating Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/

# View coverage report
open coverage/index.html
```

## Mocking and Test Utilities

### Mock Dependencies

```rust
// Mock cache layer for testing
pub struct MockCache {
    data: HashMap<String, Vec<u8>>,
}

impl MockCache {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    pub async fn set(&mut self, key: String, value: Vec<u8>) {
        self.data.insert(key, value);
    }
}
```

### Test Data Generation

```rust
// Generate realistic access patterns
pub fn generate_test_patterns(count: usize) -> Vec<AccessPattern> {
    (0..count).map(|i| {
        let mut pattern = AccessPattern::new(format!("key_{}", i));
        pattern.frequency = (i % 100) as f64;
        pattern.regularity = (i % 10) as f64 / 10.0;
        pattern
    }).collect()
}
```

## Stress Testing

### High Load Scenarios

```bash
# Run stress tests
cargo test --test stress_tests --release

# Specific stress tests
- test_predictor_1m_patterns
- test_warming_10_concurrent
- test_coherence_100_concurrent
- test_optimizer_1000_cycles
```

### Resource Limits

```rust
// Test memory limits
const MAX_MEMORY_MB: usize = 100;

// Test timeout limits
const TEST_TIMEOUT_SECS: u64 = 30;
```

## Error Handling Tests

### Error Scenarios

```rust
// Insufficient data for ML training
#[test]
async fn test_ml_predictor_insufficient_data() {
    let predictor = MlPredictor::new(100);
    let patterns = vec![AccessPattern::new("test".to_string())];

    let result = predictor.train(&patterns).await;

    assert!(result.is_err());
}

// Invalid tier index
#[test]
async fn test_adaptive_tiering_invalid_index() {
    let configs = vec![TierConfig::new("l1".to_string(), 1024)];
    let mut tiering = AdaptiveTiering::new(configs, Duration::from_secs(60));

    let result = tiering.grow_tier(5);

    assert!(result.is_err());
}
```

## Documentation Examples as Tests

### Running Doc Tests

```bash
# Test code in documentation
cargo test --doc

# Test specific module
cargo test --doc cache_layer_optimizer::predictive
```

## Test Maintenance

### Regular Test Reviews

- Weekly: Review flaky tests
- Monthly: Update performance baselines
- Quarterly: Audit test coverage

### Test Metrics

Track and monitor:
- Test execution time
- Flaky test rate
- Code coverage trends
- Benchmark performance trends

## Debugging Failed Tests

### Useful Commands

```bash
# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Run single test with backtrace
RUST_BACKTRACE=1 cargo test test_name -- --exact

# Run under gdb
cargo test -- --nocapture -- -gdb
```

### Common Issues

1. **Flaky async tests**: Use `--test-threads=1`
2. **Race conditions**: Add proper synchronization
3. **Performance variance**: Run multiple iterations
4. **Resource leaks**: Add cleanup in test teardown

## Summary

This comprehensive testing strategy ensures:

- **Correctness**: Unit and integration tests validate functionality
- **Performance**: Benchmarks enforce performance targets
- **Reliability**: Property tests verify invariants
- **Scalability**: Stress tests validate under high load
- **Safety**: Concurrent tests ensure thread safety

The test suite is designed to be fast, reliable, and maintainable while providing comprehensive coverage of all optimizer functionality.
