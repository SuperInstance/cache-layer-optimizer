// cache-layer-optimizer: Advanced caching strategies and optimizations
//
// This library extends cache-layer with:
// - Predictive caching (ML-based)
// - Cache warming strategies
// - Dynamic tier sizing
// - Cache coherence protocols
// - Performance optimization

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, unused_imports)]

mod predictive;
mod warming;
mod tiering;
mod coherence;
mod optimizer;
mod metrics;
mod error;

pub use error::{Error, Result};

pub use predictive::{
    Predictor, PredictionModel, AccessPattern, FrequencyPredictor,
    RecencyPredictor, MlPredictor,
};

pub use warming::{
    Warmer, WarmingStrategy, OnDemandWarmer, ProactiveWarmer,
    ScheduledWarmer, WarmingConfig,
};

pub use tiering::{
    TierManager, TierSizing, AdaptiveTiering, FixedTiering,
    DynamicTiering, TierConfig,
};

pub use coherence::{
    CoherenceProtocol, InvalidationStrategy, WriteThrough,
    WriteBack, WriteAround,
};

pub use optimizer::{
    Optimizer, OptimizationConfig, OptimizationResult,
    CacheOptimizer,
};

pub use metrics::{
    OptimizerMetrics, PredictionMetrics, WarmingMetrics,
    TieringMetrics, CoherenceMetrics,
};

/// Re-export cache-layer for convenience
pub use cache_layer;

/// Pre-configured optimizer with recommended defaults
pub fn default_optimizer() -> Result<Optimizer> {
    Optimizer::with_defaults()
}
