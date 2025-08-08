use rand::RngCore;

pub fn generate_encryption_key() -> Vec<u8> {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key.to_vec()
}

pub fn key_to_hex_string(key: &[u8]) -> String {
    key.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn generate_network_password(hostname: &str, key: &[u8]) -> String {
    let hex_key = key_to_hex_string(key);
    let hostname_prefix = if hostname.len() >= 2 {
        &hostname[..2]
    } else {
        hostname
    };
    let key_suffix = if hex_key.len() >= 10 {
        &hex_key[hex_key.len()-10..]
    } else {
        &hex_key
    };
    
    format!("fling-{}-{}", hostname_prefix, key_suffix)
}

use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};

pub fn encrypt_file_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Convert key to proper format
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    
    // Generate random nonce
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Encrypt the data
    let ciphertext = match cipher.encrypt(nonce, data) {
        Ok(ct) => ct,
        Err(e) => return Err(format!("Encryption failed: {:?}", e).into()),
    };
    // Prepend nonce to ciphertext (needed for decryption)
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_file_data(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if encrypted_data.len() < 12 {
        return Err("Invalid encrypted data".into());
    }
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    
    // Decrypt
    let plaintext = match cipher.decrypt(nonce, ciphertext) {
        Ok(pt) => pt,
        Err(e) => return Err(format!("Decryption failed: {:?}", e).into()),
    };

    Ok(plaintext)
}