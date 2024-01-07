use std::fs;
use std::path::Path;

const KEY_FILE: &str = "session_key.txt";

pub fn load_or_create_key() -> Vec<u8> {
    if Path::new(KEY_FILE).exists() {
        // Load the existing key
        fs::read(KEY_FILE).expect("Failed to read key file")
    } else {
        // Create a new key
        let key: [u8; 32] = rand::random();
        fs::write(KEY_FILE, &key).expect("Failed to write key file");
        key.to_vec()
    }
}
