use criterion::{black_box, criterion_group, criterion_main, Criterion};
use homomorphic_llm_proxy::config::Config;
use homomorphic_llm_proxy::proxy::ProxyServer;
use std::time::Duration;
use tokio::runtime::Runtime;

fn bench_server_creation(c: &mut Criterion) {
    c.bench_function("proxy_server_creation", |b| {
        b.iter(|| {
            let config = black_box(Config::default());
            black_box(ProxyServer::new(config).unwrap())
        })
    });
}

fn bench_config_loading(c: &mut Criterion) {
    c.bench_function("config_loading", |b| {
        b.iter(|| {
            black_box(Config::default())
        })
    });
}

fn bench_config_validation(c: &mut Criterion) {
    let config = Config::default();
    
    c.bench_function("config_validation", |b| {
        b.iter(|| {
            black_box(config.validate().unwrap())
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(5))
        .sample_size(30);
    targets = bench_server_creation, bench_config_loading, bench_config_validation
);
criterion_main!(benches);