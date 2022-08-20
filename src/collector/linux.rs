use procfs::{
    boot_time_secs,
    process::{LimitValue, Process},
    ticks_per_second,
};

use super::Metrics;

pub fn collect() -> Metrics {
    let mut metrics = Metrics::default();
    if let Ok(proc) = Process::myself() {
        if let Ok(stat) = proc.stat() {
            if let Ok(ticks_per_second) = ticks_per_second() {
                if let Ok(boot_time_secs) = boot_time_secs() {
                    metrics.start_time_seconds = {
                        let t = stat.starttime / ticks_per_second;
                        t + boot_time_secs
                    };
                }
                metrics.cpu_seconds_total = {
                    let t = (stat.utime + stat.stime) as f64;
                    t / (ticks_per_second as f64)
                };
            }
            if let Ok(rss) = stat.rss_bytes() {
                metrics.resident_memory_bytes = rss;
            }
            metrics.virtual_memory_bytes = stat.vsize;
            metrics.threads = stat.num_threads as u64;
        }
        if let Ok(fd_count) = proc.fd_count() {
            metrics.open_fds = fd_count as u64;
        }
        if let Ok(limit) = proc.limits() {
            metrics.max_fds = match limit.max_open_files.soft_limit {
                LimitValue::Value(v) => Some(v),
                LimitValue::Unlimited => Some(0),
            };
            metrics.virtual_memory_max_bytes = match limit.max_address_space.soft_limit {
                LimitValue::Value(v) => Some(v),
                LimitValue::Unlimited => Some(0),
            };
        }
    }
    metrics
}
