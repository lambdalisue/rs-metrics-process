//! This crate provides [process metrics][] of Prometheus for [metrics][].
//!
//! [process metrics]: https://prometheus.io/docs/instrumenting/writing_clientlibs/#process-metrics
//! [metrics]: https://github.com/metrics-rs/metrics
mod collector;

use metrics::{describe_gauge, gauge, Unit};

pub struct Collector {
    prefix: String,
}

impl Collector {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }

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
        describe_gauge!(
            format!("{}process_threads", prefix),
            Unit::Count,
            "Numberof OS threads in the process."
        );
    }

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
        if let Some(v) = m.threads.take() {
            gauge!(format!("{}process_therads", prefix), v as f64);
        }
    }
}
