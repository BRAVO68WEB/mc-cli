use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// Protocol constants from mcrcon reference
const RCON_EXEC_COMMAND: i32 = 2;
const RCON_AUTHENTICATE: i32 = 3;
const RCON_PID: i32 = 0x0badc0de; // arbitrary client id

const MIN_PACKET_SIZE: i32 = 10; // size(id + type + empty) + payload

pub struct RconClient {
    stream: TcpStream,
}

impl RconClient {
    pub async fn connect(
        host: &str,
        port: u16,
        password: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", host, port);
        let mut stream = TcpStream::connect(addr).await?;

        // authenticate
        let auth_packet = build_packet(RCON_PID, RCON_AUTHENTICATE, password);
        send_packet(&mut stream, &auth_packet).await?;
        let resp = recv_packet(&mut stream).await?;
        if resp.id == -1 {
            return Err("Authentication failed".into());
        }

        Ok(Self { stream })
    }

    pub async fn cmd(&mut self, command: &str) -> Result<String, Box<dyn std::error::Error>> {
        let packet = build_packet(RCON_PID, RCON_EXEC_COMMAND, command);
        send_packet(&mut self.stream, &packet).await?;
        let resp = recv_packet(&mut self.stream).await?;
        if resp.id != RCON_PID {
            return Err("Invalid response id".into());
        }
        Ok(resp.payload)
    }
}

struct Packet {
    size: i32,
    id: i32,
    kind: i32,
    payload: String,
}

fn build_packet(id: i32, kind: i32, payload: &str) -> Packet {
    // size = id(4) + kind(4) + payload bytes + 2 null bytes
    let payload_len = payload.len() as i32;
    let size = 4 + 4 + payload_len + 2;
    Packet {
        size,
        id,
        kind,
        payload: payload.to_string(),
    }
}

async fn send_packet(
    stream: &mut TcpStream,
    packet: &Packet,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = Vec::with_capacity((packet.size + 4) as usize);
    buf.extend_from_slice(&packet.size.to_le_bytes());
    buf.extend_from_slice(&packet.id.to_le_bytes());
    buf.extend_from_slice(&packet.kind.to_le_bytes());
    buf.extend_from_slice(packet.payload.as_bytes());
    buf.push(0); // string null terminator
    buf.push(0); // second empty string null terminator
    stream.write_all(&buf).await?;
    Ok(())
}

async fn recv_packet(stream: &mut TcpStream) -> Result<Packet, Box<dyn std::error::Error>> {
    let mut size_le = [0u8; 4];
    stream.read_exact(&mut size_le).await?;
    let size = i32::from_le_bytes(size_le);
    if !(MIN_PACKET_SIZE..=4096).contains(&size) {
        return Err("Invalid packet size".into());
    }

    let mut rest = vec![0u8; size as usize];
    stream.read_exact(&mut rest).await?;

    if rest.len() < 8 {
        return Err("Short packet".into());
    }
    let id = i32::from_le_bytes(rest[0..4].try_into().unwrap());
    let kind = i32::from_le_bytes(rest[4..8].try_into().unwrap());
    // payload is until last two null bytes
    if rest.len() < 10 {
        return Err("Short payload".into());
    }
    // strip last two nulls
    let payload_bytes = &rest[8..rest.len() - 2];
    let payload = String::from_utf8_lossy(payload_bytes).to_string();

    Ok(Packet {
        size,
        id,
        kind,
        payload,
    })
}
