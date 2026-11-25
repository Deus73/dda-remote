use anyhow::Result;
use dda_protocol::{decode, encode, Message, MAX_FRAME_BYTES};
use std::io::{Read, Write};
use std::net::TcpStream;
use rustls::{ServerConnection, ClientConnection, StreamOwned};

pub trait Stream: Read + Write + Send {}

impl Stream for TcpStream {}
impl Stream for StreamOwned<ServerConnection, TcpStream> {}
impl Stream for StreamOwned<ClientConnection, TcpStream> {}

pub fn send<S: Stream>(stream: &mut S, msg: &Message) -> Result<()> {
    let data = encode(msg)?;
    let len = (data.len() as u32).to_be_bytes();
    stream.write_all(&len)?;
    stream.write_all(&data)?;
    stream.flush()?;
    Ok(())
}

pub fn receive<S: Stream>(stream: &mut S) -> Result<Message> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_FRAME_BYTES {
        anyhow::bail!("frame too large: {} bytes", len);
    }
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    let msg = decode(&buf)?;
    Ok(msg)
}

