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

