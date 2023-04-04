use std::mem::{size_of, MaybeUninit};
use windows::{Win32::Foundation::*, Win32::System::ProcessStatus::*, Win32::System::Threading::*};

use super::Metrics;

/// Collect metrics.
///
/// Refer: / https://github.com/prometheus/client_golang/blob/c7aa2a5b843527449adb99ad113fe14ed15e4eb0/prometheus/process_collector_windows.go#L81-L116
///
// Copyright 2019 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
pub fn collect() -> Metrics {
    let mut metrics = Metrics::default();
    unsafe {
        let h = GetCurrentProcess();
        let (start_time_seconds, cpu_seconds_total) = {
            let mut creationtime = MaybeUninit::uninit();
            let mut _exittime = MaybeUninit::uninit();
            let mut kerneltime = MaybeUninit::uninit();
            let mut usertime = MaybeUninit::uninit();
            let ret = GetProcessTimes(
                h,
                creationtime.as_mut_ptr(),
                _exittime.as_mut_ptr(),
                kerneltime.as_mut_ptr(),
                usertime.as_mut_ptr(),
            );
            if ret.as_bool() {
                // `creationtime` and `_exittime` are points in time expressed as the amount of time that
                // has elapsed since midnight on January 1, 1601 in 100 nanosecond time units.
                let start_time_seconds =
                    filetime_to_unix_epoch_in_seconds(creationtime.assume_init());
                // `kerneltime` and `usertime` are amounts of time in 100 nanosecond time units.
                let cpu_seconds_total = {
                    let stime = filetime_to_seconds(kerneltime.assume_init());
                    let utime = filetime_to_seconds(usertime.assume_init());
                    stime + utime
                };
                (Some(start_time_seconds as u64), Some(cpu_seconds_total))
            } else {
                (None, None)
            }
        };
        metrics.start_time_seconds = start_time_seconds;
        metrics.cpu_seconds_total = cpu_seconds_total;

        let (virtual_memory_bytes, resident_memory_bytes) = {
            let memcounters = &PROCESS_MEMORY_COUNTERS_EX::default();
            let memcounters = memcounters as *const _ as *mut PROCESS_MEMORY_COUNTERS;
            let memcounters = &mut *memcounters;
            let cb = size_of::<PROCESS_MEMORY_COUNTERS_EX>();
            let ret = GetProcessMemoryInfo(h, memcounters, cb as u32);
            if ret.as_bool() {
                let memcounters = memcounters as *const _ as *const PROCESS_MEMORY_COUNTERS_EX;
                let &memcounters = &*memcounters;
                (
                    Some(memcounters.PrivateUsage as u64),
                    Some(memcounters.WorkingSetSize as u64),
                )
            } else {
                (None, None)
            }
        };
        metrics.virtual_memory_bytes = virtual_memory_bytes;
        metrics.resident_memory_bytes = resident_memory_bytes;

        let open_fds = {
            let mut handlecount = 0;
            let ret = GetProcessHandleCount(h, &mut handlecount);
            if ret.as_bool() {
                Some(handlecount as u64)
            } else {
                None
            }
        };
        metrics.open_fds = open_fds;
        metrics.max_fds = Some(16 * 1024 * 1024); // Windows has a hard-coded max limit, not per-process.
    }
    metrics
}

/// Convert FILETIME to seconds.
///
/// FILETIME contains a 64-bit value representing the number of 100-nanosecond intervals.
///
/// https://docs.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-filetime
fn filetime_to_seconds(ft: FILETIME) -> f64 {
    // 100-nanosecond intervals since January 1, 1601
    let low = ft.dwLowDateTime as u64;
    let high = ft.dwHighDateTime as u64;
    let nsec = high.checked_shl(32).unwrap_or(0) + low;
    // convert into nanoseconds
    let nsec = nsec * 100;
    // convert into seconds
    nsec as f64 / 1e9
}

/// Convert FILETIME to Unix Epoch in seconds.
///
/// FILETIME contains a 64-bit value representing the number of 100-nanosecond intervals
/// since January 1, 1601 (UTC).
///
/// https://docs.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-filetime
/// https://cs.opensource.google/go/x/sys/+/2296e014:windows/types_windows.go;l=792-800
///
// Copyright (c) 2009 The Go Authors. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//    * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//    * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//    * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//
fn filetime_to_unix_epoch_in_seconds(ft: FILETIME) -> f64 {
    let low = ft.dwLowDateTime as u64;
    let high = ft.dwHighDateTime as u64;
    let nsec = high.checked_shl(32).unwrap_or(0) + low;
    // change starting time to the epoch (00:00:00 UTC, January 1, 1970)
    let nsec = nsec - 116444736000000000;
    // convert into nanoseconds
    let nsec = nsec * 100;
    // convert into seconds
    nsec as f64 / 1e9
}
