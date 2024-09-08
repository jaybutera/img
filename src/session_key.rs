use std::fs;
use std::path::Path;
use actix_web::cookie::Key;

const KEY_FILE: &str = "session_key.txt";

pub fn load_or_create_key() -> Key {
    if Path::new(KEY_FILE).exists() {
        // Load the existing key
        let bytes = fs::read(KEY_FILE).expect("Failed to read key file");
        Key::from(&bytes)
    } else {
        // Create a new key
        let key = Key::generate();
        fs::write(KEY_FILE, key.master()).expect("Failed to write key file");
        key
    }
}
