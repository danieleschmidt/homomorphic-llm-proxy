use criterion::{black_box, criterion_group, criterion_main, Criterion};
use homomorphic_llm_proxy::fhe::{FheEngine, FheParams};
use std::time::Duration;

fn bench_key_generation(c: &mut Criterion) {
    let params = FheParams::default();
    
    c.bench_function("fhe_key_generation", |b| {
        b.iter(|| {
            let mut engine = black_box(FheEngine::new(params.clone()).unwrap());
            black_box(engine.generate_keys().unwrap())
        })
    });
}

fn bench_encryption(c: &mut Criterion) {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).unwrap();
    let (client_id, _) = engine.generate_keys().unwrap();
    
    let test_data = "The quick brown fox jumps over the lazy dog";
    
    c.bench_function("fhe_encrypt_text", |b| {
        b.iter(|| {
            engine.encrypt_text(black_box(client_id), black_box(test_data)).unwrap()
        })
    });
}

fn bench_decryption(c: &mut Criterion) {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).unwrap();
    let (client_id, _) = engine.generate_keys().unwrap();
    
    let test_data = "The quick brown fox jumps over the lazy dog";
    let ciphertext = engine.encrypt_text(client_id, test_data).unwrap();
    
    c.bench_function("fhe_decrypt_text", |b| {
        b.iter(|| {
            engine.decrypt_text(black_box(client_id), black_box(&ciphertext)).unwrap()
        })
    });
}

fn bench_homomorphic_ops(c: &mut Criterion) {
    let params = FheParams::default();
    let mut engine = FheEngine::new(params).unwrap();
    let (client_id, _) = engine.generate_keys().unwrap();
    
    let text_a = "Hello ";
    let text_b = "World!";
    let ciphertext_a = engine.encrypt_text(client_id, text_a).unwrap();
    let ciphertext_b = engine.encrypt_text(client_id, text_b).unwrap();
    
    c.bench_function("fhe_concatenate", |b| {
        b.iter(|| {
            engine.concatenate_encrypted(black_box(&ciphertext_a), black_box(&ciphertext_b)).unwrap()
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(50);
    targets = bench_key_generation, bench_encryption, bench_decryption, bench_homomorphic_ops
);
criterion_main!(benches);