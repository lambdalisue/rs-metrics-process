use super::Metrics;

fn getrusage(who: libc::c_int) -> Option<libc::rusage> {
    let mut usage = std::mem::MaybeUninit::zeroed();
    // SAFETY: libc call; usage is valid pointer to rusage struct
    if unsafe { libc::getrusage(who, usage.as_mut_ptr()) } == 0 {
        // SAFETY: libc call was success, struct must be initialized
        Some(unsafe { usage.assume_init() })
    } else {
        None
    }
}

fn getrlimit(resource: libc::c_int) -> Option<libc::rlimit> {
    let mut limit = std::mem::MaybeUninit::zeroed();
    // SAFETY: libc call; limit is valid pointer to rlimit struct
    if unsafe { libc::getrlimit(resource, limit.as_mut_ptr()) } == 0 {
        // SAFETY: libc call was success, struct must be initialized
        Some(unsafe { limit.assume_init() })
    } else {
        None
    }
}

fn translate_rlim(rlim: libc::rlim_t) -> u64 {
    if rlim == libc::RLIM_INFINITY {
        0
    } else {
        rlim as u64
    }
}

fn kinfo_getproc(pid: libc::pid_t) -> Option<libc::kinfo_proc> {
    // References:
    // kinfo_getproc() code from FreeBSD: https://github.com/freebsd/freebsd-src/blob/b22be3bbb2de75157c97d8baa01ce6cd654caddf/lib/libutil/kinfo_getproc.c
    // code from deno doing similar stuff: https://github.com/denoland/deno/blob/20ae8db50d7d48ad020b83ebe78dc0e9e9eab3b2/runtime/ops/os/mod.rs#L415
    let mib = [libc::CTL_KERN, libc::KERN_PROC, libc::KERN_PROC_PID, pid];

    let mut kinfo_proc = std::mem::MaybeUninit::zeroed();
    let kinfo_proc_size = std::mem::size_of_val(&kinfo_proc) as libc::size_t;
    let mut data_size = kinfo_proc_size;

    // SAFETY: libc call; mib is statically initialized, kinfo_proc is valid pointer
    // to kinfo_proc and data_size holds its size
    if unsafe {
        libc::sysctl(
            mib.as_ptr(),
            mib.len() as _,
            kinfo_proc.as_mut_ptr() as *mut libc::c_void,
            &mut data_size,
            std::ptr::null(),
            0,
        )
    } == 0
        && data_size == kinfo_proc_size
    {
        // SAFETY: libc call was success and check for struct size passed, struct must be initialized
        Some(unsafe { kinfo_proc.assume_init() })
    } else {
        None
    }
}

pub fn collect() -> Metrics {
    let mut metrics = Metrics::default();

    if let Some(usage) = getrusage(libc::RUSAGE_SELF) {
        metrics.cpu_seconds_total = Some(
            (usage.ru_utime.tv_sec + usage.ru_stime.tv_sec) as f64
                + (usage.ru_utime.tv_usec + usage.ru_stime.tv_usec) as f64 / 1000000.0,
        );
    }

    if let Some(limit_as) = getrlimit(libc::RLIMIT_AS) {
        metrics.virtual_memory_max_bytes = Some(translate_rlim(limit_as.rlim_cur));
    }

    if let Some(limit_as) = getrlimit(libc::RLIMIT_NOFILE) {
        metrics.max_fds = Some(translate_rlim(limit_as.rlim_cur));
    }

    // SAFETY: libc call
    let pid = unsafe { libc::getpid() };

    if let Some(kinfo_proc) = kinfo_getproc(pid) {
        // struct kinfo_proc layout for reference
        // libc crate: https://docs.rs/libc/latest/x86_64-unknown-freebsd/libc/struct.kinfo_proc.html
        // FreeBSD: https://github.com/freebsd/freebsd-src/blob/b22be3bbb2de75157c97d8baa01ce6cd654caddf/lib/libutil/kinfo_getfile.c

        // SAFETY: libc call
        let pagesize = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64;
        metrics.virtual_memory_bytes = Some(kinfo_proc.ki_size as u64);
        metrics.resident_memory_bytes = Some(kinfo_proc.ki_rssize as u64 * pagesize);
        use std::convert::TryInto as _;
        metrics.start_time_seconds = kinfo_proc.ki_start.tv_sec.try_into().ok();
        metrics.threads = kinfo_proc.ki_numthreads.try_into().ok();

        // note that we can't access pointers in kinfo_proc as these point to kernel space
    }

    // Alternative to this would be implementing kinfo_getfile() like interface, see
    // https://github.com/freebsd/freebsd-src/blob/b22be3bbb2de75157c97d8baa01ce6cd654caddf/lib/libutil/kinfo_getfile.c
    // it can be done similar to kinfo_getproc() implementation above, but is more cumbersome
    // because we have to parse structures of varying size frow raw memory. As long as
    // it's common to read /proc on Linux, it shuld be as ok to read /dev/fs (which
    // is roughly the same as /proc/self/fd) on FreeBSD.
    metrics.open_fds = std::fs::read_dir("/dev/fd")
        .ok()
        .map(|read_dir| read_dir.count() as u64);

    metrics
}
