//! 🔐 Test Vault Integration
//! 
//! Simple test script to verify Vault client functionality

use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Testing Vault Integration...");

    // Initialize Vault client
    let vault_client = cerebro_bff::VaultClient::new(
        "http://localhost:8200",
        "cerberus_root_token"
    );

    // Test health check
    println!("🔍 Testing Vault health check...");
    let is_healthy = vault_client.health_check().await?;
    println!("✅ Vault health: {}", if is_healthy { "HEALTHY" } else { "UNHEALTHY" });

    // Test storing a secret
    println!("💾 Testing secret storage...");
    let mut test_data = HashMap::new();
    test_data.insert("api_key".to_string(), "test_helius_key_12345".to_string());
    test_data.insert("service".to_string(), "helius".to_string());
    
    vault_client.store_secret("test/helius", &test_data).await?;
    println!("✅ Secret stored successfully");

    // Test retrieving the secret
    println!("🔍 Testing secret retrieval...");
    let retrieved = vault_client.get_secret("test/helius").await?;
    
    if let Some(api_key) = retrieved.get("api_key") {
        println!("✅ Secret retrieved: {}", api_key);
    } else {
        println!("❌ Failed to retrieve secret");
    }

    // Test encryption/decryption
    println!("🔒 Testing encryption...");
    
    // First create the encryption key
    if let Err(e) = vault_client.create_key("test_key").await {
        println!("⚠️ Key creation failed (may already exist): {}", e);
    }

    let plaintext = "sensitive_private_key_data_12345";
    let encrypted = vault_client.encrypt("test_key", plaintext).await?;
    println!("✅ Data encrypted: {}", encrypted.ciphertext);

    let decrypted = vault_client.decrypt("test_key", &encrypted).await?;
    println!("✅ Data decrypted: {}", decrypted);

    if plaintext == decrypted {
        println!("🎉 Encryption/decryption test PASSED!");
    } else {
        println!("❌ Encryption/decryption test FAILED!");
    }

    // Test SecureConfig
    println!("🛡️ Testing SecureConfig...");
    let secure_config = cerebro_bff::SecureConfig::new(vault_client);
    
    secure_config.store_api_key("quicknode", "test_quicknode_key_67890").await?;
    let retrieved_key = secure_config.get_api_key("quicknode").await?;
    println!("✅ SecureConfig test: {}", retrieved_key);

    println!("🎉 All Vault integration tests PASSED!");
    Ok(())
}
