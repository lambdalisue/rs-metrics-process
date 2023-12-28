//! This crate provides [Prometheus] style [process metrics] collector of [metrics] crate for Linux, macOS, and Windows.
//! Collector code is manually re-written to Rust from an official prometheus client of go ([client_golang])
//!
//! [Prometheus]: https://prometheus.io/
//! [process metrics]: https://prometheus.io/docs/instrumenting/writing_clientlibs/#process-metrics
//! [metrics]: https://crates.io/crates/metrics
//!
//! # Supported metrics
//!
//! This crate supports the following metrics, equal to what official prometheus client of go ([client_golang]) provides.
//!
//! | Metric name                        | Help string                                            | Linux | macOS | Windows |
//! | ---------------------------------- | ------------------------------------------------------ | ----- | ----- | ------- |
//! | `process_cpu_seconds_total`        | Total user and system CPU time spent in seconds.       | x     | x     | x       |
//! | `process_open_fds`                 | Number of open file descriptors.                       | x     | x     | x       |
//! | `process_max_fds`                  | Maximum number of open file descriptors.               | x     | x     | x       |
//! | `process_virtual_memory_bytes`     | Virtual memory size in bytes.                          | x     | x     | x       |
//! | `process_virtual_memory_max_bytes` | Maximum amount of virtual memory available in bytes.   | x     | x     |         |
//! | `process_resident_memory_bytes`    | Resident memory size in bytes.                         | x     | x     | x       |
//! | `process_heap_bytes`               | Process heap size in bytes.                            |       |       |         |
//! | `process_start_time_seconds`       | Start time of the process since unix epoch in seconds. | x     | x     | x       |
//! | `process_threads`                  | Number of OS threads in the process.                   | x     | x     |         |
//!
//! [client_golang]: https://github.com/prometheus/client_golang
//!
//! # Usage
//!
//! Use this crate with [metrics-exporter-prometheus] as an exporter like:
//!
//! [metrics-exporter-prometheus]: https://crates.io/crates/metrics-exporter-prometheus
//!
//! ```no_run
//! use std::thread;
//! use std::time::{Duration, Instant};
//!
//! use metrics_exporter_prometheus::PrometheusBuilder;
//! use metrics_process::Collector;
//!
//! let builder = PrometheusBuilder::new();
//! builder
//!     .install()
//!     .expect("failed to install Prometheus recorder");
//!
//! let collector = Collector::default();
//! // Call `describe()` method to register help string.
//! collector.describe();
//!
//! loop {
//!     let s = Instant::now();
//!     // Periodically call `collect()` method to update information.
//!     collector.collect();
//!     thread::sleep(Duration::from_millis(750));
//! }
//! ```
//!
//! Or with [axum] (or any web application framework you like) to collect metrics whenever
//! the `/metrics` endpoint is invoked like:
//!
//! [axum]: https://crates.io/crates/axum
//!
//! ```no_run
//! use axum::{routing::get, Router};
//! use metrics_exporter_prometheus::PrometheusBuilder;
//! use metrics_process::Collector;
//! use tokio::net::TcpListener;
//!
//! #[tokio::main]
//! async fn main() {
//!     let builder = PrometheusBuilder::new();
//!     let handle = builder
//!         .install_recorder()
//!         .expect("failed to install Prometheus recorder");
//!
//!     let collector = Collector::default();
//!     // Call `describe()` method to register help string.
//!     collector.describe();
//!
//!     let app = Router::new().route(
//!         "/metrics",
//!         get(move || {
//!             // Collect information just before handle '/metrics'
//!             collector.collect();
//!             std::future::ready(handle.render())
//!         }),
//!     );
//!     let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```
//!
//! # Difference from [metrics-process-promstyle]
//!
//! It seems [metrics-process-promstyle] only support Linux but this crate (metrics-process) supports Linux, macOS, and Windows.
//! Additionally, this crate supports `process_open_fds` and `process_max_fds` addition to what metrics-process-promstyle supports.
//!
//! [metrics-process-promstyle]: https://crates.io/crates/metrics-process-promstyle

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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Collector {
    metrics: Arc<Metrics>,
}

impl Default for Collector {
    /// Create a new Collector instance without prefix. This is the same as
    /// calling `Collector::new` with an empty string for prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// # use metrics_process::Collector;
    /// let collector = Collector::default();
    /// ```
    fn default() -> Self {
        Self {
            metrics: Arc::default(),
        }
    }
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
        drop(self);
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
    /// ```
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
    /// ```
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
