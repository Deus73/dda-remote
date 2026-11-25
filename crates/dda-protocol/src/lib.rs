use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u16 = 2; // Upgraded for security handshake
pub const MAX_FRAME_BYTES: usize = 12 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    Hello { version: u16, client: String, public_key: String }, // Added public_key
    RequestAccess,
    AccessGranted,
    AccessDenied,
    Screen { width: u32, height: u32, jpeg: Vec<u8> },
    MouseMove { x: u32, y: u32 },
    MouseButton { button: String, down: bool },
    Key { key: String, down: bool },
    // Handshake messages for E2E and Auth (Fase C)
    KeyExchange { ephemeral_key: String },
    AuthChallenge { challenge: String },
    AuthResponse { signature: String },
    AuthToken { token: String },
    // Existing messages
    Ping,
    Pong,
    Disconnect,
    Error { message: String },
}

pub fn encode(message: &Message) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(message)
}

pub fn decode(bytes: &[u8]) -> Result<Message, serde_json::Error> {
    serde_json::from_slice(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let data = encode(&Message::Hello { version: 2, client: "test".into(), public_key: "pk_test".into() }).unwrap();
        assert!(matches!(decode(&data).unwrap(), Message::Hello { version: 2, .. }));
    }
}
