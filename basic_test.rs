//! Basic functionality test for FHE LLM Proxy

use homomorphic_llm_proxy::fhe::{FheEngine, FheParams};
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Testing FHE LLM Proxy Core Functionality");

    // Create FHE engine
    let params = FheParams::default();
    let mut engine = FheEngine::new(params)?;
    
    println!("✅ Created FHE engine with security level {}", engine.get_params().security_level);

    // Generate keys
    let (client_id, server_id) = engine.generate_keys()?;
    println!("✅ Generated key pair: client={}, server={}", client_id, server_id);

    // Test encryption
    let plaintext = "Hello, FHE World! This is a test of homomorphic encryption.";
    let ciphertext = engine.encrypt_text(client_id, plaintext)?;
    println!("✅ Encrypted {} characters, noise budget: {:?}", 
             plaintext.len(), ciphertext.noise_budget);

    // Test decryption
    let decrypted = engine.decrypt_text(client_id, &ciphertext)?;
    println!("✅ Decrypted: {}", decrypted);

    // Verify roundtrip
    assert_eq!(plaintext, decrypted);
    println!("✅ Encryption/decryption roundtrip successful!");

    // Test ciphertext validation
    let is_valid = engine.validate_ciphertext(&ciphertext)?;
    println!("✅ Ciphertext validation: {}", is_valid);

    // Get engine stats
    let stats = engine.get_stats();
    println!("✅ Engine stats: {} client keys, {} server keys", 
             stats.total_client_keys, stats.total_server_keys);

    println!("🎉 All basic FHE functionality tests passed!");
    
    Ok(())
}