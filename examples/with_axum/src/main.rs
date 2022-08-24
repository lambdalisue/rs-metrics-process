use axum::{routing::get, Router, Server};
use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_process::Collector;

#[tokio::main]
async fn main() {
    let builder = PrometheusBuilder::new();
    let handle = builder
        .install_recorder()
        .expect("failed to install Prometheus recorder");

    let collector = Collector::default();
    // Call `describe()` method to register help string.
    collector.describe();

    let addr = "127.0.0.1:9000".parse().unwrap();
    let app = Router::new().route(
        "/metrics",
        get(move || {
            // Collect information just before handle '/metrics'
            collector.collect();
            std::future::ready(handle.render())
        }),
    );
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
