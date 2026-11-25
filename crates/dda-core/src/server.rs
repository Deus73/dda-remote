use tokio::net::UdpSocket;
use quinn::{Endpoint, Incoming, RustlsConfig};
use anyhow::Result;
use dda_core::{Stream, SecureStream};
use dda_security::{generate_device_identity, verify_totp, get_tls_config}; // Import security components
use dda_protocol::{Message, decode, encode}; // Import protocol definition

// Server Configuration (will be loaded from config/env in production)
const SERVER_ADDR: &str = "0.0.0.0:8500";

/// Initialiseert de RustlsConfig op basis van de serverconfiguratie.
fn create_rustls_config() -> Result<RustlsConfig> {
    // Gebruik de implementatie uit dda-security
    let config = get_tls_config(true)?;
    Ok(RustlsConfig::from_server_config(config)?)
}

/// Hoofdserverfunctie die de QUIC endpoint opzet.
pub async fn run_server() -> Result<()> {
    let rustls_config = create_rustls_config()?;
    let endpoint = Endpoint::server(SERVER_ADDR, rustls_config)?;

    println!("DDA Remote Server draait op {}", SERVER_ADDR);

    loop {
        match endpoint.accept().await {
            Ok(incoming) => {
                println!("Nieuwe client verbinding geaccepteerd. Start handshake...");
                tokio::spawn(handle_connection(incoming));
            }
            Err(e) => {
                eprintln!("Fout bij het accepteren van verbinding: {:?}", e);
                // Blijf draaien, maar log het fout
            }
        }
    }
}

/// Behandelt een individuele verbinding met handshake en secure stream.
async fn handle_connection(incoming: Incoming) {
    // 1. Handshake & Authenticatie Simulatie
    // TODO: In production: Dit is waar de ECDH/TOTP handshake plaatsvindt met de client.
    
    // Simuleer sleutelgeneratie op de server
    match generate_device_identity() {
        Ok(device_id) => println!("Server Device ID gegenereerd: {}", device_id),
        Err(e) => {
            eprintln!("Fout bij sleutelgeneratie: {:?}", e);
            return;
        }
    }
    
    // Simuleer authenticatie (e.g., client stuurt secret/code)
    let client_secret = "some_base32_secret"; // Vanaf config/handshake
    let client_code = "123456"; // Vanaf config/handshake
    if !verify_totp(client_secret, client_code) {
        eprintln!("AUTHENTICATIE MISLUKT: TOTP-validatie faalt.");
        return;
    }
    println!("AUTHENTICATIE SUCCESVOL. Sessie opgebouwd.");
    
    // Simuleer afgeleide sessiesleutel uit ECDH (Moet in productie afkomstig zijn van de handshake)
    let derived_session_key = [0xAA; 32]; 

    // 2. Handshake afronden
    let (mut client_stream, _) = incoming.accept().await.expect("Handshake mislukt");

    // 3. SecureStream initialiseren
    let dummy_inner_stream = UdpSocket::bind("0.0.0.0:0").await.unwrap(); // Dummy stream
    let mut secure_stream = SecureStream::new(dummy_inner_stream, client_stream, derived_session_key);

    println!("Verbinding beveiligd. Start message exchange.");

    // 4. Message exchange loop
    while let Ok(message) = secure_stream.receive_secure::<Message>() {
        println!("Ontvangen bericht: {:?}", message);
        // TODO: Verwerk het bericht en stuur een antwoord terug
        // secure_stream.send_secure(&Message)?;
    }
    println!("Verbinding gesloten.");
}