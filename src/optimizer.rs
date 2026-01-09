//! Main optimizer orchestrator

use crate::coherence::{CoherenceProtocol, WriteThrough};
use crate::error::Result;
use crate::metrics::OptimizerMetrics;
use crate::predictive::{AccessPattern, Predictor};
use crate::tiering::{TierConfig, TierManager, TierSizing};
use crate::warming::Warmer;
use serde::{Deserialize, Serialize};

/// Optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub enable_prediction: bool,
    pub enable_warming: bool,
    pub enable_tiering: bool,
    pub enable_coherence: bool,
    pub optimization_interval_secs: u64,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            enable_prediction: true,
            enable_warming: true,
            enable_tiering: true,
            enable_coherence: true,
            optimization_interval_secs: 60,
        }
    }
}

/// Optimization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub hit_rate_improvement: f64,
    pub latency_reduction: f64,
    pub memory_efficiency: f64,
    pub keys_optimized: usize,
    pub duration_ms: u64,
}

/// Main cache optimizer
pub struct CacheOptimizer {
    config: OptimizationConfig,
    predictor: Predictor,
    warmer: Option<Warmer>,
    tier_manager: Option<TierManager>,
    coherence: Option<Box<dyn CoherenceProtocol>>,
    metrics: OptimizerMetrics,
}

impl CacheOptimizer {
    pub fn new(config: OptimizationConfig) -> Self {
        Self {
            config,
            predictor: Predictor::new(),
            warmer: None,
            tier_manager: None,
            coherence: None,
            metrics: OptimizerMetrics::new(),
        }
    }

    pub fn with_warming(mut self, warmer: Warmer) -> Self {
        self.warmer = Some(warmer);
        self
    }

    pub fn with_tiering(mut self, tier_manager: TierManager) -> Self {
        self.tier_manager = Some(tier_manager);
        self
    }

    pub fn with_coherence(mut self, protocol: Box<dyn CoherenceProtocol>) -> Self {
        self.coherence = Some(protocol);
        self
    }

    /// Run optimization cycle
    pub async fn optimize(&mut self, patterns: &[AccessPattern]) -> Result<OptimizationResult> {
        let start = std::time::Instant::now();

        // 1. Predictive caching
        if self.config.enable_prediction {
            let _predicted_keys = self.predictor.get_predictions(patterns).await?;
        }

        // 2. Cache warming
        let mut keys_warmed = 0;
        if let Some(warmer) = &mut self.warmer {
            let keys = warmer.warm(patterns).await?;
            keys_warmed = keys.len();
        }

        // 3. Dynamic tiering
        if let Some(tier_manager) = &mut self.tier_manager {
            tier_manager.optimize_tiers().await?;
        }

        let duration = start.elapsed();

        Ok(OptimizationResult {
            hit_rate_improvement: self.metrics.tiering.hit_rate_improvement,
            latency_reduction: 0.0, // Calculate based on actual metrics
            memory_efficiency: self.metrics.tiering.memory_efficiency,
            keys_optimized: keys_warmed,
            duration_ms: duration.as_millis() as u64,
        })
    }

    pub fn metrics(&self) -> &OptimizerMetrics {
        &self.metrics
    }
}

/// Simplified optimizer interface
pub struct Optimizer {
    inner: CacheOptimizer,
}

impl Optimizer {
    pub fn new(config: OptimizationConfig) -> Self {
        Self {
            inner: CacheOptimizer::new(config),
        }
    }

    pub fn with_defaults() -> Result<Self> {
        let config = OptimizationConfig::default();
        let mut optimizer = CacheOptimizer::new(config);

        // Add default coherence protocol
        optimizer.coherence = Some(Box::new(WriteThrough::new()));

        Ok(Self { inner: optimizer })
    }

    pub async fn optimize(&mut self, patterns: &[AccessPattern]) -> Result<OptimizationResult> {
        self.inner.optimize(patterns).await
    }

    pub fn metrics(&self) -> &OptimizerMetrics {
        self.inner.metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_config() {
        let config = OptimizationConfig::default();
        assert!(config.enable_prediction);
        assert!(config.enable_warming);
    }

    #[tokio::test]
    async fn test_optimizer() {
        let mut optimizer = Optimizer::with_defaults().unwrap();
        let patterns = vec![AccessPattern::new("test".to_string())];

        let result = optimizer.optimize(&patterns).await.unwrap();
        assert_eq!(result.keys_optimized, 0); // No warmer configured
    }
}
