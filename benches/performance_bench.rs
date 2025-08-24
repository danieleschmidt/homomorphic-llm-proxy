//! Performance benchmarks for Homomorphic LLM Proxy
//!
//! Comprehensive benchmarking suite to validate performance targets

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use homomorphic_llm_proxy::fhe::{FheEngine, FheParams};
use std::time::Duration;
use uuid::Uuid;

fn bench_fhe_key_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("fhe_key_generation");

    for security_level in [128, 192, 256].iter() {
        group.bench_with_input(
            BenchmarkId::new("security_level", security_level),
            security_level,
            |b, &security_level| {
                b.iter(|| {
                    let mut params = FheParams::default();
                    params.security_level = security_level;
                    let mut engine = FheEngine::new(params).expect("Failed to create engine");
                    black_box(engine.generate_keys().expect("Failed to generate keys"));
                });
            },
        );
    }
    group.finish();
}

fn bench_fhe_encryption(c: &mut Criterion) {
    let mut group = c.benchmark_group("fhe_encryption");
    group.measurement_time(Duration::from_secs(10));

    // Prepare engine and keys
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create engine");
    let (client_id, _) = engine.generate_keys().expect("Failed to generate keys");

    for text_length in [10, 50, 100, 500, 1000].iter() {
        let text = "x".repeat(*text_length);
        group.bench_with_input(
            BenchmarkId::new("text_length", text_length),
            &text,
            |b, text| {
                b.iter(|| {
                    black_box(
                        engine
                            .encrypt_text(client_id, text)
                            .expect("Failed to encrypt"),
                    );
                });
            },
        );
    }
    group.finish();
}

fn bench_fhe_decryption(c: &mut Criterion) {
    let mut group = c.benchmark_group("fhe_decryption");
    group.measurement_time(Duration::from_secs(10));

    // Prepare engine, keys, and ciphertexts
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create engine");
    let (client_id, _) = engine.generate_keys().expect("Failed to generate keys");

    for text_length in [10, 50, 100, 500, 1000].iter() {
        let text = "x".repeat(*text_length);
        let ciphertext = engine
            .encrypt_text(client_id, &text)
            .expect("Failed to encrypt");

        group.bench_with_input(
            BenchmarkId::new("text_length", text_length),
            &ciphertext,
            |b, ct| {
                b.iter(|| {
                    black_box(
                        engine
                            .decrypt_text(client_id, ct)
                            .expect("Failed to decrypt"),
                    );
                });
            },
        );
    }
    group.finish();
}

fn bench_validation_framework(c: &mut Criterion) {
    use homomorphic_llm_proxy::validation::ValidationFramework;

    let framework = ValidationFramework::with_fhe_defaults().expect("Failed to create framework");

    c.bench_function("input_validation", |b| {
        b.iter(|| {
            let result = framework.validate_input(
                "test_input",
                black_box("This is a test input for validation performance testing"),
            );
            black_box(result);
        });
    });

    c.bench_function("security_threat_detection", |b| {
        b.iter(|| {
            let threats = framework.detect_security_threats(black_box(
                "<script>alert('test')</script>'; DROP TABLE users; --",
            ));
            black_box(threats);
        });
    });

    c.bench_function("uuid_validation", |b| {
        b.iter(|| {
            let result = framework.validate_uuid(black_box("550e8400-e29b-41d4-a716-446655440000"));
            black_box(result);
        });
    });
}

fn bench_concurrent_fhe_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_fhe");
    group.measurement_time(Duration::from_secs(15));

    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create engine");

    // Pre-generate multiple key pairs
    let mut key_pairs = Vec::new();
    for _ in 0..10 {
        let keys = engine.generate_keys().expect("Failed to generate keys");
        key_pairs.push(keys);
    }

    for num_operations in [1, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_operations", num_operations),
            num_operations,
            |b, &num_ops| {
                b.iter(|| {
                    for i in 0..num_ops {
                        let (client_id, _) = key_pairs[i % key_pairs.len()];
                        let text = format!("Test message {}", i);
                        let ciphertext = engine
                            .encrypt_text(client_id, &text)
                            .expect("Failed to encrypt");
                        let _decrypted = engine
                            .decrypt_text(client_id, &ciphertext)
                            .expect("Failed to decrypt");
                    }
                });
            },
        );
    }
    group.finish();
}

fn bench_memory_intensive_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_intensive");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10); // Reduce sample size for memory-intensive tests

    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create engine");

    // Test memory allocation patterns
    group.bench_function("multiple_key_generation", |b| {
        b.iter(|| {
            let mut keys = Vec::new();
            for _ in 0..5 {
                keys.push(engine.generate_keys().expect("Failed to generate keys"));
            }
            black_box(keys);
        });
    });

    group.bench_function("large_batch_encryption", |b| {
        let (client_id, _) = engine.generate_keys().expect("Failed to generate keys");
        b.iter(|| {
            let mut ciphertexts = Vec::new();
            for i in 0..10 {
                let text = format!("Batch encryption test message number {}", i);
                let ct = engine
                    .encrypt_text(client_id, &text)
                    .expect("Failed to encrypt");
                ciphertexts.push(ct);
            }
            black_box(ciphertexts);
        });
    });

    group.finish();
}

fn bench_edge_case_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("edge_cases");

    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create engine");
    let (client_id, _) = engine.generate_keys().expect("Failed to generate keys");

    // Test minimum input
    group.bench_function("minimum_input", |b| {
        b.iter(|| {
            let ciphertext = engine
                .encrypt_text(client_id, black_box("a"))
                .expect("Failed to encrypt");
            let _decrypted = engine
                .decrypt_text(client_id, &ciphertext)
                .expect("Failed to decrypt");
        });
    });

    // Test maximum supported input
    group.bench_function("maximum_input", |b| {
        let max_text = "x".repeat(10000); // Adjust based on actual limits
        b.iter(|| {
            let ciphertext = engine
                .encrypt_text(client_id, black_box(&max_text))
                .expect("Failed to encrypt");
            let _decrypted = engine
                .decrypt_text(client_id, &ciphertext)
                .expect("Failed to decrypt");
        });
    });

    // Test special characters
    group.bench_function("special_characters", |b| {
        let special_text = "!@#$%^&*()_+-=[]{}|;:,.<>?/~`";
        b.iter(|| {
            let ciphertext = engine
                .encrypt_text(client_id, black_box(special_text))
                .expect("Failed to encrypt");
            let _decrypted = engine
                .decrypt_text(client_id, &ciphertext)
                .expect("Failed to decrypt");
        });
    });

    // Test Unicode
    group.bench_function("unicode_text", |b| {
        let unicode_text = "Hello 世界 🌍 Здравствуй мир";
        b.iter(|| {
            let ciphertext = engine
                .encrypt_text(client_id, black_box(unicode_text))
                .expect("Failed to encrypt");
            let _decrypted = engine
                .decrypt_text(client_id, &ciphertext)
                .expect("Failed to decrypt");
        });
    });

    group.finish();
}

fn bench_throughput_test(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.measurement_time(Duration::from_secs(30));
    group.throughput(criterion::Throughput::Elements(100));

    let params = FheParams::default();
    let mut engine = FheEngine::new(params).expect("Failed to create engine");
    let (client_id, _) = engine.generate_keys().expect("Failed to generate keys");

    group.bench_function("operations_per_second", |b| {
        b.iter(|| {
            for i in 0..100 {
                let text = format!("Throughput test {}", i);
                let ciphertext = engine
                    .encrypt_text(client_id, &text)
                    .expect("Failed to encrypt");
                let _decrypted = engine
                    .decrypt_text(client_id, &ciphertext)
                    .expect("Failed to decrypt");
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_fhe_key_generation,
    bench_fhe_encryption,
    bench_fhe_decryption,
    bench_validation_framework,
    bench_concurrent_fhe_operations,
    bench_memory_intensive_operations,
    bench_edge_case_performance,
    bench_throughput_test
);

criterion_main!(benches);
