//! Performance monitoring and data collection
//!
//! Collects performance metrics and exports them as JSON for visualization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: String,
    pub benchmark_name: String,
    pub measurements: Vec<PerformanceMeasurement>,
    pub metadata: HashMap<String, String>,
}

/// Individual performance measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMeasurement {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub sample_size: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
}

/// Performance collector
pub struct PerformanceCollector {
    measurements: Vec<PerformanceMeasurement>,
    start_times: HashMap<String, Instant>,
}

impl PerformanceCollector {
    pub fn new() -> Self {
        Self {
            measurements: Vec::new(),
            start_times: HashMap::new(),
        }
    }

    /// Start timing a measurement
    pub fn start_measurement(&mut self, name: &str) {
        self.start_times.insert(name.to_string(), Instant::now());
    }

    /// End timing a measurement and record it
    pub fn end_measurement(&mut self, name: &str, sample_size: usize) {
        if let Some(start_time) = self.start_times.remove(name) {
            let duration = start_time.elapsed();
            let value_ms = duration.as_secs_f64() * 1000.0;

            let measurement = PerformanceMeasurement {
                name: name.to_string(),
                value: value_ms,
                unit: "ms".to_string(),
                sample_size,
                min: value_ms,
                max: value_ms,
                mean: value_ms,
                std_dev: 0.0,
            };

            self.measurements.push(measurement);
        }
    }

    /// Record a measurement with multiple samples
    pub fn record_measurement(&mut self, name: &str, values: &[f64], unit: &str) {
        if values.is_empty() {
            return;
        }

        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        let measurement = PerformanceMeasurement {
            name: name.to_string(),
            value: mean,
            unit: unit.to_string(),
            sample_size: values.len(),
            min,
            max,
            mean,
            std_dev,
        };

        self.measurements.push(measurement);
    }

    /// Get all measurements
    pub fn measurements(&self) -> &[PerformanceMeasurement] {
        &self.measurements
    }

    /// Export measurements as JSON
    pub fn export_json(&self, benchmark_name: &str) -> Result<String, serde_json::Error> {
        use chrono::Utc;

        let metrics = PerformanceMetrics {
            timestamp: Utc::now().to_rfc3339(),
            benchmark_name: benchmark_name.to_string(),
            measurements: self.measurements.clone(),
            metadata: HashMap::new(),
        };

        serde_json::to_string_pretty(&metrics)
    }

    /// Save measurements to file
    pub fn save_to_file(&self, benchmark_name: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = self.export_json(benchmark_name)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

/// Performance profiler for detailed analysis
pub struct PerformanceProfiler {
    collector: PerformanceCollector,
    traces: Vec<PerformanceTrace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrace {
    pub name: String,
    pub start_time: String,
    pub duration_ms: f64,
    pub memory_usage_kb: Option<u64>,
    pub thread_id: String,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            collector: PerformanceCollector::new(),
            traces: Vec::new(),
        }
    }

    /// Start profiling a function or operation
    pub fn start_trace(&mut self, name: &str) -> TraceGuard {
        let start_time = Instant::now();
        let trace = PerformanceTrace {
            name: name.to_string(),
            start_time: chrono::Utc::now().to_rfc3339(),
            duration_ms: 0.0,
            memory_usage_kb: None,
            thread_id: format!("{:?}", std::thread::current().id()),
        };

        self.traces.push(trace);
        let trace_index = self.traces.len() - 1;

        TraceGuard {
            profiler: self,
            trace_index,
            start_time,
        }
    }

    /// End trace and record duration
    fn end_trace(&mut self, trace_index: usize, duration: Duration) {
        if let Some(trace) = self.traces.get_mut(trace_index) {
            trace.duration_ms = duration.as_secs_f64() * 1000.0;
        }
    }

    /// Export profiling data as JSON
    pub fn export_profile_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.traces)
    }
}

/// RAII guard for performance tracing
pub struct TraceGuard<'a> {
    profiler: &'a mut PerformanceProfiler,
    trace_index: usize,
    start_time: Instant,
}

impl<'a> Drop for TraceGuard<'a> {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        self.profiler.end_trace(self.trace_index, duration);
    }
}

/// Memory usage measurement utilities
pub mod memory {
    use std::sync::atomic::{AtomicU64, Ordering};

    static MEMORY_USAGE: AtomicU64 = AtomicU64::new(0);

    pub fn record_memory_usage(bytes: u64) {
        MEMORY_USAGE.store(bytes, Ordering::Relaxed);
    }

    pub fn get_memory_usage_kb() -> u64 {
        MEMORY_USAGE.load(Ordering::Relaxed) / 1024
    }

    /// Get current memory usage from system (if available)
    pub fn get_system_memory_usage() -> Option<u64> {
        // This is a simplified implementation
        // In a real implementation, you'd use platform-specific APIs
        // like getrusage() on Unix or GetProcessMemoryInfo() on Windows
        Some(get_memory_usage_kb())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_collector() {
        let mut collector = PerformanceCollector::new();

        // Record some measurements
        collector.record_measurement("test_operation", &[100.0, 110.0, 90.0], "ms");

        let measurements = collector.measurements();
        assert_eq!(measurements.len(), 1);

        let measurement = &measurements[0];
        assert_eq!(measurement.name, "test_operation");
        assert_eq!(measurement.unit, "ms");
        assert_eq!(measurement.sample_size, 3);
        assert_eq!(measurement.mean, 100.0);
    }

    #[test]
    fn test_performance_profiler() {
        let mut profiler = PerformanceProfiler::new();

        {
            let _trace = profiler.start_trace("test_function");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        assert_eq!(profiler.traces.len(), 1);
        let trace = &profiler.traces[0];
        assert_eq!(trace.name, "test_function");
        assert!(trace.duration_ms >= 10.0);
    }

    #[test]
    fn test_json_export() {
        let mut collector = PerformanceCollector::new();
        collector.record_measurement("test", &[50.0], "ms");

        let json = collector.export_json("test_benchmark").unwrap();
        assert!(json.contains("test_benchmark"));
        assert!(json.contains("test"));
    }
}
