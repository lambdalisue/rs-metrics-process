use libproc::libproc::file_info::ListFDs;
use libproc::libproc::pid_rusage::{pidrusage, RUsageInfoV2};
use libproc::libproc::proc_pid::{listpidinfo, pidinfo};
use libproc::libproc::task_info::TaskAllInfo;
use rlimit::{getrlimit, Resource};
use std::process;

use super::Metrics;

pub fn collect() -> Metrics {
    let pid = process::id() as i32;
    let mut metrics = Metrics::default();
    if let Ok(res) = pidrusage::<RUsageInfoV2>(pid) {
        metrics.cpu_seconds_total = {
            // Unit of 'ri_xxxx_time' is 'nano' (10^-9) seconds
            let t = res.ri_user_time + res.ri_system_time;
            (t as f64) * 1e-9
        };
    }
    if let Ok(info) = pidinfo::<TaskAllInfo>(pid, 0) {
        metrics.start_time_seconds = info.pbsd.pbi_start_tvsec;
        metrics.virtual_memory_bytes = info.ptinfo.pti_virtual_size;
        metrics.resident_memory_bytes = info.ptinfo.pti_resident_size;
        metrics.threads = info.ptinfo.pti_threadnum as u64;
        if let Ok(fds) = listpidinfo::<ListFDs>(pid, info.pbsd.pbi_nfiles as usize) {
            metrics.open_fds = fds.len() as u64;
        }
    }
    if let Ok((soft, _hard)) = getrlimit(Resource::AS) {
        metrics.virtual_memory_max_bytes = soft;
    }
    if let Ok((soft, _hard)) = getrlimit(Resource::NOFILE) {
        metrics.max_fds = soft;
    }
    metrics
}
