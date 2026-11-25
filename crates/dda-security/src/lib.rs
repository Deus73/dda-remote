use anyhow::{Result, anyhow};
use rustls::{ClientConfig, ServerConfig, RootCertStore, Certificate};
use std::sync::Arc;
use p256::{P256, SecretKey, PublicKey};
use totp_rs::TOTP;
use rand::thread_rng;
use std::fs;
use std::path::Path;

// Implementatie van het genereren van een cryptografische apparaat-ID
pub fn generate_device_identity() -> anyhow::Result<String> {
    let secret = SecretKey::random(&mut thread_rng());
    let public_key = secret.public_key();
    // Returneer de hex-gecodeerde publieke sleutel als ID
    Ok(public_key.to_encoded_point().to_string())
}

// Implementatie van TOTP-verificatie (RFC 6238)
pub fn verify_totp(secret: &str, code: &str) -> bool {
    // Initialiseer de TOTP met de secret en verifieer de code
    // Gebruikt standaard parameters zoals SHA1 en 30-seconden window.
    TOTP::new(secret).verify(code).is_ok()
}

// Laadt certificaten en configureert de rustls server/client
pub fn get_tls_config(is_server: bool) -> anyhow::Result<rustls::ClientConfig> {
    // Zoek certificaten in de /certs map (relatief aan projectroot)
    let cert_path = Path::new("dda-remote/DDA-Remote-v0.1.0-source/dda-remote/certs");
    let cert_file = cert_path.join("server.crt");
    let key_file = cert_path.join("server.key");

    let cert_content = fs::read_to_string(cert_file)?;
    let key_content = fs::read_to_string(key_file)?;
    
    let cert = Certificate(cert_content.into_bytes());
    
    // Gebruik ServerConfig om de sleutel en certificaat in te laden
    let server_config = ServerConfig::builder()
        .with_safe_patterns(true)
        .with_no_client_auth() 
        .with_single_cert(vec![cert], key_content.into_bytes())?;

    // De functie moet ClientConfig retourneren conform het oorspronkelijke API
    Ok(ClientConfig::builder().with_root_certificates(RootCertStore::empty()).with_no_client_auth().into())
}