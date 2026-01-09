//! Error types for cache-layer-optimizer

use thiserror::Error;

/// Error types for optimizer operations
#[derive(Error, Debug)]
pub enum Error {
    /// Cache-layer error
    #[error("Cache error: {0}")]
    Cache(#[from] cache_layer::Error),

    /// Prediction model error
    #[error("Prediction error: {0}")]
    Prediction(String),

    /// Insufficient data for training
    #[error("Insufficient training data: need at least {0} samples, got {1}")]
    InsufficientData { required: usize, actual: usize },

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Tier sizing error
    #[error("Tier sizing error: {0}")]
    TierSizing(String),

    /// Coherence protocol error
    #[error("Coherence error: {0}")]
    Coherence(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Generic error
    #[error("{0}")]
    Generic(String),
}

/// Result type for optimizer operations
pub type Result<T> = std::result::Result<T, Error>;
