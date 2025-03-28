//passwd.rs

use linux_keyutils::{KeyRing, Key, KeyRingIdentifier, KeyError};
use std::sync::{Mutex, Arc};
use sha2::{Sha256, Digest};
use std::error::Error as StdError;
use std::fmt;

// Create a wrapper around KeyError to make it compatible with StdError
#[derive(Debug)]
struct KeyErrorWrapper(KeyError);

impl fmt::Display for KeyErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key error: {:?}", self.0)
    }
}

impl StdError for KeyErrorWrapper {}

// Helper function to convert KeyError to our wrapper
fn wrap_key_error(err: KeyError) -> Box<dyn StdError> {
    Box::new(KeyErrorWrapper(err))
}

lazy_static::lazy_static! {
    /// A static, thread-safe reference to a `PasswordStore` instance, protected by a `Mutex`.
    
    pub static ref PASSWORD_STORAGE: Arc<Mutex<PasswordStore>> = Arc::new(Mutex::new(PasswordStore::default()));
}

/// A structure for storing passwords used in the CLI.
pub struct PasswordStore {
    pub enable_password: Option<String>,
    pub enable_secret: Option<String>,
}

impl Default for PasswordStore {
    /// Creates a new instance of `PasswordStore` with default values.
    ///
    /// Both `enable_password` and `enable_secret` are initialized to `None`
    fn default() -> Self {
        PasswordStore {
            enable_password: None,
            enable_secret: None,
        }
    }
}


const PASSWORD_KEY: &str = "cli_enable_password";
const SECRET_KEY: &str = "cli_enable_secret";

/// Store a password in the linux keyring
fn store_key(name: &str, value: &str) -> Result<(), Box<dyn StdError>> {
    let session_keyring = KeyRing::from_special_id(KeyRingIdentifier::Session, true)
        .map_err(wrap_key_error)?;
    session_keyring.add_key(name, value.as_bytes())
        .map_err(wrap_key_error)?;
    Ok(())
}

/// Retrieve a password from the linux keyring
fn retrieve_key(name: &str) -> Option<String> {
    if let Ok(session_keyring) = KeyRing::from_special_id(KeyRingIdentifier::Session, false) {
        if let Ok(key) = session_keyring.search(name) {
            // Create a buffer to read into
            let mut buffer = vec![0u8; 1024]; // Adjust size as needed
            
            // Read the key data into the buffer
            if let Ok(bytes_read) = key.read(&mut buffer) {
                // Resize buffer to actual bytes read
                buffer.truncate(bytes_read);
                
                // Convert buffer to string
                return String::from_utf8(buffer).ok();
            }
        }
    }
    None
}

/// Set the enable password and store it in the keyring
pub fn set_enable_password(password: &str) {
    let _ = store_key(PASSWORD_KEY, password);
    let mut storage = PASSWORD_STORAGE.lock().unwrap();
    storage.enable_password = Some(password.to_string());
}

/// Set the enable secret and store it in the keyring
pub fn set_enable_secret(secret: &str) {
    let _ = store_key(SECRET_KEY, secret);
    let mut storage = PASSWORD_STORAGE.lock().unwrap();
    storage.enable_secret = Some(secret.to_string());
}

/// Get the enable password from the keyring
pub fn get_enable_password() -> Option<String> {
    if let Some(password) = retrieve_key(PASSWORD_KEY) {
        let mut storage = PASSWORD_STORAGE.lock().unwrap();
        storage.enable_password = Some(password.clone());
        return Some(password);
    }
    None
}

/// Get the enable secret from the keyring
pub fn get_enable_secret() -> Option<String> {
    if let Some(secret) = retrieve_key(SECRET_KEY) {
        let mut storage = PASSWORD_STORAGE.lock().unwrap();
        storage.enable_secret = Some(secret.clone());
        return Some(secret);
    }
    None
}

/// Encrypts a password using the SHA-256 hashing algorithm.
pub fn encrypt_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password);
    let result = hasher.finalize();
    format!("{:x}", result)  
}