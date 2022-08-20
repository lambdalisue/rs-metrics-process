use std::thread;
use std::time::Duration;

use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_process::Collector;

fn main() {
    let builder = PrometheusBuilder::new();
    builder
        .install()
        .expect("failed to install Prometheus recorder");

    let collector = Collector::new("");
    collector.describe();

    loop {
        collector.collect();
        thread::sleep(Duration::from_millis(750));
    }
}
