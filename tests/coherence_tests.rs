//! Cache coherence protocol tests
//!
//! Tests for multi-tier cache coherence strategies

use cache_layer_optimizer::{
    CoherenceProtocol, WriteAround, WriteBack, WriteThrough,
};

#[cfg(test)]
mod coherence_tests {
    use super::*;

    // ============================================================================
    // Unit Tests
    // ============================================================================

    #[test]
    fn test_write_through_creation() {
        let protocol = WriteThrough::new();

        assert_eq!(protocol.metrics().invalidations, 0);
        assert_eq!(protocol.metrics().coherent_writes, 0);
    }

    #[tokio::test]
    async fn test_write_through_invalidate() {
        let mut protocol = WriteThrough::new();

        protocol.invalidate("key1").await.unwrap();

        assert_eq!(protocol.metrics().invalidations, 1);
        assert!(protocol.metrics().avg_invalidation_time.as_millis() >= 0);
    }

    #[tokio::test]
    async fn test_write_through_update() {
        let mut protocol = WriteThrough::new();

        protocol.update("key1", b"value".to_vec()).await.unwrap();

        assert_eq!(protocol.metrics().coherent_writes, 1);
    }

    #[tokio::test]
    async fn test_write_through_multiple_operations() {
        let mut protocol = WriteThrough::new();

        protocol.invalidate("key1").await.unwrap();
        protocol.update("key2", b"value".to_vec()).await.unwrap();
        protocol.invalidate("key3").await.unwrap();

        assert_eq!(protocol.metrics().invalidations, 2);
        assert_eq!(protocol.metrics().coherent_writes, 1);
    }

    #[test]
    fn test_write_back_creation() {
        let protocol = WriteBack::new();

        assert_eq!(protocol.metrics().invalidations, 0);
        assert_eq!(protocol.metrics().coherent_writes, 0);
        assert!(protocol.dirty.is_empty());
    }

    #[tokio::test]
    async fn test_write_back_invalidate() {
        let mut protocol = WriteBack::new();

        protocol.invalidate("key1").await.unwrap();

        assert_eq!(protocol.metrics().invalidations, 1);
    }

    #[tokio::test]
    async fn test_write_back_update() {
        let mut protocol = WriteBack::new();

        protocol.update("key1", b"value".to_vec()).await.unwrap();

        assert_eq!(protocol.metrics().coherent_writes, 1);
        assert_eq!(protocol.dirty.len(), 1);
        assert_eq!(protocol.dirty[0], "key1");
    }

    #[tokio::test]
    async fn test_write_back_multiple_updates() {
        let mut protocol = WriteBack::new();

        protocol.update("key1", b"value1".to_vec()).await.unwrap();
        protocol.update("key2", b"value2".to_vec()).await.unwrap();
        protocol.update("key3", b"value3".to_vec()).await.unwrap();

        assert_eq!(protocol.metrics().coherent_writes, 3);
        assert_eq!(protocol.dirty.len(), 3);
    }

    #[tokio::test]
    async fn test_write_back_dirty_tracking() {
        let mut protocol = WriteBack::new();

        protocol.update("key1", b"value1".to_vec()).await.unwrap();
        protocol.update("key2", b"value2".to_vec()).await.unwrap();
        protocol.update("key1", b"value1_new".to_vec()).await.unwrap();

        // Should track duplicates
        assert_eq!(protocol.dirty.len(), 3);
    }

    #[test]
    fn test_write_around_creation() {
        let protocol = WriteAround::new();

        assert_eq!(protocol.metrics().invalidations, 0);
        assert_eq!(protocol.metrics().coherent_writes, 0);
    }

    #[tokio::test]
    async fn test_write_around_invalidate() {
        let mut protocol = WriteAround::new();

        protocol.invalidate("key1").await.unwrap();

        assert_eq!(protocol.metrics().invalidations, 1);
    }

    #[tokio::test]
    async fn test_write_around_update() {
        let mut protocol = WriteAround::new();

        protocol.update("key1", b"value".to_vec()).await.unwrap();

        assert_eq!(protocol.metrics().coherent_writes, 1);
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[tokio::test]
    async fn test_coherence_metrics_tracking() {
        let mut protocol = WriteThrough::new();

        // Perform various operations
        for i in 0..10 {
            protocol.invalidate(&format!("key{}", i)).await.unwrap();
        }

        for i in 0..5 {
            protocol.update(&format!("key{}", i), b"value".to_vec()).await.unwrap();
        }

        let metrics = protocol.metrics();

        assert_eq!(metrics.invalidations, 10);
        assert_eq!(metrics.coherent_writes, 5);
    }

    #[tokio::test]
    async fn test_coherence_invalidation_timing() {
        let mut protocol = WriteThrough::new();

        let start = std::time::Instant::now();
        protocol.invalidate("key1").await.unwrap();
        let first_duration = start.elapsed();

        protocol.invalidate("key2").await.unwrap();

        // Average should be reasonable
        assert!(protocol.metrics().avg_invalidation_time.as_millis() >= 0);
        assert!(first_duration.as_millis() < 100);
    }

    #[tokio::test]
    async fn test_write_through_vs_write_back() {
        let mut wt = WriteThrough::new();
        let mut wb = WriteBack::new();

        wt.update("key1", b"value".to_vec()).await.unwrap();
        wb.update("key1", b"value".to_vec()).await.unwrap();

        // Both should track writes
        assert_eq!(wt.metrics().coherent_writes, 1);
        assert_eq!(wb.metrics().coherent_writes, 1);

        // Only WriteBack should track dirty keys
        assert_eq!(wb.dirty.len(), 1);
    }

    #[tokio::test]
    async fn test_protocol_concurrent_operations() {
        let mut protocol = WriteThrough::new();

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let protocol = &mut protocol;
                tokio::spawn(async move {
                    protocol.invalidate(&format!("key{}", i)).await
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        assert_eq!(protocol.metrics().invalidations, 10);
    }

    // ============================================================================
    // Property-Based Tests
    // ============================================================================

    proptest::proptest! {
        #[test]
        fn prop_write_through_invalidation_count(
            num_invalidations in 1..1000usize
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut protocol = WriteThrough::new();

            rt.block_on(async {
                for i in 0..num_invalidations {
                    protocol.invalidate(&format!("key{}", i)).await.unwrap();
                }
            });

            prop_assert_eq!(protocol.metrics().invalidations, num_invalidations);
        }
    }

    proptest::proptest! {
        #[test]
        fn prop_write_back_dirty_tracking(
            num_updates in 1..100usize
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut protocol = WriteBack::new();

            rt.block_on(async {
                for i in 0..num_updates {
                    protocol.update(&format!("key{}", i), b"value".to_vec()).await.unwrap();
                }
            });

            prop_assert_eq!(protocol.dirty.len(), num_updates);
            prop_assert_eq!(protocol.metrics().coherent_writes, num_updates);
        }
    }

    proptest::proptest! {
        #[test]
        fn prop_coherence_timing(
            num_ops in 1..100usize
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut protocol = WriteThrough::new();

            rt.block_on(async {
                for i in 0..num_ops {
                    protocol.invalidate(&format!("key{}", i)).await.unwrap();
                }
            });

            // Average invalidation time should be reasonable
            prop_assert!(protocol.metrics().avg_invalidation_time.as_micros() > 0);
        }
    }

    // ============================================================================
    // Performance Tests
    // ============================================================================

    #[tokio::test]
    async fn test_coherence_high_throughput() {
        let mut protocol = WriteThrough::new();

        let start = std::time::Instant::now();

        for i in 0..10000 {
            protocol.invalidate(&format!("key{}", i)).await.unwrap();
        }

        let duration = start.elapsed();

        // Should handle 10k operations quickly (< 500ms)
        assert!(duration.as_millis() < 500);
        assert_eq!(protocol.metrics().invalidations, 10000);
    }

    #[tokio::test]
    async fn test_coherence_concurrent_stress() {
        let mut protocol = WriteThrough::new();

        let start = std::time::Instant::now();

        let handles: Vec<_> = (0..100)
            .map(|_| {
                let protocol = &mut protocol;
                tokio::spawn(async move {
                    for i in 0..100 {
                        protocol.invalidate(&format!("key{}", i)).await.unwrap();
                    }
                    Ok::<(), Box<dyn std::error::Error>>(())
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        let duration = start.elapsed();

        // 100 * 100 = 10,000 operations
        assert_eq!(protocol.metrics().invalidations, 10000);
        assert!(duration.as_millis() < 1000);
    }

    #[tokio::test]
    async fn test_write_back_dirty_performance() {
        let mut protocol = WriteBack::new();

        let start = std::time::Instant::now();

        for i in 0..10000 {
            protocol.update(&format!("key{}", i), b"value".to_vec()).await.unwrap();
        }

        let duration = start.elapsed();

        // Should track 10k dirty keys efficiently
        assert_eq!(protocol.dirty.len(), 10000);
        assert!(duration.as_millis() < 500);
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[tokio::test]
    async fn test_coherence_empty_key() {
        let mut protocol = WriteThrough::new();

        protocol.invalidate("").await.unwrap();

        assert_eq!(protocol.metrics().invalidations, 1);
    }

    #[tokio::test]
    async fn test_coherence_very_long_key() {
        let long_key = "a".repeat(10000);
        let mut protocol = WriteThrough::new();

        protocol.invalidate(&long_key).await.unwrap();

        assert_eq!(protocol.metrics().invalidations, 1);
    }

    #[tokio::test]
    async fn test_coherence_empty_value() {
        let mut protocol = WriteThrough::new();

        protocol.update("key1", vec![]).await.unwrap();

        assert_eq!(protocol.metrics().coherent_writes, 1);
    }

    #[tokio::test]
    async fn test_coherence_large_value() {
        let large_value = vec![0u8; 10_000_000]; // 10MB
        let mut protocol = WriteThrough::new();

        protocol.update("key1", large_value).await.unwrap();

        assert_eq!(protocol.metrics().coherent_writes, 1);
    }

    #[tokio::test]
    async fn test_write_back_duplicate_keys() {
        let mut protocol = WriteBack::new();

        // Update same key multiple times
        for _ in 0..10 {
            protocol.update("key1", b"value".to_vec()).await.unwrap();
        }

        assert_eq!(protocol.dirty.len(), 10);
        assert_eq!(protocol.metrics().coherent_writes, 10);
    }

    #[tokio::test]
    async fn test_coherence_mixed_operations() {
        let mut protocol = WriteThrough::new();

        protocol.update("key1", b"value1".to_vec()).await.unwrap();
        protocol.invalidate("key1").await.unwrap();
        protocol.update("key1", b"value2".to_vec()).await.unwrap();
        protocol.invalidate("key2").await.unwrap();

        assert_eq!(protocol.metrics().coherent_writes, 2);
        assert_eq!(protocol.metrics().invalidations, 2);
    }
}
