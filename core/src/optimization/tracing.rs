use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info, instrument, span, warn, Level};

/// Performance metrics for critical operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation_name: String,
    pub duration: Duration,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(skip)]
    pub thread_id: Option<std::thread::ThreadId>,
    pub additional_context: HashMap<String, String>,
}

/// Performance tracker for measuring operation durations
pub struct PerformanceTracker {
    operation_name: String,
    start_time: Instant,
    context: HashMap<String, String>,
    span: tracing::Span,
}

impl PerformanceTracker {
    /// Start tracking a new operation
    #[instrument(level = "debug", fields(operation = %operation_name))]
    pub fn start(operation_name: &str) -> Self {
        let span = span!(Level::DEBUG, "performance", operation = %operation_name);
        {
            let _guard = span.enter();
            debug!(operation = %operation_name, "Starting performance tracking");
        }

        Self {
            operation_name: operation_name.to_string(),
            start_time: Instant::now(),
            context: HashMap::new(),
            span,
        }
    }

    /// Add contextual information to the performance tracking
    pub fn add_context(&mut self, key: &str, value: &str) {
        self.context.insert(key.to_string(), value.to_string());
        debug!(key = %key, value = %value, "Added performance context");
    }

    /// Finish tracking and return performance metrics
    pub fn finish(self) -> PerformanceMetrics {
        let duration = self.start_time.elapsed();
        let _guard = self.span.enter();

        let metrics = PerformanceMetrics {
            operation_name: self.operation_name.clone(),
            duration,
            timestamp: chrono::Utc::now(),
            thread_id: Some(std::thread::current().id()),
            additional_context: self.context,
        };

        // Log performance based on duration thresholds
        match duration.as_millis() {
            0..=100 => debug!(
                operation = %self.operation_name,
                duration_ms = duration.as_millis(),
                "Operation completed (fast)"
            ),
            101..=1000 => info!(
                operation = %self.operation_name,
                duration_ms = duration.as_millis(),
                "Operation completed (normal)"
            ),
            1001..=5000 => warn!(
                operation = %self.operation_name,
                duration_ms = duration.as_millis(),
                "Operation completed (slow)"
            ),
            _ => warn!(
                operation = %self.operation_name,
                duration_ms = duration.as_millis(),
                "Operation completed (very slow - investigate)"
            ),
        }

        metrics
    }
}

/// Macro for easy performance tracking with automatic cleanup
#[macro_export]
macro_rules! track_performance {
    ($operation:expr) => {{
        let _tracker = $crate::optimization::tracing::PerformanceTracker::start($operation);
    }};
    ($operation:expr, $block:block) => {{
        let mut tracker = $crate::optimization::tracing::PerformanceTracker::start($operation);
        let result = $block;
        let _metrics = tracker.finish();
        result
    }};
}

/// Performance monitoring for async operations
#[derive(Debug)]
pub struct AsyncPerformanceTracker {
    operation_name: String,
    start_time: Instant,
    context: HashMap<String, String>,
}

impl AsyncPerformanceTracker {
    /// Start tracking an async operation
    pub fn start(operation_name: &str) -> Self {
        debug!(operation = %operation_name, "Starting async performance tracking");

        Self {
            operation_name: operation_name.to_string(),
            start_time: Instant::now(),
            context: HashMap::new(),
        }
    }

    /// Add contextual information
    pub fn add_context(&mut self, key: &str, value: &str) {
        self.context.insert(key.to_string(), value.to_string());
    }

    /// Finish tracking
    #[instrument(level = "debug", fields(operation = %self.operation_name))]
    pub async fn finish(self) -> PerformanceMetrics {
        let duration = self.start_time.elapsed();

        let metrics = PerformanceMetrics {
            operation_name: self.operation_name.clone(),
            duration,
            timestamp: chrono::Utc::now(),
            thread_id: Some(std::thread::current().id()),
            additional_context: self.context,
        };

        // Log performance with structured data
        info!(
            operation = %self.operation_name,
            duration_ms = duration.as_millis(),
            duration_us = duration.as_micros(),
            context = ?metrics.additional_context,
            "Async operation completed"
        );

        metrics
    }
}

/// Critical path performance monitoring
#[derive(Debug)]
pub struct CriticalPathMonitor {
    checkpoints: Vec<(String, Instant)>,
    start_time: Instant,
    operation_name: String,
}

impl CriticalPathMonitor {
    /// Start monitoring a critical path
    #[instrument(level = "info", fields(operation = %operation_name))]
    pub fn start(operation_name: &str) -> Self {
        info!(operation = %operation_name, "Starting critical path monitoring");

        Self {
            checkpoints: Vec::new(),
            start_time: Instant::now(),
            operation_name: operation_name.to_string(),
        }
    }

    /// Add a checkpoint along the critical path
    pub fn checkpoint(&mut self, checkpoint_name: &str) {
        let now = Instant::now();
        self.checkpoints.push((checkpoint_name.to_string(), now));

        let elapsed_since_start = now.duration_since(self.start_time);
        debug!(
            operation = %self.operation_name,
            checkpoint = %checkpoint_name,
            elapsed_ms = elapsed_since_start.as_millis(),
            "Critical path checkpoint reached"
        );
    }

    /// Finish monitoring and log the complete critical path
    #[instrument(level = "info", fields(operation = %self.operation_name))]
    pub fn finish(self) -> Vec<PerformanceMetrics> {
        let total_duration = self.start_time.elapsed();
        let mut metrics = Vec::new();
        let mut previous_time = self.start_time;

        // Generate metrics for each checkpoint
        for (checkpoint_name, checkpoint_time) in &self.checkpoints {
            let segment_duration = checkpoint_time.duration_since(previous_time);

            metrics.push(PerformanceMetrics {
                operation_name: format!("{}::{}", self.operation_name, checkpoint_name),
                duration: segment_duration,
                timestamp: chrono::Utc::now(),
                thread_id: Some(std::thread::current().id()),
                additional_context: HashMap::new(),
            });

            info!(
                operation = %self.operation_name,
                checkpoint = %checkpoint_name,
                segment_duration_ms = segment_duration.as_millis(),
                total_elapsed_ms = checkpoint_time.duration_since(self.start_time).as_millis(),
                "Critical path segment completed"
            );

            previous_time = *checkpoint_time;
        }

        info!(
            operation = %self.operation_name,
            total_duration_ms = total_duration.as_millis(),
            checkpoint_count = self.checkpoints.len(),
            "Critical path monitoring completed"
        );

        metrics
    }
}

/// Utility functions for common performance tracking scenarios
pub mod utils {
    use super::*;

    /// Track database operation performance
    #[instrument(level = "debug", fields(query_type = %query_type), skip(operation))]
    pub async fn track_database_operation<F, T>(
        query_type: &str,
        operation: F,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        let mut tracker = AsyncPerformanceTracker::start(&format!("db::{}", query_type));
        tracker.add_context("type", "database");
        tracker.add_context("query_type", query_type);

        let result = operation.await;
        let _metrics = tracker.finish().await;

        match &result {
            Ok(_) => debug!(query_type = %query_type, "Database operation successful"),
            Err(e) => warn!(query_type = %query_type, error = %e, "Database operation failed"),
        }

        result
    }

    /// Track API request performance
    #[instrument(level = "debug", fields(endpoint = %endpoint, method = %method), skip(operation))]
    pub async fn track_api_request<F, T>(
        method: &str,
        endpoint: &str,
        operation: F,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        let mut tracker = AsyncPerformanceTracker::start(&format!("api::{}::{}", method, endpoint));
        tracker.add_context("type", "api_request");
        tracker.add_context("method", method);
        tracker.add_context("endpoint", endpoint);

        let result = operation.await;
        let metrics = tracker.finish().await;

        // Log additional API-specific metrics
        info!(
            method = %method,
            endpoint = %endpoint,
            duration_ms = metrics.duration.as_millis(),
            success = result.is_ok(),
            "API request completed"
        );

        result
    }

    /// Track adaptive algorithm performance
    pub fn track_algorithm_performance<F, T>(algorithm_name: &str, operation: F) -> T
    where
        F: FnOnce() -> T,
    {
        track_performance!(&format!("algorithm::{}", algorithm_name), { operation() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_performance_tracker() {
        let mut tracker = PerformanceTracker::start("test_operation");
        tracker.add_context("test_key", "test_value");

        // Simulate some work
        thread::sleep(Duration::from_millis(10));

        let metrics = tracker.finish();
        assert_eq!(metrics.operation_name, "test_operation");
        assert!(metrics.duration >= Duration::from_millis(10));
        assert!(metrics.additional_context.contains_key("test_key"));
    }

    #[test]
    fn test_critical_path_monitor() {
        let mut monitor = CriticalPathMonitor::start("test_critical_path");

        thread::sleep(Duration::from_millis(5));
        monitor.checkpoint("step1");

        thread::sleep(Duration::from_millis(5));
        monitor.checkpoint("step2");

        let metrics = monitor.finish();
        assert_eq!(metrics.len(), 2);
        assert!(metrics[0].operation_name.contains("step1"));
        assert!(metrics[1].operation_name.contains("step2"));
    }

    #[tokio::test]
    async fn test_async_performance_tracker() {
        let mut tracker = AsyncPerformanceTracker::start("test_async_operation");
        tracker.add_context("async_test", "true");

        // Simulate async work
        tokio::time::sleep(Duration::from_millis(10)).await;

        let metrics = tracker.finish().await;
        assert_eq!(metrics.operation_name, "test_async_operation");
        assert!(metrics.duration >= Duration::from_millis(10));
    }
}
