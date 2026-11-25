//! DDA Remote — TCP prototype with TLS (issue #1 step).
//! Still prototype security in other areas; TLS encrypts transport.
use anyhow::{bail, Context, Result};
use dda_core::{receive, send, Stream};
use dda_protocol::{Message, PROTOCOL_VERSION};
use enigo::{Button, Coordinate, Direction, Enigo, Keyboard, Mouse, Settings};
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageBuffer, Rgba};
use rustls::client::{ServerCertVerified, ServerCertVerifier};
use rustls::{Certificate, ClientConfig, PrivateKey, ServerConfig, ServerConnection, ClientConnection, StreamOwned};
use rustls_pemfile::{certs, pkcs8_private_keys};
use screenshots::Screen;
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct NoCertificateVerification;

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

fn load_certs(path: &str) -> Result<Vec<Certificate>> {
    let f = File::open(path).with_context(|| format!("open certs: {}", path))?;
    let mut reader = BufReader::new(f);
    let certs = certs(&mut reader)?;
    Ok(certs.into_iter().map(Certificate).collect())
}

fn load_private_key(path: &str) -> Result<PrivateKey> {
    let f = File::open(path).with_context(|| format!("open key: {}", path))?;
    let mut reader = BufReader::new(f);
    let keys = pkcs8_private_keys(&mut reader)?;
    if keys.is_empty() {
        anyhow::bail!("no PKCS#8 private key found in key file");
    }
    Ok(PrivateKey(keys[0].clone()))
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("host") => {
            let port = args.get(2).map(String::as_str).unwrap_or("45890").parse::<u16>()?;
            host(port)
        }
        Some("connect") => {
            let target = args.get(2).context("usage: dda-remote connect 127.0.0.1:45890")?;
            connect(target)
        }
        _ => {
            println!("DDA Remote 0.1.0 | TCP+TLS prototype (issue #1 in progress)");
            println!("  dda-remote host [port]");
            println!("  dda-remote connect <ip:port>");
            println!("WARNING: Prototype; use only on trusted networks or SSH tunnels.");
            Ok(())
        }
    }
}

fn host(port: u16) -> Result<()> {
    let certs = load_certs("certs/server.crt")?;
    let key = load_private_key("certs/server.key")?;
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| anyhow::anyhow!("server config: {e}"))?;
    let config = Arc::new(config);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr)?;
    println!("DDA Remote host listening (TLS) on {addr} (loopback only)");

    for stream in listener.incoming() {
        match stream {
            Ok(tcp) => {
                tcp.set_nodelay(true).ok();
                let conn = match ServerConnection::new(config.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("TLS setup failed: {e}");
                        continue;
                    }
                };
                let mut tls_stream = StreamOwned::new(conn, tcp);
                println!("Incoming TLS connection established.");
                if let Err(e) = serve(&mut tls_stream) {
                    eprintln!("Session ended: {e:#}");
                }
            }
            Err(e) => eprintln!("Error accepting connection: {e}"),
        }
    }
    Ok(())
}

fn serve<S: Stream>(stream: &mut S) -> Result<()> {
    match receive(stream)? {
        Message::Hello { version: PROTOCOL_VERSION, device_id, .. } => {
            println!("Client device_id: {}", device_id);
            if let Ok(trusted) = std::fs::read_to_string("trusted_devices.txt") {
                let allowed: Vec<&str> = trusted.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
                if !allowed.contains(&device_id.as_str()) {
                    send(stream, &Message::AccessDenied)?;
                    bail!("device_id not trusted");
                }
            }
        }
        _ => bail!("invalid protocol handshake"),
    }
    // Optional token auth
    if let Ok(expected) = std::env::var("DDA_TOKEN") {
        match receive(stream)? {
            Message::AuthToken { token } if token == expected => {}
            _ => {
                send(stream, &Message::AccessDenied)?;
                bail!("auth token rejected");
            }
        }
    }
    if !matches!(receive(stream)?, Message::RequestAccess) {
        bail!("access request expected");
    }
    print!("Allow remote screen and control? Type YES: ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    if answer.trim() != "YES" {
        send(stream, &Message::AccessDenied)?;
        bail!("access denied");
    }
    send(stream, &Message::AccessGranted)?;

    let screens = Screen::all()?;
    let screen = screens.first().context("no display detected")?;
    let mut enigo = Enigo::new(&Settings::default())?;
    let mut last_frame = Instant::now() - Duration::from_secs(1);

    loop {
        if last_frame.elapsed() >= Duration::from_millis(200) {
            let shot = screen.capture()?;
            let (w, h) = (shot.width(), shot.height());
            let mut jpeg = Vec::new();
            let rgba = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(w, h, shot.into_raw())
                .context("invalid captured image")?;
            JpegEncoder::new_with_quality(&mut jpeg, 55)
                .encode_image(&DynamicImage::ImageRgba8(rgba))?;
            send(stream, &Message::Screen { width: w, height: h, jpeg })?;
            last_frame = Instant::now();
        }
        match receive(stream) {
            Ok(Message::MouseMove { x, y }) => {
                let _ = enigo.move_mouse(x as i32, y as i32, Coordinate::Abs);
            }
            Ok(Message::MouseButton { button, down }) => {
                let b = match button.as_str() {
                    "left" => Button::Left,
                    "right" => Button::Right,
                    _ => continue,
                };
                let _ = enigo.button(b, if down { Direction::Press } else { Direction::Release });
            }
            Ok(Message::Key { key, down }) => {
                if let Some(c) = key.chars().next() {
                    let _ = enigo.key(enigo::Key::Unicode(c), if down { Direction::Press } else { Direction::Release });
                }
            }
            Ok(Message::Disconnect) => break,
            Err(_) => break,
            _ => {}
        }
    }
    Ok(())
}

fn connect(target: &str) -> Result<()> {
    let addr: SocketAddr = target.parse().context("target must be an IP:port")?;
    if !addr.ip().is_loopback() {
        bail!("prototype allows only loopback; use SSH port forwarding for remote hosts");
    }

    let mut root_store = rustls::RootCertStore::empty();
    // For prototype: accept self-signed cert by skipping verification
    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(NoCertificateVerification))
        .with_no_client_auth();
    let config = Arc::new(config);
    let server_name = rustls::ServerName::try_from("localhost").unwrap();

    let tcp = TcpStream::connect(addr)?;
    tcp.set_nodelay(true).ok();
    let conn = ClientConnection::new(config, server_name)?;
    let mut tls_stream = StreamOwned::new(conn, tcp);

    let device_id = std::fs::read_to_string("device_id.txt").unwrap_or_else(|_| "client-unknown".to_string());
    send(
        &mut tls_stream,
        &Message::Hello {
            version: PROTOCOL_VERSION,
            client: "dda-remote".into(),
            public_key: "".into(),
            device_id,
        },
    )?;
    if let Ok(token) = std::env::var("DDA_TOKEN") {
        send(&mut tls_stream, &Message::AuthToken { token })?;
    }
    send(&mut tls_stream, &Message::RequestAccess)?;

    match receive(&mut tls_stream)? {
        Message::AccessGranted => println!("Host granted access (TLS)"),
        Message::AccessDenied => bail!("Host denied access"),
        other => bail!("unexpected response: {other:?}"),
    }

    println!("Connected (TCP+TLS prototype). Viewer loop not fully implemented in this CLI build.");
    Ok(())
}
