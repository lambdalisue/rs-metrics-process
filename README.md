[![crates.io](https://img.shields.io/crates/v/metrics-process.svg)](https://crates.io/crates/metrics-process)
[![docs.rs](https://docs.rs/metrics-process/badge.svg)](https://docs.rs/metrics-process)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Build](https://github.com/lambdalisue/rs-metrics-process/actions/workflows/build.yml/badge.svg)](https://github.com/lambdalisue/rs-metrics-process/actions/workflows/build.yml)
[![Test](https://github.com/lambdalisue/rs-metrics-process/actions/workflows/test.yml/badge.svg)](https://github.com/lambdalisue/rs-metrics-process/actions/workflows/test.yml)
[![Audit](https://github.com/lambdalisue/rs-metrics-process/actions/workflows/audit.yml/badge.svg)](https://github.com/lambdalisue/rs-metrics-process/actions/workflows/audit.yml)

# ⏱ metrics-process

This crate provides a [Prometheus]-style [process metrics] collector for the
[metrics] crate, supporting Linux, macOS, and Windows. The collector code is
manually rewritten in Rust from the official Prometheus client for Go
([client_golang]).

[Prometheus]: https://prometheus.io/
[process metrics]: https://prometheus.io/docs/instrumenting/writing_clientlibs/#process-metrics
[metrics]: https://crates.io/crates/metrics

## Supported Metrics

This crate supports the following metrics provided by [Prometheus] for
[process metrics].

| Metric name                        | Help string                                                |
| ---------------------------------- | ---------------------------------------------------------- |
| `process_cpu_seconds_total`        | Total user and system CPU time spent in seconds.           |
| `process_open_fds`                 | Number of open file descriptors.                           |
| `process_max_fds`                  | Maximum number of open file descriptors.                   |
| `process_virtual_memory_bytes`     | Virtual memory size in bytes.                              |
| `process_virtual_memory_max_bytes` | Maximum amount of virtual memory available in bytes.       |
| `process_resident_memory_bytes`    | Resident memory size in bytes.                             |
| `process_heap_bytes`               | Process heap size in bytes.                                |
| `process_start_time_seconds`       | Start time of the process since the Unix epoch in seconds. |
| `process_threads`                  | Number of OS threads in the process.                       |

For each platform, it is equivalent to what the official Prometheus client for
Go ([client_golang]) provides.

| Metric name                        | Linux | macOS | Windows |
| ---------------------------------- | ----- | ----- | ------- |
| `process_cpu_seconds_total`        | x     | x     | x       |
| `process_open_fds`                 | x     | x     | x       |
| `process_max_fds`                  | x     | x     | x       |
| `process_virtual_memory_bytes`     | x     | x     | x       |
| `process_virtual_memory_max_bytes` | x     | x     |         |
| `process_resident_memory_bytes`    | x     | x     | x       |
| `process_heap_bytes`               |       |       |         |
| `process_start_time_seconds`       | x     | x     | x       |
| `process_threads`                  | x     | x     |         |

[client_golang]: https://github.com/prometheus/client_golang

## Usage

Use this crate with [metrics-exporter-prometheus] as an exporter like:

[metrics-exporter-prometheus]: https://crates.io/crates/metrics-exporter-prometheus

```rust,no_run
use std::thread;
use std::time::{Duration, Instant};

use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_process::Collector;

let builder = PrometheusBuilder::new();
builder
    .install()
    .expect("failed to install Prometheus recorder");

let collector = Collector::default();
// Call `describe()` method to register help string.
collector.describe();

loop {
    let s = Instant::now();
    // Periodically call `collect()` method to update information.
    collector.collect();
    thread::sleep(Duration::from_millis(750));
}
```

Or with [axum] (or any web application framework you like) to collect metrics
whenever the `/metrics` endpoint is invoked like:

[axum]: https://crates.io/crates/axum

```rust,no_run
use axum::{routing::get, Router};
use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_process::Collector;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let builder = PrometheusBuilder::new();
    let handle = builder
        .install_recorder()
        .expect("failed to install Prometheus recorder");

    let collector = Collector::default();
    // Call `describe()` method to register help string.
    collector.describe();

    let app = Router::new().route(
        "/metrics",
        get(move || {
            // Collect information just before handling '/metrics'
            collector.collect();
            std::future::ready(handle.render())
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

## Difference from [metrics-process-promstyle]

It appears that [metrics-process-promstyle] only supports Linux, but this crate
(metrics-process) supports Linux, macOS, and Windows. Additionally, this crate
supports `process_open_fds` and `process_max_fds` in addition to what
metrics-process-promstyle supports.

[metrics-process-promstyle]: https://crates.io/crates/metrics-process-promstyle

# License

The code follows the MIT license written in [LICENSE](./LICENSE). Contributors
need to agree that any modifications sent to this repository follow the license.
