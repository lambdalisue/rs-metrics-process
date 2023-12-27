# with_axum

An example of `metrics-process` crate with [axum] and [metrics-exporter-prometheus].

[axum]: https://crates.io/crates/axum
[metrics-exporter-prometheus]: https://crates.io/crates/metrics-exporter-prometheus

## Usage

Start the server with the following command

```
cargo run --release
```

Then access to http://localhost:9000/metrics
