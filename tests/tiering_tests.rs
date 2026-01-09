//! Dynamic tier sizing tests
//!
//! Tests for adaptive cache tier sizing strategies

use cache_layer_optimizer::{
    AdaptiveTiering, DynamicTiering, FixedTiering, TierConfig, TierManager, TierSizing,
};
use std::time::Duration;

#[cfg(test)]
mod tiering_tests {
    use super::*;

    // ============================================================================
    // Unit Tests
    // ============================================================================

    #[test]
    fn test_tier_config_creation() {
        let config = TierConfig::new("l1".to_string(), 1024);

        assert_eq!(config.tier_name, "l1");
        assert_eq!(config.max_size, 1024);
        assert_eq!(config.current_size, 512);
        assert_eq!(config.target_size, 512);
    }

    #[test]
    fn test_tier_config_utilization() {
        let mut config = TierConfig::new("l1".to_string(), 1024);
        config.current_size = 768;

        let utilization = config.utilization();

        assert_eq!(utilization, 0.75);
    }

    #[test]
    fn test_tier_config_full_utilization() {
        let mut config = TierConfig::new("l1".to_string(), 1024);
        config.current_size = 1024;

        let utilization = config.utilization();

        assert_eq!(utilization, 1.0);
    }

    #[test]
    fn test_tier_config_empty_utilization() {
        let mut config = TierConfig::new("l1".to_string(), 1024);
        config.current_size = 0;

        let utilization = config.utilization();

        assert_eq!(utilization, 0.0);
    }

    #[test]
    fn test_fixed_tiering_creation() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let tiering = FixedTiering::new(configs.clone());

        assert_eq!(tiering.configs.len(), 1);
    }

    #[tokio::test]
    async fn test_fixed_tiering_calculate_sizes() {
        let configs = vec![
            TierConfig::new("l1".to_string(), 1024),
            TierConfig::new("l2".to_string(), 2048),
        ];
        let tiering = FixedTiering::new(configs.clone());

        let result = tiering.calculate_sizes(&tiering.metrics).await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].tier_name, "l1");
        assert_eq!(result[1].tier_name, "l2");
    }

    #[tokio::test]
    async fn test_fixed_tiering_unchanged() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let mut tiering = FixedTiering::new(configs.clone());

        let result = tiering.calculate_sizes(&tiering.metrics).await.unwrap();
        tiering.adjust_sizes(&result).await.unwrap();

        // Fixed tiering should not change sizes
        assert_eq!(tiering.configs[0].current_size, configs[0].current_size);
    }

    #[test]
    fn test_adaptive_tiering_creation() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let tiering = AdaptiveTiering::new(configs, Duration::from_secs(60));

        assert_eq!(tiering.adjustment_interval.as_secs(), 60);
        assert_eq!(tiering.hit_rate_threshold, 0.8);
    }

    #[tokio::test]
    async fn test_adaptive_tiering_calculate_sizes() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let tiering = AdaptiveTiering::new(configs, Duration::from_secs(60));

        let result = tiering.calculate_sizes(&tiering.metrics).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tier_name, "l1");
    }

    #[tokio::test]
    async fn test_adaptive_tiering_grow() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let mut tiering = AdaptiveTiering::new(configs, Duration::from_secs(60));

        tiering.grow_tier(0).unwrap();

        assert_eq!(tiering.configs[0].current_size, 768); // 512 * 1.5
        assert_eq!(tiering.configs[0].target_size, 768);
    }

    #[tokio::test]
    async fn test_adaptive_tiering_shrink() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let mut tiering = AdaptiveTiering::new(configs, Duration::from_secs(60));

        tiering.shrink_tier(0).unwrap();

        assert_eq!(tiering.configs[0].current_size, 384); // 512 * 0.75
        assert_eq!(tiering.configs[0].target_size, 384);
    }

    #[tokio::test]
    async fn test_adaptive_tiering_multiple_adjustments() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let mut tiering = AdaptiveTiering::new(configs, Duration::from_secs(60));

        // Grow then shrink
        tiering.grow_tier(0).unwrap();
        assert_eq!(tiering.configs[0].current_size, 768);

        tiering.shrink_tier(0).unwrap();
        assert_eq!(tiering.configs[0].current_size, 576); // 768 * 0.75
    }

    #[tokio::test]
    async fn test_adaptive_tiering_grow_to_max() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let mut tiering = AdaptiveTiering::new(configs, Duration::from_secs(60));

        // Grow to exceed max
        tiering.configs[0].current_size = 800;
        tiering.grow_tier(0).unwrap();

        // Should cap at max_size
        assert_eq!(tiering.configs[0].current_size, 1024);
    }

    #[tokio::test]
    async fn test_adaptive_tiering_shrink_to_min() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let mut tiering = AdaptiveTiering::new(configs, Duration::from_secs(60));

        // Shrink multiple times to hit minimum
        for _ in 0..10 {
            tiering.shrink_tier(0).unwrap();
        }

        // Should not go below 10% of max
        assert!(tiering.configs[0].current_size >= 102);
    }

    #[test]
    fn test_adaptive_tiering_invalid_index() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let mut tiering = AdaptiveTiering::new(configs, Duration::from_secs(60));

        let result = tiering.grow_tier(5);

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dynamic_tiering_creation() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let tiering = DynamicTiering::new(configs);

        assert_eq!(tiering.configs.len(), 1);
    }

    #[tokio::test]
    async fn test_dynamic_tiering_calculate_sizes() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let tiering = DynamicTiering::new(configs);

        let result = tiering.calculate_sizes(&tiering.metrics).await.unwrap();

        assert_eq!(result.len(), 1);
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[tokio::test]
    async fn test_tier_manager_with_fixed() {
        let configs = vec![
            TierConfig::new("l1".to_string(), 1024),
            TierConfig::new("l2".to_string(), 2048),
        ];
        let tiering = FixedTiering::new(configs);
        let mut manager = TierManager::new(Box::new(tiering));

        let result = manager.optimize_tiers().await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(manager.metrics().sizing_adjustments, 1);
    }

    #[tokio::test]
    async fn test_tier_manager_with_adaptive() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let tiering = AdaptiveTiering::new(configs, Duration::from_secs(60));
        let mut manager = TierManager::new(Box::new(tiering));

        let result = manager.optimize_tiers().await.unwrap();

        assert_eq!(result.len(), 1);
        assert!(manager.metrics().avg_adjustment_time.as_millis() >= 0);
    }

    #[tokio::test]
    async fn test_tier_manager_multiple_optimizations() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let tiering = FixedTiering::new(configs);
        let mut manager = TierManager::new(Box::new(tiering));

        for _ in 0..5 {
            manager.optimize_tiers().await.unwrap();
        }

        assert_eq!(manager.metrics().sizing_adjustments, 5);
    }

    // ============================================================================
    // Property-Based Tests
    // ============================================================================

    proptest::proptest! {
        #[test]
        fn prop_tier_config_utilization(
            max_size in 1024..10240u64,
            current_size in 0..10240u64
        ) {
            let mut config = TierConfig::new("test".to_string(), max_size);
            config.current_size = current_size.min(max_size);

            let utilization = config.utilization();

            prop_assert!(utilization >= 0.0);
            prop_assert!(utilization <= 1.0);
        }
    }

    proptest::proptest! {
        #[test]
        fn prop_adaptive_tiering_grow_factor(
            initial_size in 100..1000u64,
            max_size in 1000..10000u64
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();

            let mut config = TierConfig::new("test".to_string(), max_size);
            config.current_size = initial_size;

            let mut tiering = AdaptiveTiering::new(vec![config], Duration::from_secs(60));
            rt.block_on(tiering.grow_tier(0)).unwrap();

            let new_size = tiering.configs[0].current_size;

            // Should grow by growth_factor (1.5) but not exceed max
            prop_assert!(new_size >= initial_size);
            prop_assert!(new_size <= max_size);
        }
    }

    proptest::proptest! {
        #[test]
        fn prop_adaptive_tiering_shrink_factor(
            initial_size in 200..1000u64
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();

            let mut config = TierConfig::new("test".to_string(), 10000);
            config.current_size = initial_size;

            let mut tiering = AdaptiveTiering::new(vec![config], Duration::from_secs(60));
            rt.block_on(tiering.shrink_tier(0)).unwrap();

            let new_size = tiering.configs[0].current_size;

            // Should shrink by shrink_factor (0.75) but not go below 10%
            prop_assert!(new_size < initial_size);
            prop_assert!(new_size >= 1000); // 10% of 10000
        }
    }

    // ============================================================================
    // Performance Tests
    // ============================================================================

    #[tokio::test]
    async fn test_tiering_many_tiers() {
        let configs: Vec<TierConfig> =
            (0..100).map(|i| TierConfig::new(format!("tier_{}", i), 1024)).collect();

        let tiering = FixedTiering::new(configs);
        let mut manager = TierManager::new(Box::new(tiering));

        let start = std::time::Instant::now();
        let result = manager.optimize_tiers().await.unwrap();
        let duration = start.elapsed();

        assert_eq!(result.len(), 100);
        assert!(duration.as_millis() < 50);
    }

    #[tokio::test]
    async fn test_tiering_concurrent_optimizations() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let tiering = FixedTiering::new(configs);

        let start = std::time::Instant::now();

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let tiering = FixedTiering::new(vec![TierConfig::new("l1".to_string(), 1024)]);
                tokio::spawn(async move {
                    let mut manager = TierManager::new(Box::new(tiering));
                    manager.optimize_tiers().await
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        let duration = start.elapsed();

        // Concurrent operations should be fast
        assert!(duration.as_millis() < 100);
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[tokio::test]
    async fn test_tiering_empty_configs() {
        let tiering = FixedTiering::new(vec![]);
        let mut manager = TierManager::new(Box::new(tiering));

        let result = manager.optimize_tiers().await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_tiering_zero_max_size() {
        let config = TierConfig::new("zero".to_string(), 0);
        assert_eq!(config.max_size, 0);
        assert_eq!(config.current_size, 0);
    }

    #[tokio::test]
    async fn test_adaptive_tiering_extreme_growth() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let mut tiering = AdaptiveTiering::new(configs, Duration::from_secs(60));

        // Try to grow many times
        for _ in 0..20 {
            let _ = tiering.grow_tier(0);
        }

        // Should cap at max_size
        assert_eq!(tiering.configs[0].current_size, 1024);
    }

    #[tokio::test]
    async fn test_adaptive_tiering_extreme_shrinking() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let mut tiering = AdaptiveTiering::new(configs, Duration::from_secs(60));

        // Shrink many times
        for _ in 0..20 {
            let _ = tiering.shrink_tier(0);
        }

        // Should stop at minimum (10% of max)
        assert_eq!(tiering.configs[0].current_size, 102);
    }
}
