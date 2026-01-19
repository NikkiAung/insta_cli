//! RSA encryption for secure credential transmission

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::rngs::OsRng;
use rsa::{pkcs8::DecodePublicKey, sha2::Sha256, Oaep, RsaPublicKey};

/// Encrypt a password using the server's RSA public key
///
/// Uses RSA-OAEP with SHA-256 padding, matching the server's decryption
pub fn encrypt_password(password: &str, public_key_pem: &str) -> Result<String> {
    // Parse the PEM-encoded public key
    let public_key = RsaPublicKey::from_public_key_pem(public_key_pem)
        .context("Failed to parse server's public key")?;

    // Encrypt using OAEP padding with SHA-256
    let mut rng = OsRng;
    let padding = Oaep::new::<Sha256>();

    let encrypted = public_key
        .encrypt(&mut rng, padding, password.as_bytes())
        .context("Failed to encrypt password")?;

    // Encode as base64 for transmission
    Ok(STANDARD.encode(&encrypted))
}

