//! Cache warming module
//!
//! Provides strategies for proactive cache warming.

use crate::error::Result;
use crate::metrics::WarmingMetrics;
use crate::predictive::{AccessPattern, Predictor};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Cache warming strategy
#[async_trait]
pub trait WarmingStrategy: Send + Sync {
    /// Select keys to warm
    async fn select_keys(&self, patterns: &[AccessPattern]) -> Result<Vec<String>>;

    /// Get warming metrics
    fn metrics(&self) -> &WarmingMetrics;
}

/// On-demand warming (warm when requested)
#[derive(Debug)]
pub struct OnDemandWarmer {
    metrics: WarmingMetrics,
    max_keys: usize,
}

impl OnDemandWarmer {
    pub fn new(max_keys: usize) -> Self {
        Self {
            metrics: WarmingMetrics::new(),
            max_keys,
        }
    }
}

#[async_trait]
impl WarmingStrategy for OnDemandWarmer {
    async fn select_keys(&self, patterns: &[AccessPattern]) -> Result<Vec<String>> {
        let mut sorted: Vec<_> = patterns
            .iter()
            .map(|p| (p.key.clone(), p.priority_score()))
            .collect();

        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        Ok(sorted
            .into_iter()
            .take(self.max_keys)
            .map(|(k, _)| k)
            .collect())
    }

    fn metrics(&self) -> &WarmingMetrics {
        &self.metrics
    }
}

/// Proactive warming (warm based on predictions)
#[derive(Debug)]
pub struct ProactiveWarmer {
    metrics: WarmingMetrics,
    predictor: Predictor,
    interval: Duration,
}

impl ProactiveWarmer {
    pub fn new(predictor: Predictor, interval: Duration) -> Self {
        Self {
            metrics: WarmingMetrics::new(),
            predictor,
            interval,
        }
    }
}

#[async_trait]
impl WarmingStrategy for ProactiveWarmer {
    async fn select_keys(&self, patterns: &[AccessPattern]) -> Result<Vec<String>> {
        self.predictor.get_predictions(patterns).await
    }

    fn metrics(&self) -> &WarmingMetrics {
        &self.metrics
    }
}

/// Scheduled warming
#[derive(Debug, Clone)]
pub struct ScheduledWarmer {
    metrics: WarmingMetrics,
    schedule: Vec<WarmingSchedule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmingSchedule {
    pub time: String, // HH:MM format
    pub keys: Vec<String>,
}

impl ScheduledWarmer {
    pub fn new(schedule: Vec<WarmingSchedule>) -> Self {
        Self {
            metrics: WarmingMetrics::new(),
            schedule,
        }
    }
}

#[async_trait]
impl WarmingStrategy for ScheduledWarmer {
    async fn select_keys(&self, _patterns: &[AccessPattern]) -> Result<Vec<String>> {
        // Return keys for current time
        // This is a simplified version
        Ok(Vec::new())
    }

    fn metrics(&self) -> &WarmingMetrics {
        &self.metrics
    }
}

/// Warming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmingConfig {
    pub max_keys: usize,
    pub interval: Duration,
    pub strategy: WarmingStrategyType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarmingStrategyType {
    OnDemand,
    Proactive,
    Scheduled,
}

/// Main cache warmer
#[derive(Debug)]
pub struct Warmer {
    strategy: Box<dyn WarmingStrategy>,
    metrics: WarmingMetrics,
}

impl Warmer {
    pub fn new(strategy: Box<dyn WarmingStrategy>) -> Self {
        Self {
            strategy,
            metrics: WarmingMetrics::new(),
        }
    }

    /// Warm the cache with selected keys
    pub async fn warm(&mut self, patterns: &[AccessPattern]) -> Result<Vec<String>> {
        let start = std::time::Instant::now();

        self.metrics.warming_cycles += 1;
        let keys = self.strategy.select_keys(patterns).await?;

        self.metrics.keys_warmed += keys.len();
        self.metrics.avg_warming_time = start.elapsed();

        Ok(keys)
    }

    pub fn metrics(&self) -> &WarmingMetrics {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_on_demand_warmer() {
        let warmer = OnDemandWarmer::new(10);
        assert_eq!(warmer.metrics.warming_cycles, 0);
    }
}
