#![doc = include_str!("../README.md")]

mod collector;

use metrics::{describe_gauge, gauge, Unit};

/// Prometheus style process metrics collector
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Collector {
    cpu_seconds_total: &'static str,
    open_fds: &'static str,
    max_fds: &'static str,
    virtual_memory_bytes: &'static str,
    virtual_memory_max_bytes: &'static str,
    resident_memory_bytes: &'static str,
    start_time_seconds: &'static str,
    threads: &'static str,
}

impl Default for Collector {
    /// Create a new Collector instance without prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// # use metrics_process::Collector;
    /// let collector = Collector::default();
    /// ```
    fn default() -> Self {
        Self {
            cpu_seconds_total: "process_cpu_seconds_total",
            open_fds: "process_open_fds",
            max_fds: "process_max_fds",
            virtual_memory_bytes: "process_virtual_memory_bytes",
            virtual_memory_max_bytes: "process_virtual_memory_max_bytes",
            resident_memory_bytes: "process_resident_memory_bytes",
            start_time_seconds: "process_start_time_seconds",
            threads: "process_threads",
        }
    }
}

impl Collector {
    /// Add an prefix that is prepended to metric keys.
    /// # Examples
    ///
    /// ```
    /// # use metrics_process::Collector;
    /// let collector = Collector::new("my_prefix_");
    /// ```
    pub fn new(prefix: impl AsRef<str>) -> Self {
        let prefix = prefix.as_ref();
        Self {
            cpu_seconds_total: format!("{prefix}process_cpu_seconds_total").leak(),
            open_fds: format!("{prefix}process_open_fds").leak(),
            max_fds: format!("{prefix}process_max_fds").leak(),
            virtual_memory_bytes: format!("{prefix}process_virtual_memory_bytes").leak(),
            virtual_memory_max_bytes: format!("{prefix}process_virtual_memory_max_bytes").leak(),
            resident_memory_bytes: format!("{prefix}process_resident_memory_bytes").leak(),
            start_time_seconds: format!("{prefix}process_start_time_seconds").leak(),
            threads: format!("{prefix}process_threads").leak(),
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
        describe_gauge!(
            self.cpu_seconds_total,
            Unit::Seconds,
            "Total user and system CPU time spent in seconds."
        );
        describe_gauge!(
            self.open_fds,
            Unit::Count,
            "Number of open file descriptors."
        );
        describe_gauge!(
            self.max_fds,
            Unit::Count,
            "Maximum number of open file descriptors."
        );
        describe_gauge!(
            self.virtual_memory_bytes,
            Unit::Bytes,
            "Virtual memory size in bytes."
        );
        #[cfg(not(target_os = "windows"))]
        describe_gauge!(
            self.virtual_memory_max_bytes,
            Unit::Bytes,
            "Maximum amount of virtual memory available in bytes."
        );
        describe_gauge!(
            self.resident_memory_bytes,
            Unit::Bytes,
            "Resident memory size in bytes."
        );
        describe_gauge!(
            self.start_time_seconds,
            Unit::Seconds,
            "Start time of the process since unix epoch in seconds."
        );
        #[cfg(not(target_os = "windows"))]
        describe_gauge!(
            self.threads,
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
        let mut m = collector::collect();
        if let Some(v) = m.cpu_seconds_total.take() {
            gauge!(self.cpu_seconds_total).set(v as f64);
        }
        if let Some(v) = m.open_fds.take() {
            gauge!(self.open_fds).set(v as f64);
        }
        if let Some(v) = m.max_fds.take() {
            gauge!(self.max_fds).set(v as f64);
        }
        if let Some(v) = m.virtual_memory_bytes.take() {
            gauge!(self.virtual_memory_bytes).set(v as f64);
        }
        #[cfg(not(target_os = "windows"))]
        if let Some(v) = m.virtual_memory_max_bytes.take() {
            gauge!(self.virtual_memory_max_bytes).set(v as f64);
        }
        if let Some(v) = m.resident_memory_bytes.take() {
            gauge!(self.resident_memory_bytes).set(v as f64);
        }
        if let Some(v) = m.start_time_seconds.take() {
            gauge!(self.start_time_seconds).set(v as f64);
        }
        #[cfg(not(target_os = "windows"))]
        if let Some(v) = m.threads.take() {
            gauge!(self.threads).set(v as f64);
        }
    }
}
