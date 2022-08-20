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
        let m = collector::collect();
        if m.cpu_seconds_total.is_normal() {
            gauge!(
                format!("{}process_cpu_seconds_total", prefix),
                m.cpu_seconds_total,
            );
        }
        if m.open_fds > 0 {
            gauge!(format!("{}process_open_fds", prefix), m.open_fds as f64);
        }
        if m.max_fds > 0 {
            gauge!(format!("{}process_max_fds", prefix), m.max_fds as f64);
        }
        if m.virtual_memory_bytes > 0 {
            gauge!(
                format!("{}process_virtual_memory_bytes", prefix),
                m.virtual_memory_bytes as f64,
            );
        }
        if m.virtual_memory_max_bytes > 0 {
            gauge!(
                format!("{}process_virtual_memory_max_bytes", prefix),
                m.virtual_memory_max_bytes as f64,
            );
        }
        if m.resident_memory_bytes > 0 {
            gauge!(
                format!("{}process_resident_memory_bytes", prefix),
                m.resident_memory_bytes as f64,
            );
        }
        if m.start_time_seconds > 0 {
            gauge!(
                format!("{}process_start_time_seconds", prefix),
                m.start_time_seconds as f64,
            );
        }
        if m.threads > 0 {
            gauge!(format!("{}process_therads", prefix), m.threads as f64);
        }
    }
}
