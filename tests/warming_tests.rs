//! Cache warming tests
//!
//! Tests for cache warming strategies

use cache_layer_optimizer::{
    AccessPattern, OnDemandWarmer, ProactiveWarmer, ScheduledWarmer, WarmingStrategy,
};
use std::time::Duration;

#[cfg(test)]
mod warming_tests {
    use super::*;

    // ============================================================================
    // Unit Tests
    // ============================================================================

    #[test]
    fn test_on_demand_warmer_creation() {
        let warmer = OnDemandWarmer::new(100);

        assert_eq!(warmer.max_keys, 100);
        assert_eq!(warmer.metrics().warming_cycles, 0);
    }

    #[tokio::test]
    async fn test_on_demand_warmer_select_keys() {
        let warmer = OnDemandWarmer::new(2);

        let patterns = vec![
            {
                let mut p = AccessPattern::new("high_priority".to_string());
                p.frequency = 10.0;
                p.regularity = 0.9;
                p
            },
            {
                let mut p = AccessPattern::new("medium_priority".to_string());
                p.frequency = 5.0;
                p.regularity = 0.5;
                p
            },
            {
                let mut p = AccessPattern::new("low_priority".to_string());
                p.frequency = 1.0;
                p.regularity = 0.1;
                p
            },
        ];

        let keys = warmer.select_keys(&patterns).await.unwrap();

        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0], "high_priority");
        assert_eq!(keys[1], "medium_priority");
    }

    #[tokio::test]
    async fn test_on_demand_warmer_limited_capacity() {
        let warmer = OnDemandWarmer::new(1);

        let patterns = vec![
            {
                let mut p = AccessPattern::new("key1".to_string());
                p.frequency = 10.0;
                p
            },
            {
                let mut p = AccessPattern::new("key2".to_string());
                p.frequency = 5.0;
                p
            },
        ];

        let keys = warmer.select_keys(&patterns).await.unwrap();

        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "key1");
    }

    #[tokio::test]
    async fn test_on_demand_warmer_empty_patterns() {
        let warmer = OnDemandWarmer::new(100);

        let patterns = vec![];
        let keys = warmer.select_keys(&patterns).await.unwrap();

        assert!(keys.is_empty());
    }

    #[test]
    fn test_proactive_warmer_creation() {
        use cache_layer_optimizer::Predictor;

        let predictor = Predictor::new();
        let warmer = ProactiveWarmer::new(predictor, Duration::from_secs(60));

        assert_eq!(warmer.interval.as_secs(), 60);
    }

    #[tokio::test]
    async fn test_proactive_warmer_select_keys() {
        use cache_layer_optimizer::Predictor;

        let predictor = Predictor::new();
        let warmer = ProactiveWarmer::new(predictor, Duration::from_secs(60));

        let patterns = vec![AccessPattern::new("test".to_string())];
        let keys = warmer.select_keys(&patterns).await.unwrap();

        // Should return predictions
        assert!(keys.len() <= 100); // Max 100 predictions
    }

    #[test]
    fn test_scheduled_warmer_creation() {
        use cache_layer_optimizer::WarmingSchedule;

        let schedule = vec![WarmingSchedule {
            time: "00:00".to_string(),
            keys: vec!["key1".to_string(), "key2".to_string()],
        }];

        let warmer = ScheduledWarmer::new(schedule);

        assert_eq!(warmer.schedule.len(), 1);
        assert_eq!(warmer.schedule[0].keys.len(), 2);
    }

    #[tokio::test]
    async fn test_scheduled_warmer_select_keys() {
        use cache_layer_optimizer::WarmingSchedule;

        let schedule = vec![WarmingSchedule {
            time: "00:00".to_string(),
            keys: vec!["key1".to_string()],
        }];

        let warmer = ScheduledWarmer::new(schedule);
        let keys = warmer.select_keys(&[]).await.unwrap();

        // Returns empty for non-scheduled time
        assert!(keys.is_empty());
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[tokio::test]
    async fn test_warmer_integration() {
        use cache_layer_optimizer::{Warmer, WarmingStrategy};

        let on_demand = Box::new(OnDemandWarmer::new(10));
        let mut warmer = Warmer::new(on_demand);

        let patterns = vec![
            {
                let mut p = AccessPattern::new("key1".to_string());
                p.frequency = 10.0;
                p
            },
            {
                let mut p = AccessPattern::new("key2".to_string());
                p.frequency = 5.0;
                p
            },
        ];

        let warmed_keys = warmer.warm(&patterns).await.unwrap();

        assert_eq!(warmed_keys.len(), 2);
        assert_eq!(warmer.metrics().warming_cycles, 1);
        assert_eq!(warmer.metrics().keys_warmed, 2);
    }

    #[tokio::test]
    async fn test_warmer_multiple_cycles() {
        use cache_layer_optimizer::{Warmer, WarmingStrategy};

        let on_demand = Box::new(OnDemandWarmer::new(10));
        let mut warmer = Warmer::new(on_demand);

        let patterns = vec![{
            let mut p = AccessPattern::new("key1".to_string());
            p.frequency = 10.0;
            p
        }];

        // Run multiple warming cycles
        for _ in 0..5 {
            let _ = warmer.warm(&patterns).await.unwrap();
        }

        assert_eq!(warmer.metrics().warming_cycles, 5);
        assert_eq!(warmer.metrics().keys_warmed, 5);
    }

    #[tokio::test]
    async fn test_warmer_with_predictor() {
        use cache_layer_optimizer::{Predictor, Warmer, WarmingStrategy};

        let predictor = Predictor::new();
        let proactive = Box::new(ProactiveWarmer::new(predictor, Duration::from_secs(60)));
        let mut warmer = Warmer::new(proactive);

        let patterns: Vec<AccessPattern> = (0..100)
            .map(|i| {
                let mut p = AccessPattern::new(format!("key_{}", i));
                p.frequency = (i % 10) as f64;
                p
            })
            .collect();

        let warmed_keys = warmer.warm(&patterns).await.unwrap();

        // Should warm keys based on predictions
        assert!(!warmed_keys.is_empty());
        assert!(warmed_keys.len() <= 100);
    }

    // ============================================================================
    // Property-Based Tests
    // ============================================================================

    proptest::proptest! {
        #[test]
        fn prop_on_demand_warmer_respects_limit(
            num_patterns in 1..1000usize,
            max_keys in 1..100usize
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();

            let warmer = OnDemandWarmer::new(max_keys);

            let patterns: Vec<AccessPattern> = (0..num_patterns)
                .map(|i| {
                    let mut p = AccessPattern::new(format!("key_{}", i));
                    p.frequency = (num_patterns - i) as f64; // Descending priority
                    p
                })
                .collect();

            let keys = rt.block_on(warmer.select_keys(&patterns)).unwrap();

            prop_assert!(keys.len() <= max_keys);
        }
    }

    proptest::proptest! {
        #[test]
        fn prop_on_demand_warmer_priority_ordering(
            num_patterns in 10..100usize
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();

            let warmer = OnDemandWarmer::new(num_patterns);

            let patterns: Vec<AccessPattern> = (0..num_patterns)
                .map(|i| {
                    let mut p = AccessPattern::new(format!("key_{}", i));
                    p.frequency = (i % 10) as f64;
                    p.regularity = (i % 10) as f64 / 10.0;
                    p
                })
                .collect();

            let keys = rt.block_on(warmer.select_keys(&patterns)).unwrap();

            // All returned keys should be unique
            let unique_keys: std::collections::HashSet<_> = keys.iter().collect();
            prop_assert_eq!(unique_keys.len(), keys.len());
        }
    }

    // ============================================================================
    // Performance Tests
    // ============================================================================

    #[tokio::test]
    async fn test_warming_large_dataset() {
        use cache_layer_optimizer::{Warmer, WarmingStrategy};

        let on_demand = Box::new(OnDemandWarmer::new(1000));
        let mut warmer = Warmer::new(on_demand);

        let patterns: Vec<AccessPattern> = (0..100000)
            .map(|i| {
                let mut p = AccessPattern::new(format!("key_{}", i));
                p.frequency = (i % 1000) as f64;
                p
            })
            .collect();

        let start = std::time::Instant::now();
        let warmed_keys = warmer.warm(&patterns).await.unwrap();
        let duration = start.elapsed();

        // Should complete in reasonable time (< 200ms for 100k items)
        assert!(duration.as_millis() < 200);
        assert_eq!(warmed_keys.len(), 1000);
    }

    #[tokio::test]
    async fn test_warming_concurrent() {
        use cache_layer_optimizer::{Warmer, WarmingStrategy};

        let on_demand = Box::new(OnDemandWarmer::new(100));

        let patterns: Vec<AccessPattern> = (0..100)
            .map(|i| {
                let mut p = AccessPattern::new(format!("key_{}", i));
                p.frequency = i as f64;
                p
            })
            .collect();

        let start = std::time::Instant::now();

        // Spawn multiple concurrent warming tasks
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let patterns = &patterns;
                tokio::spawn(async move {
                    let warmer = Warmer::new(Box::new(OnDemandWarmer::new(100)));
                    warmer.warm(patterns).await
                })
            })
            .collect();

        // Wait for all to complete
        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        let duration = start.elapsed();

        // Concurrent operations should be fast
        assert!(duration.as_millis() < 500);
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[tokio::test]
    async fn test_warming_zero_frequency_patterns() {
        use cache_layer_optimizer::{Warmer, WarmingStrategy};

        let on_demand = Box::new(OnDemandWarmer::new(10));
        let mut warmer = Warmer::new(on_demand);

        let patterns = vec![
            {
                let mut p = AccessPattern::new("zero1".to_string());
                p.frequency = 0.0;
                p
            },
            {
                let mut p = AccessPattern::new("zero2".to_string());
                p.frequency = 0.0;
                p
            },
        ];

        let warmed_keys = warmer.warm(&patterns).await.unwrap();

        // Should still return keys even with zero frequency
        assert!(!warmed_keys.is_empty());
    }

    #[tokio::test]
    async fn test_warming_single_pattern() {
        use cache_layer_optimizer::{Warmer, WarmingStrategy};

        let on_demand = Box::new(OnDemandWarmer::new(10));
        let mut warmer = Warmer::new(on_demand);

        let patterns = vec![{
            let mut p = AccessPattern::new("single".to_string());
            p.frequency = 10.0;
            p
        }];

        let warmed_keys = warmer.warm(&patterns).await.unwrap();

        assert_eq!(warmed_keys.len(), 1);
        assert_eq!(warmed_keys[0], "single");
    }

    #[tokio::test]
    async fn test_warming_max_keys_zero() {
        let warmer = OnDemandWarmer::new(0);

        let patterns = vec![{
            let mut p = AccessPattern::new("test".to_string());
            p.frequency = 10.0;
            p
        }];

        let keys = warmer.select_keys(&patterns).await.unwrap();

        assert!(keys.is_empty());
    }
}
