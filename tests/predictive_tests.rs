//! Predictive caching tests
//!
//! Tests for ML-based cache prediction strategies

use cache_layer_optimizer::{
    AccessPattern, FrequencyPredictor, MlPredictor, PredictionModel, Predictor, RecencyPredictor,
};
use chrono::{Duration, Utc};
use proptest::prelude::*;

#[cfg(test)]
mod predictive_tests {
    use super::*;

    // ============================================================================
    // Unit Tests
    // ============================================================================

    #[test]
    fn test_access_pattern_creation() {
        let pattern = AccessPattern::new("test_key".to_string());

        assert_eq!(pattern.key, "test_key");
        assert_eq!(pattern.frequency, 0.0);
        assert_eq!(pattern.regularity, 0.0);
        assert!(pattern.predicted_next_access.is_none());
    }

    #[test]
    fn test_access_pattern_priority_score() {
        let mut pattern = AccessPattern::new("test_key".to_string());
        pattern.frequency = 10.0;
        pattern.recency = std::time::Duration::from_secs(100);
        pattern.regularity = 0.8;

        let score = pattern.priority_score();

        // Score should be between 0 and 1
        assert!(score >= 0.0 && score <= 10.0); // frequency can push it above 1
        assert!(score > 0.0); // With frequency=10, score should be significant
    }

    #[test]
    fn test_access_pattern_priority_zero_frequency() {
        let pattern = AccessPattern::new("test_key".to_string());

        let score = pattern.priority_score();

        // Zero frequency should still give some score due to recency and regularity
        assert!(score >= 0.0);
    }

    #[test]
    fn test_frequency_predictor_creation() {
        let predictor = FrequencyPredictor::new(1.0, 100);

        assert_eq!(predictor.min_frequency, 1.0);
        assert_eq!(predictor.window_size, 100);
        assert_eq!(predictor.metrics().training_runs, 0);
    }

    #[test]
    fn test_frequency_predictor_high_frequency_keys() {
        let predictor = FrequencyPredictor::new(5.0, 100);

        let patterns = vec![
            {
                let mut p = AccessPattern::new("high_freq".to_string());
                p.frequency = 10.0;
                p
            },
            {
                let mut p = AccessPattern::new("low_freq".to_string());
                p.frequency = 1.0;
                p
            },
        ];

        let high_freq_keys = predictor.get_high_frequency_keys(&patterns);

        assert_eq!(high_freq_keys.len(), 1);
        assert_eq!(high_freq_keys[0], "high_freq");
    }

    #[tokio::test]
    async fn test_frequency_predictor_train() {
        let mut predictor = FrequencyPredictor::new(1.0, 100);

        let patterns = vec![AccessPattern::new("test".to_string())];
        let result = predictor.train(&patterns).await;

        assert!(result.is_ok());
        assert_eq!(predictor.metrics().training_runs, 1);
        assert_eq!(predictor.metrics().training_samples, 1);
    }

    #[test]
    fn test_recency_predictor_creation() {
        let predictor = RecencyPredictor::new(std::time::Duration::from_secs(300));

        assert_eq!(predictor.recency_threshold.as_secs(), 300);
        assert_eq!(predictor.metrics().training_runs, 0);
    }

    #[tokio::test]
    async fn test_recency_predictor_train() {
        let mut predictor = RecencyPredictor::new(std::time::Duration::from_secs(300));

        let patterns = vec![AccessPattern::new("test".to_string())];
        let result = predictor.train(&patterns).await;

        assert!(result.is_ok());
        assert_eq!(predictor.metrics().training_runs, 1);
    }

    #[test]
    fn test_recency_predictor_recent_keys() {
        let predictor = RecencyPredictor::new(std::time::Duration::from_secs(300));

        let now = Utc::now();
        let patterns = vec![
            {
                let mut p = AccessPattern::new("recent".to_string());
                p.last_access = now - chrono::Duration::seconds(100);
                p
            },
            {
                let mut p = AccessPattern::new("old".to_string());
                p.last_access = now - chrono::Duration::seconds(400);
                p
            },
        ];

        let recent_keys = predictor.get_recent_keys(&patterns);

        assert_eq!(recent_keys.len(), 1);
        assert_eq!(recent_keys[0], "recent");
    }

    #[test]
    fn test_predictor_creation() {
        let predictor = Predictor::new();

        assert!(predictor.enabled);
        assert_eq!(predictor.metrics().prediction_requests, 0);
    }

    #[tokio::test]
    async fn test_predictor_disabled() {
        let mut predictor = Predictor::new();
        predictor.set_enabled(false);

        let patterns = vec![AccessPattern::new("test".to_string())];
        let predictions = predictor.get_predictions(&patterns).await.unwrap();

        assert!(predictions.is_empty());
        assert_eq!(predictor.metrics().prediction_requests, 1);
    }

    #[tokio::test]
    async fn test_predictor_get_predictions() {
        let mut pattern = AccessPattern::new("high_priority".to_string());
        pattern.frequency = 10.0;
        pattern.regularity = 0.9;
        pattern.recency = std::time::Duration::from_secs(10);

        let patterns = vec![pattern];
        let predictor = Predictor::new();

        let predictions = predictor.get_predictions(&patterns).await.unwrap();

        // Should return high priority key
        assert!(!predictions.is_empty());
    }

    // ============================================================================
    // Property-Based Tests
    // ============================================================================

    proptest! {
        #[test]
        fn prop_access_pattern_priority_score(
            frequency in 0.0..100.0,
            recency_secs in 0u64..3600,
            regularity in 0.0..1.0
        ) {
            let mut pattern = AccessPattern::new("test".to_string());
            pattern.frequency = frequency;
            pattern.recency = std::time::Duration::from_secs(recency_secs);
            pattern.regularity = regularity;

            let score = pattern.priority_score();

            // Score should always be non-negative
            prop_assert!(score >= 0.0);

            // Higher frequency should increase score
            prop_assert!(score <= frequency * 0.5 + 1.0); // Maximum possible contribution from other factors
        }
    }

    proptest! {
        #[test]
        fn prop_frequency_predictor_filters(
            min_freq in 1.0..50.0,
            freq1 in 0.0..100.0,
            freq2 in 0.0..100.0
        ) {
            let predictor = FrequencyPredictor::new(min_freq, 100);

            let patterns = vec![
                {
                    let mut p = AccessPattern::new("key1".to_string());
                    p.frequency = freq1;
                    p
                },
                {
                    let mut p = AccessPattern::new("key2".to_string());
                    p.frequency = freq2;
                    p
                },
            ];

            let high_freq_keys = predictor.get_high_frequency_keys(&patterns);

            // All returned keys should have frequency >= min_freq
            for key in &high_freq_keys {
                let pattern = patterns.iter().find(|p| &p.key == key).unwrap();
                prop_assert!(pattern.frequency >= min_freq);
            }
        }
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[tokio::test]
    async fn test_predictor_with_multiple_models() {
        let predictor = Predictor::new()
            .add_model(Box::new(FrequencyPredictor::new(1.0, 100)))
            .add_model(Box::new(RecencyPredictor::new(std::time::Duration::from_secs(300))));

        let mut pattern = AccessPattern::new("test".to_string());
        pattern.frequency = 10.0;
        pattern.regularity = 0.8;

        let patterns = vec![pattern];
        let predictions = predictor.get_predictions(&patterns).await.unwrap();

        // Should return predictions from both models
        assert!(!predictions.is_empty());
    }

    #[tokio::test]
    async fn test_predictor_metrics_tracking() {
        let predictor = Predictor::new();

        let patterns = vec![
            {
                let mut p = AccessPattern::new("test1".to_string());
                p.frequency = 5.0;
                p
            },
            {
                let mut p = AccessPattern::new("test2".to_string());
                p.frequency = 3.0;
                p
            },
        ];

        let _ = predictor.get_predictions(&patterns).await.unwrap();

        assert_eq!(predictor.metrics().prediction_requests, 1);
        assert!(predictor.metrics().predictions_made > 0);
    }

    // ============================================================================
    // Performance Tests
    // ============================================================================

    #[tokio::test]
    async fn test_predictor_large_dataset() {
        let predictor = Predictor::new();

        let patterns: Vec<AccessPattern> = (0..10000)
            .map(|i| {
                let mut p = AccessPattern::new(format!("key_{}", i));
                p.frequency = (i % 100) as f64;
                p
            })
            .collect();

        let start = std::time::Instant::now();
        let predictions = predictor.get_predictions(&patterns).await.unwrap();
        let duration = start.elapsed();

        // Should complete in reasonable time (< 100ms for 10k items)
        assert!(duration.as_millis() < 100);
        assert!(!predictions.is_empty());
    }

    #[tokio::test]
    async fn test_predictor_concurrent_predictions() {
        let predictor = Predictor::new();

        let patterns: Vec<AccessPattern> = (0..100)
            .map(|i| {
                let mut p = AccessPattern::new(format!("key_{}", i));
                p.frequency = i as f64;
                p
            })
            .collect();

        let start = std::time::Instant::now();

        // Spawn multiple concurrent prediction tasks
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let predictor = &predictor;
                let patterns = &patterns;
                tokio::spawn(async move {
                    predictor.get_predictions(patterns).await
                })
            })
            .collect();

        // Wait for all to complete
        for handle in handles {
            let _ = handle.await.unwrap().unwrap();
        }

        let duration = start.elapsed();

        // Concurrent operations should still be fast
        assert!(duration.as_millis() < 200);
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[tokio::test]
    async fn test_predictor_empty_patterns() {
        let predictor = Predictor::new();
        let patterns = vec![];

        let predictions = predictor.get_predictions(&patterns).await.unwrap();

        assert!(predictions.is_empty());
    }

    #[tokio::test]
    async fn test_predictor_single_pattern() {
        let predictor = Predictor::new();

        let mut pattern = AccessPattern::new("single".to_string());
        pattern.frequency = 1.0;

        let predictions = predictor.get_predictions(&vec![pattern]).await.unwrap();

        // Should handle single item
        assert!(!predictions.is_empty());
    }

    #[tokio::test]
    async fn test_frequency_predictor_zero_frequency() {
        let predictor = FrequencyPredictor::new(0.0, 100);

        let patterns = vec![AccessPattern::new("zero_freq".to_string())];
        let result = predictor.train(&patterns).await;

        assert!(result.is_ok());
    }

    #[cfg(feature = "predictive")]
    #[tokio::test]
    async fn test_ml_predictor_insufficient_data() {
        let predictor = MlPredictor::new(100);

        let patterns = vec![AccessPattern::new("test".to_string())];
        let result = predictor.train(&patterns).await;

        assert!(result.is_err());
    }
}
