//! This crate provides [Prometheus][] style [process metrics][] collector of [metrics][] crate.
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
//! Use this crate with [metrics-exporter-prometheus][] as an exporter like:
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
//! fn main() {
//!     let builder = PrometheusBuilder::new();
//!     builder
//!         .install()
//!         .expect("failed to install Prometheus recorder");
//!
//!     let collector = Collector::new("");
//!     // Call `describe()` method to register help string.
//!     collector.describe();
//!
//!     loop {
//!         let s = Instant::now();
//!         // Periodically call `collect()` method to update information.
//!         collector.collect();
//!         thread::sleep(Duration::from_millis(750));
//!     }
//! }
//! ```
//!
//! Or with [axum][] (or any web application framework you like) to collect metrics whenever
//! the `/metrics` endpoint is invoked like:
//!
//! [axum]: https://crates.io/crates/axum
//!
//! ```no_run
//! use axum::{routing::get, Router, Server};
//! use metrics_exporter_prometheus::PrometheusBuilder;
//! use metrics_process::Collector;
//!
//! #[tokio::main]
//! async fn main() {
//!     let builder = PrometheusBuilder::new();
//!     let handle = builder
//!         .install_recorder()
//!         .expect("failed to install Prometheus recorder");
//!
//!     let collector = Collector::new("");
//!     // Call `describe()` method to register help string.
//!     collector.describe();
//!
//!     let addr = "127.0.0.1:9000".parse().unwrap();
//!     let app = Router::new().route(
//!         "/metrics",
//!         get(move || {
//!             // Collect information just before handle '/metrics'
//!             collector.collect();
//!             std::future::ready(handle.render())
//!         }),
//!     );
//!     Server::bind(&addr)
//!         .serve(app.into_make_service())
//!         .await
//!         .unwrap();
//! }
//! ```
mod collector;

use metrics::{describe_gauge, gauge, Unit};

/// Prometheus style process metrics collector
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Collector {
    prefix: String,
}

impl Collector {
    /// Create a new Collector instance
    ///
    /// * `prefix` - A prefix string that is prepended to metric keys.
    ///
    /// # Examples
    ///
    /// ```
    /// # use metrics_process::Collector;
    /// let collector = Collector::new("my_metrics_");
    /// ```
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
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
    /// let collector = Collector::new("my_metrics_");
    /// // Describe collector
    /// collector.describe();
    /// # }
    /// ```
    pub fn describe(&self) {
        let prefix = &self.prefix;
        describe_gauge!(
            format!("{}process_cpu_seconds_total", prefix),
            Unit::Seconds,
            "Total user and system CPU time spent in seconds."
        );
        describe_gauge!(
            format!("{}process_open_fds", prefix),
            Unit::Count,
            "Number of open file descriptors."
        );
        describe_gauge!(
            format!("{}process_max_fds", prefix),
            Unit::Count,
            "Maximum number of open file descriptors."
        );
        describe_gauge!(
            format!("{}process_virtual_memory_bytes", prefix),
            Unit::Bytes,
            "Virtual memory size in bytes."
        );
        #[cfg(not(target_os = "windows"))]
        describe_gauge!(
            format!("{}process_virtual_memory_max_bytes", prefix),
            Unit::Bytes,
            "Maximum amount of virtual memory available in bytes."
        );
        describe_gauge!(
            format!("{}process_resident_memory_bytes", prefix),
            Unit::Bytes,
            "Resident memory size in bytes."
        );
        describe_gauge!(
            format!("{}process_start_time_seconds", prefix),
            Unit::Seconds,
            "Start time of the process since unix epoch in seconds."
        );
        #[cfg(not(target_os = "windows"))]
        describe_gauge!(
            format!("{}process_threads", prefix),
            Unit::Count,
            "Numberof OS threads in the process."
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
    /// let collector = Collector::new("my_metrics_");
    /// collector.describe();
    /// // Collect metrics
    /// collector.collect();
    /// # }
    /// ```
    pub fn collect(&self) {
        let prefix = &self.prefix;
        let mut m = collector::collect();
        if let Some(v) = m.cpu_seconds_total.take() {
            gauge!(format!("{}process_cpu_seconds_total", prefix), v);
        }
        if let Some(v) = m.open_fds.take() {
            gauge!(format!("{}process_open_fds", prefix), v as f64);
        }
        if let Some(v) = m.max_fds.take() {
            gauge!(format!("{}process_max_fds", prefix), v as f64);
        }
        if let Some(v) = m.virtual_memory_bytes.take() {
            gauge!(format!("{}process_virtual_memory_bytes", prefix), v as f64);
        }
        #[cfg(not(target_os = "windows"))]
        if let Some(v) = m.virtual_memory_max_bytes.take() {
            gauge!(
                format!("{}process_virtual_memory_max_bytes", prefix),
                v as f64,
            );
        }
        if let Some(v) = m.resident_memory_bytes.take() {
            gauge!(format!("{}process_resident_memory_bytes", prefix), v as f64);
        }
        if let Some(v) = m.start_time_seconds.take() {
            gauge!(format!("{}process_start_time_seconds", prefix), v as f64);
        }
        #[cfg(not(target_os = "windows"))]
        if let Some(v) = m.threads.take() {
            gauge!(format!("{}process_therads", prefix), v as f64);
        }
    }
}
