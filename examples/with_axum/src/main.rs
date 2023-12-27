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
            // Collect information just before handle '/metrics'
            collector.collect();
            std::future::ready(handle.render())
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:9000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
