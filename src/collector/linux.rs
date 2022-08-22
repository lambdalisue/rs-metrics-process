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
                metrics.start_time_seconds = boot_time_secs().ok().map(|b| {
                    let t = stat.starttime / ticks_per_second;
                    t + b
                });
                metrics.cpu_seconds_total = {
                    let t = (stat.utime + stat.stime) as f64;
                    Some(t / (ticks_per_second as f64))
                };
            }
            metrics.resident_memory_bytes = stat.rss_bytes().ok();
            metrics.virtual_memory_bytes = Some(stat.vsize);
            metrics.threads = Some(stat.num_threads as u64);
        }
        metrics.open_fds = proc.fd_count().ok().map(|v| v as u64);
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
