use once_cell::sync::Lazy;
use procfs::process::{LimitValue, Process};

use super::Metrics;

static TICKS_PER_SECOND: Lazy<Option<f64>> =
    Lazy::new(|| procfs::ticks_per_second().ok().map(|v| v as f64));
static BOOT_TIME_SECS: Lazy<Option<u64>> = Lazy::new(|| procfs::boot_time_secs().ok());

pub fn collect() -> Metrics {
    let mut metrics = Metrics::default();
    if let Ok(proc) = Process::myself() {
        if let Ok(stat) = proc.stat() {
            if let Some(tps) = *TICKS_PER_SECOND {
                if let Some(bts) = *BOOT_TIME_SECS {
                    metrics.start_time_seconds = Some(bts + ((stat.starttime as f64) / tps) as u64);
                }
                metrics.cpu_seconds_total = Some((stat.utime + stat.stime) as f64 / tps);
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
