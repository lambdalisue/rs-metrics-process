#![doc = include_str!("../README.md")]
mod collector;

use std::sync::Arc;

use metrics::{describe_gauge, gauge, Unit};

/// Metrics names
#[derive(Debug, PartialEq, Eq)]
struct Metrics {
    cpu_seconds_total: Arc<str>,
    open_fds: Arc<str>,
    max_fds: Arc<str>,
    virtual_memory_bytes: Arc<str>,
    virtual_memory_max_bytes: Arc<str>,
    resident_memory_bytes: Arc<str>,
    start_time_seconds: Arc<str>,
    threads: Arc<str>,
}

impl Metrics {
    // Create new Metrics, allocating prefixed strings for metrics names.
    fn new(prefix: impl AsRef<str>) -> Self {
        let prefix = prefix.as_ref();
        Self {
            cpu_seconds_total: format!("{prefix}process_cpu_seconds_total").into(),
            open_fds: format!("{prefix}process_open_fds").into(),
            max_fds: format!("{prefix}process_max_fds").into(),
            virtual_memory_bytes: format!("{prefix}process_virtual_memory_bytes").into(),
            virtual_memory_max_bytes: format!("{prefix}process_virtual_memory_max_bytes").into(),
            resident_memory_bytes: format!("{prefix}process_resident_memory_bytes").into(),
            start_time_seconds: format!("{prefix}process_start_time_seconds").into(),
            threads: format!("{prefix}process_threads").into(),
        }
    }
}

impl Default for Metrics {
    // Create new Metrics, without prefixing and thus allocating.
    fn default() -> Self {
        Self::new("")
    }
}

/// Prometheus style process metrics collector
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Collector {
    metrics: Arc<Metrics>,
}

impl Collector {
    /// Add an prefix that is prepended to metric keys.
    /// # Examples
    ///
    /// ```
    /// # use metrics_process::Collector;
    /// let collector = Collector::default().prefix("my_prefix_");
    /// ```
    ///
    /// # Deprecated
    ///
    /// The new interface for creating a Collector should be utilized.
    ///
    /// ```
    /// # use metrics_process::Collector;
    /// let collector = Collector::new("my_prefix_");
    /// ```
    #[deprecated(since = "1.1.0", note = "Use `Collector::new(prefix)`.")]
    pub fn prefix(self, prefix: impl Into<String>) -> Self {
        let _ = self;
        Self::new(prefix.into())
    }

    /// Create a new Collector instance with the provided prefix that is
    /// prepended to metric keys.
    ///
    /// # Examples
    ///
    /// ```
    /// # use metrics_process::Collector;
    /// let collector = Collector::default();
    /// ```
    pub fn new(prefix: impl AsRef<str>) -> Self {
        Self {
            metrics: Arc::new(Metrics::new(prefix)),
        }
    }

    /// Describe available metrics through `describe_gauge!` macro of `metrics` crate.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use metrics_exporter_prometheus::PrometheusBuilder;
    /// # use metrics_process::Collector;
    /// # #[tokio::main]
    /// # async fn main() {
    /// // Recorder must be initialized prior to describe.
    /// let builder = PrometheusBuilder::new();
    /// builder.install().expect("failed to install recorder/exporter");
    ///
    /// let collector = Collector::default();
    /// // Describe collector
    /// collector.describe();
    /// # }
    /// ```
    pub fn describe(&self) {
        let metrics = self.metrics.as_ref();

        describe_gauge!(
            Arc::clone(&metrics.cpu_seconds_total),
            Unit::Seconds,
            "Total user and system CPU time spent in seconds."
        );
        describe_gauge!(
            Arc::clone(&metrics.open_fds),
            Unit::Count,
            "Number of open file descriptors."
        );
        describe_gauge!(
            Arc::clone(&metrics.max_fds),
            Unit::Count,
            "Maximum number of open file descriptors."
        );
        describe_gauge!(
            Arc::clone(&metrics.virtual_memory_bytes),
            Unit::Bytes,
            "Virtual memory size in bytes."
        );
        #[cfg(not(target_os = "windows"))]
        describe_gauge!(
            Arc::clone(&metrics.virtual_memory_max_bytes),
            Unit::Bytes,
            "Maximum amount of virtual memory available in bytes."
        );
        describe_gauge!(
            Arc::clone(&metrics.resident_memory_bytes),
            Unit::Bytes,
            "Resident memory size in bytes."
        );
        describe_gauge!(
            Arc::clone(&metrics.start_time_seconds),
            Unit::Seconds,
            "Start time of the process since unix epoch in seconds."
        );
        #[cfg(not(target_os = "windows"))]
        describe_gauge!(
            Arc::clone(&metrics.threads),
            Unit::Count,
            "Number of OS threads in the process."
        );
    }

    /// Collect metrics and record through `gauge!` macro of `metrics` crate.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use metrics_exporter_prometheus::PrometheusBuilder;
    /// # use metrics_process::Collector;
    /// # #[tokio::main]
    /// # async fn main() {
    /// // Recorder must be initialized prior to describe.
    /// let builder = PrometheusBuilder::new();
    /// builder.install().expect("failed to install recorder/exporter");
    ///
    /// let collector = Collector::default();
    /// collector.describe();
    /// // Collect metrics
    /// collector.collect();
    /// # }
    /// ```
    pub fn collect(&self) {
        let metrics = self.metrics.as_ref();
        let mut m = collector::collect();
        if let Some(v) = m.cpu_seconds_total.take() {
            gauge!(Arc::clone(&metrics.cpu_seconds_total)).set(v);
        }
        if let Some(v) = m.open_fds.take() {
            gauge!(Arc::clone(&metrics.open_fds)).set(v as f64);
        }
        if let Some(v) = m.max_fds.take() {
            gauge!(Arc::clone(&metrics.max_fds)).set(v as f64);
        }
        if let Some(v) = m.virtual_memory_bytes.take() {
            gauge!(Arc::clone(&metrics.virtual_memory_bytes)).set(v as f64);
        }
        #[cfg(not(target_os = "windows"))]
        if let Some(v) = m.virtual_memory_max_bytes.take() {
            gauge!(Arc::clone(&metrics.virtual_memory_max_bytes)).set(v as f64);
        }
        if let Some(v) = m.resident_memory_bytes.take() {
            gauge!(Arc::clone(&metrics.resident_memory_bytes)).set(v as f64);
        }
        if let Some(v) = m.start_time_seconds.take() {
            gauge!(Arc::clone(&metrics.start_time_seconds)).set(v as f64);
        }
        #[cfg(not(target_os = "windows"))]
        if let Some(v) = m.threads.take() {
            gauge!(Arc::clone(&metrics.threads)).set(v as f64);
        }
    }
}
