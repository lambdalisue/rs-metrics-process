use rand::{thread_rng, Rng};
use std::thread;
use std::time::{Duration, Instant};

use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_process::Collector;

fn main() {
    let builder = PrometheusBuilder::new();
    builder
        .install()
        .expect("failed to install Prometheus recorder");

    let collector = Collector::default();
    collector.describe();

    let mut rng = thread_rng();

    loop {
        let s = Instant::now();
        let n: u64 = rng.gen_range(0..40);
        println!(
            "fibonacci({}) = {} ({} ns)",
            n,
            fibonacci(n),
            s.elapsed().as_nanos()
        );
        collector.collect();
        thread::sleep(Duration::from_millis(750));
    }
}

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 2) + fibonacci(n - 1),
    }
}
