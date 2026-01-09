//! Predictive caching module
//!
//! Provides ML-based prediction of cache access patterns for proactive caching.

use crate::error::{Error, Result};
use crate::metrics::PredictionMetrics;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Access pattern for a cache key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPattern {
    /// Cache key
    pub key: String,

    /// Access frequency (accesses per second)
    pub frequency: f64,

    /// Last access timestamp
    pub last_access: DateTime<Utc>,

    /// Time since last access
    pub recency: Duration,

    /// Access pattern regularity (0.0 = random, 1.0 = perfectly regular)
    pub regularity: f64,

    /// Predicted next access time
    pub predicted_next_access: Option<DateTime<Utc>>,
}

impl AccessPattern {
    /// Create a new access pattern
    pub fn new(key: String) -> Self {
        Self {
            key,
            frequency: 0.0,
            last_access: Utc::now(),
            recency: Duration::ZERO,
            regularity: 0.0,
            predicted_next_access: None,
        }
    }

    /// Calculate priority score (higher = more likely to be accessed)
    pub fn priority_score(&self) -> f64 {
        let frequency_weight = 0.5;
        let recency_weight = 0.3;
        let regularity_weight = 0.2;

        let recency_score = 1.0 / (1.0 + self.recency.as_secs_f64());
        let regularity_score = self.regularity;

        (self.frequency * frequency_weight)
            + (recency_score * recency_weight)
            + (regularity_score * regularity_weight)
    }
}

/// Prediction model for cache access
#[async_trait]
pub trait PredictionModel: Send + Sync {
    /// Train the model on access history
    async fn train(&mut self, history: &[AccessPattern]) -> Result<()>;

    /// Predict next access for a key
    async fn predict(&self, key: &str) -> Result<Option<DateTime<Utc>>>;

    /// Predict multiple keys
    async fn predict_batch(&self, keys: &[String]) -> Result<HashMap<String, DateTime<Utc>>>;

    /// Get model metrics
    fn metrics(&self) -> &PredictionMetrics;
}

/// Frequency-based predictor (simple heuristic)
#[derive(Debug, Clone)]
pub struct FrequencyPredictor {
    metrics: PredictionMetrics,
    min_frequency: f64,
    window_size: usize,
}

impl FrequencyPredictor {
    /// Create a new frequency predictor
    pub fn new(min_frequency: f64, window_size: usize) -> Self {
        Self {
            metrics: PredictionMetrics::new(),
            min_frequency,
            window_size,
        }
    }

    /// Get high-frequency keys
    pub fn get_high_frequency_keys(&self, patterns: &[AccessPattern]) -> Vec<String> {
        patterns
            .iter()
            .filter(|p| p.frequency >= self.min_frequency)
            .map(|p| p.key.clone())
            .collect()
    }
}

#[async_trait]
impl PredictionModel for FrequencyPredictor {
    async fn train(&mut self, history: &[AccessPattern]) -> Result<()> {
        self.metrics.training_runs += 1;
        self.metrics.training_samples += history.len();
        Ok(())
    }

    async fn predict(&self, key: &str) -> Result<Option<DateTime<Utc>>> {
        Ok(None) // Frequency predictor doesn't predict exact times
    }

    async fn predict_batch(&self, keys: &[String]) -> Result<HashMap<String, DateTime<Utc>>> {
        Ok(HashMap::new())
    }

    fn metrics(&self) -> &PredictionMetrics {
        &self.metrics
    }
}

/// Recency-based predictor
#[derive(Debug, Clone)]
pub struct RecencyPredictor {
    metrics: PredictionMetrics,
    recency_threshold: Duration,
}

impl RecencyPredictor {
    /// Create a new recency predictor
    pub fn new(recency_threshold: Duration) -> Self {
        Self {
            metrics: PredictionMetrics::new(),
            recency_threshold,
        }
    }

    /// Get recently accessed keys
    pub fn get_recent_keys(&self, patterns: &[AccessPattern]) -> Vec<String> {
        let now = Utc::now();
        patterns
            .iter()
            .filter(|p| {
                now.signed_duration_since(p.last_access).to_std().unwrap_or(Duration::ZERO)
                    < self.recency_threshold
            })
            .map(|p| p.key.clone())
            .collect()
    }
}

#[async_trait]
impl PredictionModel for RecencyPredictor {
    async fn train(&mut self, history: &[AccessPattern]) -> Result<()> {
        self.metrics.training_runs += 1;
        self.metrics.training_samples += history.len();
        Ok(())
    }

    async fn predict(&self, key: &str) -> Result<Option<DateTime<Utc>>> {
        Ok(None)
    }

    async fn predict_batch(&self, keys: &[String]) -> Result<HashMap<String, DateTime<Utc>>> {
        Ok(HashMap::new())
    }

    fn metrics(&self) -> &PredictionMetrics {
        &self.metrics
    }
}

/// ML-based predictor (requires "predictive" feature)
#[cfg(feature = "predictive")]
pub struct MlPredictor {
    metrics: PredictionMetrics,
    model: Option<linfa_linear_regression::LinearRegression>,
    min_samples: usize,
}

#[cfg(feature = "predictive")]
impl MlPredictor {
    /// Create a new ML predictor
    pub fn new(min_samples: usize) -> Self {
        Self {
            metrics: PredictionMetrics::new(),
            model: None,
            min_samples,
        }
    }
}

#[cfg(feature = "predictive")]
#[async_trait]
impl PredictionModel for MlPredictor {
    async fn train(&mut self, history: &[AccessPattern]) -> Result<()> {
        if history.len() < self.min_samples {
            return Err(Error::InsufficientData {
                required: self.min_samples,
                actual: history.len(),
            });
        }

        self.metrics.training_runs += 1;
        self.metrics.training_samples += history.len();

        // TODO: Implement actual ML training
        // This is a placeholder for the ML model training logic

        Ok(())
    }

    async fn predict(&self, key: &str) -> Result<Option<DateTime<Utc>>> {
        Ok(None)
    }

    async fn predict_batch(&self, keys: &[String]) -> Result<HashMap<String, DateTime<Utc>>> {
        Ok(HashMap::new())
    }

    fn metrics(&self) -> &PredictionMetrics {
        &self.metrics
    }
}

/// Main predictor that combines multiple strategies
#[derive(Debug)]
pub struct Predictor {
    models: Vec<Box<dyn PredictionModel>>,
    metrics: PredictionMetrics,
    enabled: bool,
}

impl Predictor {
    /// Create a new predictor
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            metrics: PredictionMetrics::new(),
            enabled: true,
        }
    }

    /// Add a prediction model
    pub fn add_model(mut self, model: Box<dyn PredictionModel>) -> Self {
        self.models.push(model);
        self
    }

    /// Enable/disable predictions
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get predictions for warming
    pub async fn get_predictions(&self, patterns: &[AccessPattern]) -> Result<Vec<String>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        self.metrics.prediction_requests += 1;

        // Combine predictions from all models
        let mut keys = Vec::new();
        for model in &self.models {
            // For now, return high-priority keys based on priority score
            let mut scored: Vec<_> = patterns
                .iter()
                .map(|p| (p.key.clone(), p.priority_score()))
                .collect();

            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            keys.extend(scored.into_iter().take(100).map(|(k, _)| k));
        }

        self.metrics.predictions_made += keys.len();
        Ok(keys)
    }

    /// Get predictor metrics
    pub fn metrics(&self) -> &PredictionMetrics {
        &self.metrics
    }
}

impl Default for Predictor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_pattern_priority() {
        let pattern = AccessPattern::new("test".to_string());
        assert_eq!(pattern.priority_score(), 0.0);
    }

    #[test]
    fn test_frequency_predictor() {
        let predictor = FrequencyPredictor::new(1.0, 100);
        assert_eq!(predictor.metrics.training_runs, 0);
    }

    #[tokio::test]
    async fn test_predictor_disabled() {
        let mut predictor = Predictor::new();
        predictor.set_enabled(false);

        let patterns = vec![AccessPattern::new("test".to_string())];
        let predictions = predictor.get_predictions(&patterns).await.unwrap();

        assert!(predictions.is_empty());
    }
}
