#[cfg_attr(target_os = "macos", path = "collector/macos.rs")]
#[cfg_attr(target_os = "linux", path = "collector/linux.rs")]
#[cfg_attr(target_os = "windows", path = "collector/windows.rs")]
#[allow(unused_attributes)]
#[cfg_attr(feature = "dummy", path = "collector/dummy.rs")]
mod collector;

#[cfg(all(
    not(feature = "dummy"),
    not(any(target_os = "macos", target_os = "linux", target_os = "windows"))
))]
compile_error!(
    "A feature \"dummy\" must be enabled to compile this crate on non supported platforms."
);

pub use collector::collect;

/// Process metrics
/// https://prometheus.io/docs/instrumenting/writing_clientlibs/#process-metrics
#[derive(Debug, Default, PartialEq)]
pub struct Metrics {
    /// Total user and system CPU time spent in seconds.
    pub cpu_seconds_total: Option<f64>,
    /// Number of open file descriptors.
    pub open_fds: Option<u64>,
    /// Maximum number of open file descriptors.
    /// 0 indicates 'unlimited'.
    pub max_fds: Option<u64>,
    /// Virtual memory size in bytes.
    pub virtual_memory_bytes: Option<u64>,
    /// Maximum amount of virtual memory available in bytes.
    /// 0 indicates 'unlimited'.
    pub virtual_memory_max_bytes: Option<u64>,
    /// Resident memory size in bytes.
    pub resident_memory_bytes: Option<u64>,
    /// Start time of the process since unix epoch in seconds.
    pub start_time_seconds: Option<u64>,
    /// Numberof OS threads in the process.
    pub threads: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    fn fibonacci(n: u64) -> u64 {
        match n {
            0 => 0,
            1 => 1,
            _ => fibonacci(n - 2) + fibonacci(n - 1),
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    #[test]
    fn test_collect_internal_ok() {
        fibonacci(40);
        let m = collect();
        dbg!(&m);
        assert_matches!(m.cpu_seconds_total, Some(_));
        assert_matches!(m.open_fds, Some(_));
        assert_matches!(m.max_fds, Some(_));
        assert_matches!(m.virtual_memory_bytes, Some(_));
        #[cfg(not(target_os = "windows"))]
        assert_matches!(m.virtual_memory_max_bytes, Some(_)); // maybe 'unlimited'
        assert_matches!(m.resident_memory_bytes, Some(_));
        assert_matches!(m.start_time_seconds, Some(_));
        #[cfg(not(target_os = "windows"))]
        assert_matches!(m.threads, Some(_));
    }

    #[cfg(not(target_os = "macos"))]
    #[cfg(not(target_os = "linux"))]
    #[cfg(not(target_os = "windows"))]
    #[cfg(feature = "dummy")]
    #[test]
    fn test_collect_internal_ok_dummy() {
        fibonacci(40);
        let m = collect();
        dbg!(&m);
        assert_matches!(m.cpu_seconds_total, None);
        assert_matches!(m.open_fds, None);
        assert_matches!(m.max_fds, None);
        assert_matches!(m.virtual_memory_bytes, None);
        assert_matches!(m.virtual_memory_max_bytes, None);
        assert_matches!(m.resident_memory_bytes, None);
        assert_matches!(m.start_time_seconds, None);
        assert_matches!(m.threads, None);
    }
}
