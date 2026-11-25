//! DDA Remote v0.1.0 — TCP prototype (reverted from QUIC for buildability).
//! NO encryption/authentication yet: NEVER expose the host to the internet.
use anyhow::{bail, Context, Result};
use dda_core::{receive, send, Stream};
use dda_protocol::{Message, PROTOCOL_VERSION};
use enigo::{Button, Coordinate, Direction, Enigo, Keyboard, Mouse, Settings};
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageBuffer, Rgba};
use screenshots::Screen;
use std::io::{self, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::{Duration, Instant};

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
            println!("DDA Remote 0.1.0 | TCP PROTOTYPE");
            println!("  dda-remote host [port]");
            println!("  dda-remote connect <ip:port>");
            println!("WARNING: Unencrypted; use ONLY on loopback or trusted network.");
            Ok(())
        }
    }
}

fn host(port: u16) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr)?;
    println!("DDA Remote host listening on {addr} (loopback only)");
    println!("For two computers, use a separately authenticated SSH tunnel.");

    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                println!("Incoming connection established.");
                if let Err(e) = serve(&mut s) {
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
        Message::Hello { version: PROTOCOL_VERSION, .. } => {}
        _ => bail!("invalid protocol handshake"),
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
        // non-blocking-ish attempt: try to read with short timeout not easy on std TcpStream without set_read_timeout
        // For simplicity, set a read timeout
        // But Stream is generic; easier to handle in TcpStream case in practice. For this prototype, do blocking reads carefully.
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
            Err(_) => {
                // connection closed or error
                break;
            }
            _ => {}
        }
    }
    Ok(())
}

fn connect(target: &str) -> Result<()> {
    let addr: SocketAddr = target.parse().context("target must be an IP:port")?;
    if !addr.ip().is_loopback() {
        bail!("prototype allows only loopback; use SSH port forwarding");
    }

    let mut stream = TcpStream::connect(addr)?;
    stream.set_nodelay(true).ok();

    send(
        &mut stream,
        &Message::Hello {
            version: PROTOCOL_VERSION,
            client: "dda-remote".into(),
            public_key: "".into(),
        },
    )?;
    send(&mut stream, &Message::RequestAccess)?;

    match receive(&mut stream)? {
        Message::AccessGranted => println!("Host granted access"),
        Message::AccessDenied => bail!("Host denied access"),
        other => bail!("unexpected response: {other:?}"),
    }

    println!("Connected (TCP prototype). UI loop not fully implemented in this build.");
    Ok(())
}
