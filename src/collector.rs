#[cfg(target_os = "macos")]
mod macos;

/// Process metrics
/// https://prometheus.io/docs/instrumenting/writing_clientlibs/#process-metrics
#[derive(Debug, Default, PartialEq)]
pub struct Metrics {
    /// Total user and system CPU time spent in seconds.
    pub cpu_seconds_total: f64,
    /// Number of open file descriptors.
    pub open_fds: u64,
    /// Maximum number of open file descriptors.
    pub max_fds: u64,
    /// Virtual memory size in bytes.
    pub virtual_memory_bytes: u64,
    /// Maximum amount of virtual memory available in bytes.
    pub virtual_memory_max_bytes: u64,
    /// Resident memory size in bytes.
    pub resident_memory_bytes: u64,
    /// Start time of the process since unix epoch in seconds.
    pub start_time_seconds: u64,
    /// Numberof OS threads in the process.
    pub threads: u64,
}

#[cfg(target_os = "macos")]
pub use macos::collect;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_internal_ok() {
        let m = collect();
        assert!(m.cpu_seconds_total > 0.0);
        assert!(m.open_fds > 0);
        assert!(m.max_fds > 0);
        assert!(m.virtual_memory_bytes > 0);
        assert!(m.virtual_memory_max_bytes > 0);
        assert!(m.resident_memory_bytes > 0);
        assert!(m.start_time_seconds > 0);
        assert!(m.threads > 0);
    }
}
