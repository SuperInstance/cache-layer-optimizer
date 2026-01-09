//! Dynamic tier sizing module

use crate::error::{Error, Result};
use crate::metrics::TieringMetrics;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Cache tier configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    pub tier_name: String,
    pub max_size: u64,
    pub current_size: u64,
    pub target_size: u64,
    pub growth_factor: f64,
    pub shrink_factor: f64,
}

impl TierConfig {
    pub fn new(name: String, max_size: u64) -> Self {
        Self {
            tier_name: name,
            max_size,
            current_size: max_size / 2,
            target_size: max_size / 2,
            growth_factor: 1.5,
            shrink_factor: 0.75,
        }
    }

    pub fn utilization(&self) -> f64 {
        self.current_size as f64 / self.max_size as f64
    }
}

/// Tier sizing strategy
#[async_trait::async_trait]
pub trait TierSizing: Send + Sync {
    /// Calculate optimal tier sizes
    async fn calculate_sizes(&self, metrics: &TieringMetrics) -> Result<Vec<TierConfig>>;

    /// Adjust tier sizes
    async fn adjust_sizes(&mut self, configs: &[TierConfig]) -> Result<()>;
}

/// Fixed tier sizing (no changes)
#[derive(Debug, Clone)]
pub struct FixedTiering {
    configs: Vec<TierConfig>,
}

impl FixedTiering {
    pub fn new(configs: Vec<TierConfig>) -> Self {
        Self { configs }
    }
}

#[async_trait::async_trait]
impl TierSizing for FixedTiering {
    async fn calculate_sizes(&self, _metrics: &TieringMetrics) -> Result<Vec<TierConfig>> {
        Ok(self.configs.clone())
    }

    async fn adjust_sizes(&mut self, _configs: &[TierConfig]) -> Result<()> {
        Ok(())
    }
}

/// Adaptive tier sizing (based on usage)
#[derive(Debug)]
pub struct AdaptiveTiering {
    configs: Vec<TierConfig>,
    metrics: TieringMetrics,
    adjustment_interval: Duration,
    hit_rate_threshold: f64,
}

impl AdaptiveTiering {
    pub fn new(configs: Vec<TierConfig>, adjustment_interval: Duration) -> Self {
        Self {
            configs,
            metrics: TieringMetrics::new(),
            adjustment_interval,
            hit_rate_threshold: 0.8,
        }
    }

    /// Grow a tier based on demand
    pub fn grow_tier(&mut self, tier_idx: usize) -> Result<()> {
        if tier_idx >= self.configs.len() {
            return Err(Error::TierSizing("Invalid tier index".to_string()));
        }

        let config = &mut self.configs[tier_idx];
        let new_size = (config.current_size as f64 * config.growth_factor) as u64;

        if new_size <= config.max_size {
            config.current_size = new_size;
            config.target_size = new_size;
        }

        Ok(())
    }

    /// Shrink a tier based on low utilization
    pub fn shrink_tier(&mut self, tier_idx: usize) -> Result<()> {
        if tier_idx >= self.configs.len() {
            return Err(Error::TierSizing("Invalid tier index".to_string()));
        }

        let config = &mut self.configs[tier_idx];
        let new_size = (config.current_size as f64 * config.shrink_factor) as u64;
        let min_size = config.max_size / 10; // Minimum 10% of max

        config.current_size = new_size.max(min_size);
        config.target_size = config.current_size;

        Ok(())
    }
}

#[async_trait::async_trait]
impl TierSizing for AdaptiveTiering {
    async fn calculate_sizes(&self, _metrics: &TieringMetrics) -> Result<Vec<TierConfig>> {
        Ok(self.configs.clone())
    }

    async fn adjust_sizes(&mut self, configs: &[TierConfig]) -> Result<()> {
        self.metrics.sizing_adjustments += 1;
        self.configs = configs.to_vec();
        Ok(())
    }
}

/// Dynamic tier sizing (ML-based)
#[derive(Debug)]
pub struct DynamicTiering {
    configs: Vec<TierConfig>,
    metrics: TieringMetrics,
}

impl DynamicTiering {
    pub fn new(configs: Vec<TierConfig>) -> Self {
        Self {
            configs,
            metrics: TieringMetrics::new(),
        }
    }
}

#[async_trait::async_trait]
impl TierSizing for DynamicTiering {
    async fn calculate_sizes(&self, _metrics: &TieringMetrics) -> Result<Vec<TierConfig>> {
        Ok(self.configs.clone())
    }

    async fn adjust_sizes(&mut self, configs: &[TierConfig]) -> Result<()> {
        self.metrics.sizing_adjustments += 1;
        self.configs = configs.to_vec();
        Ok(())
    }
}

/// Tier manager
#[derive(Debug)]
pub struct TierManager {
    strategy: Box<dyn TierSizing>,
    metrics: TieringMetrics,
}

impl TierManager {
    pub fn new(strategy: Box<dyn TierSizing>) -> Self {
        Self {
            strategy,
            metrics: TieringMetrics::new(),
        }
    }

    /// Optimize tier sizes
    pub async fn optimize_tiers(&mut self) -> Result<Vec<TierConfig>> {
        let start = std::time::Instant::now();

        let configs = self.strategy.calculate_sizes(&self.metrics).await?;
        self.strategy.adjust_sizes(&configs).await?;

        self.metrics.avg_adjustment_time = start.elapsed();
        Ok(configs)
    }

    pub fn metrics(&self) -> &TieringMetrics {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_config() {
        let config = TierConfig::new("l1".to_string(), 1024);
        assert_eq!(config.max_size, 1024);
        assert_eq!(config.current_size, 512);
    }

    #[test]
    fn test_adaptive_tiering() {
        let configs = vec![TierConfig::new("l1".to_string(), 1024)];
        let mut tiering = AdaptiveTiering::new(configs, Duration::from_secs(60));

        tiering.grow_tier(0).unwrap();
        assert_eq!(tiering.configs[0].current_size, 768);

        tiering.shrink_tier(0).unwrap();
        assert_eq!(tiering.configs[0].current_size, 576);
    }
}
