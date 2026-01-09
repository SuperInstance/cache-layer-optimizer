//! Cache coherence protocols

use crate::error::{Error, Result};
use crate::metrics::CoherenceMetrics;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Coherence protocol
#[async_trait]
pub trait CoherenceProtocol: Send + Sync {
    /// Invalidate a key across all tiers
    async fn invalidate(&mut self, key: &str) -> Result<()>;

    /// Update a key across all tiers
    async fn update(&mut self, key: &str, value: Vec<u8>) -> Result<()>;

    /// Get coherence metrics
    fn metrics(&self) -> &CoherenceMetrics;
}

/// Write-through protocol
#[derive(Debug)]
pub struct WriteThrough {
    metrics: CoherenceMetrics,
}

impl WriteThrough {
    pub fn new() -> Self {
        Self {
            metrics: CoherenceMetrics::new(),
        }
    }
}

#[async_trait]
impl CoherenceProtocol for WriteThrough {
    async fn invalidate(&mut self, key: &str) -> Result<()> {
        let start = std::time::Instant::now();
        self.metrics.invalidations += 1;
        self.metrics.avg_invalidation_time = start.elapsed();
        Ok(())
    }

    async fn update(&mut self, key: &str, value: Vec<u8>) -> Result<()> {
        self.metrics.coherent_writes += 1;
        Ok(())
    }

    fn metrics(&self) -> &CoherenceMetrics {
        &self.metrics
    }
}

/// Write-back protocol
#[derive(Debug)]
pub struct WriteBack {
    metrics: CoherenceMetrics,
    dirty: Vec<String>,
}

impl WriteBack {
    pub fn new() -> Self {
        Self {
            metrics: CoherenceMetrics::new(),
            dirty: Vec::new(),
        }
    }
}

#[async_trait]
impl CoherenceProtocol for WriteBack {
    async fn invalidate(&mut self, key: &str) -> Result<()> {
        let start = std::time::Instant::now();
        self.metrics.invalidations += 1;
        self.metrics.avg_invalidation_time = start.elapsed();
        Ok(())
    }

    async fn update(&mut self, key: &str, value: Vec<u8>) -> Result<()> {
        self.metrics.coherent_writes += 1;
        self.dirty.push(key.to_string());
        Ok(())
    }

    fn metrics(&self) -> &CoherenceMetrics {
        &self.metrics
    }
}

/// Write-around protocol
#[derive(Debug)]
pub struct WriteAround {
    metrics: CoherenceMetrics,
}

impl WriteAround {
    pub fn new() -> Self {
        Self {
            metrics: CoherenceMetrics::new(),
        }
    }
}

#[async_trait]
impl CoherenceProtocol for WriteAround {
    async fn invalidate(&mut self, key: &str) -> Result<()> {
        let start = std::time::Instant::now();
        self.metrics.invalidations += 1;
        self.metrics.avg_invalidation_time = start.elapsed();
        Ok(())
    }

    async fn update(&mut self, key: &str, value: Vec<u8>) -> Result<()> {
        self.metrics.coherent_writes += 1;
        Ok(())
    }

    fn metrics(&self) -> &CoherenceMetrics {
        &self.metrics
    }
}

/// Invalidation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvalidationStrategy {
    Immediate,
    Lazy,
    Periodic,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_write_through() {
        let mut protocol = WriteThrough::new();
        protocol.invalidate("key1").await.unwrap();
        assert_eq!(protocol.metrics().invalidations, 1);
    }
}
