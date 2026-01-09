//! Metrics for optimizer components

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Prediction model metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionMetrics {
    pub training_runs: usize,
    pub training_samples: usize,
    pub prediction_requests: usize,
    pub predictions_made: usize,
    pub prediction_accuracy: f64,
}

impl PredictionMetrics {
    pub fn new() -> Self {
        Self {
            training_runs: 0,
            training_samples: 0,
            prediction_requests: 0,
            predictions_made: 0,
            prediction_accuracy: 0.0,
        }
    }
}

impl Default for PredictionMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache warming metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmingMetrics {
    pub warming_cycles: usize,
    pub keys_warmed: usize,
    pub warming_success_rate: f64,
    pub avg_warming_time: Duration,
    pub warming_hit_rate: f64,
}

impl WarmingMetrics {
    pub fn new() -> Self {
        Self {
            warming_cycles: 0,
            keys_warmed: 0,
            warming_success_rate: 0.0,
            avg_warming_time: Duration::ZERO,
            warming_hit_rate: 0.0,
        }
    }
}

impl Default for WarmingMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Tiering metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TieringMetrics {
    pub sizing_adjustments: usize,
    pub total_bytes_moved: u64,
    pub avg_adjustment_time: Duration,
    pub hit_rate_improvement: f64,
    pub memory_efficiency: f64,
}

impl TieringMetrics {
    pub fn new() -> Self {
        Self {
            sizing_adjustments: 0,
            total_bytes_moved: 0,
            avg_adjustment_time: Duration::ZERO,
            hit_rate_improvement: 0.0,
            memory_efficiency: 0.0,
        }
    }
}

impl Default for TieringMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Coherence protocol metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceMetrics {
    pub invalidations: usize,
    pub coherent_writes: usize,
    pub incoherent_reads: usize,
    pub avg_invalidation_time: Duration,
    pub coherence_overhead: f64,
}

impl CoherenceMetrics {
    pub fn new() -> Self {
        Self {
            invalidations: 0,
            coherent_writes: 0,
            incoherent_reads: 0,
            avg_invalidation_time: Duration::ZERO,
            coherence_overhead: 0.0,
        }
    }
}

impl Default for CoherenceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined optimizer metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerMetrics {
    pub prediction: PredictionMetrics,
    pub warming: WarmingMetrics,
    pub tiering: TieringMetrics,
    pub coherence: CoherenceMetrics,
}

impl OptimizerMetrics {
    pub fn new() -> Self {
        Self {
            prediction: PredictionMetrics::new(),
            warming: WarmingMetrics::new(),
            tiering: TieringMetrics::new(),
            coherence: CoherenceMetrics::new(),
        }
    }

    /// Get overall efficiency score (0.0 - 1.0)
    pub fn efficiency_score(&self) -> f64 {
        let prediction_score = self.prediction.prediction_accuracy;
        let warming_score = self.warming.warming_hit_rate;
        let tiering_score = self.tiering.hit_rate_improvement;
        let coherence_score = 1.0 - self.coherence.coherence_overhead;

        (prediction_score + warming_score + tiering_score + coherence_score) / 4.0
    }
}

impl Default for OptimizerMetrics {
    fn default() -> Self {
        Self::new()
    }
}
