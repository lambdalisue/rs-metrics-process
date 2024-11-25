use libproc::libproc::file_info::ListFDs;
use libproc::libproc::pid_rusage::{pidrusage, RUsageInfoV2};
use libproc::libproc::proc_pid::{listpidinfo, pidinfo};
use libproc::libproc::task_info::TaskAllInfo;
use mach2::mach_time;
use once_cell::sync::Lazy;
use rlimit::{getrlimit, Resource};
use std::mem::MaybeUninit;
use std::process;

use super::Metrics;

// https://stackoverflow.com/a/72915413
// https://openradar.appspot.com/FB9546856
// https://developer.apple.com/documentation/kernel/mach_timebase_info_data_t
static TIMEBASE_TO_NANOSECONDS: Lazy<f64> = Lazy::new(|| {
    let mut info = MaybeUninit::uninit();
    let info = unsafe {
        mach_time::mach_timebase_info(info.as_mut_ptr());
        info.assume_init()
    };
    info.numer as f64 / info.denom as f64
});

pub fn collect() -> Metrics {
    let pid = process::id() as i32;
    let mut metrics = Metrics::default();
    if let Ok(res) = pidrusage::<RUsageInfoV2>(pid) {
        metrics.cpu_seconds_total = {
            let t = res.ri_user_time + res.ri_system_time;
            let t = t as f64 * *TIMEBASE_TO_NANOSECONDS / 1e9;
            Some(t)
        };
    }
    if let Ok(info) = pidinfo::<TaskAllInfo>(pid, 0) {
        metrics.start_time_seconds = Some(info.pbsd.pbi_start_tvsec);
        metrics.virtual_memory_bytes = Some(info.ptinfo.pti_virtual_size);
        metrics.resident_memory_bytes = Some(info.ptinfo.pti_resident_size);
        metrics.threads = Some(info.ptinfo.pti_threadnum as u64);
        metrics.open_fds = listpidinfo::<ListFDs>(pid, info.pbsd.pbi_nfiles as usize)
            .ok()
            .map(|v| v.len() as u64);
    }
    metrics.virtual_memory_max_bytes = getrlimit(Resource::AS).ok().map(|(soft, _hard)| soft);
    metrics.max_fds = getrlimit(Resource::NOFILE).ok().map(|(soft, _hard)| soft);
    metrics
}
