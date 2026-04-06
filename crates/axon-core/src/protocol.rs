use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::io::{Error, ErrorKind};

/// AXP Protocol Magic Number: 'AXON'
pub const AXP_MAGIC: u32 = 0x41584F4E;
pub const AXP_VERSION: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum PacketType {
    Status = 0,
    Hold = 1,
    Resume = 2,
    Control = 3,
    Data = 4,
    Heartbeat = 5,
    Nogari = 6,
}

impl From<u8> for PacketType {
    fn from(v: u8) -> Self {
        match v {
            0 => PacketType::Status,
            1 => PacketType::Hold,
            2 => PacketType::Resume,
            3 => PacketType::Control,
            4 => PacketType::Data,
            5 => PacketType::Heartbeat,
            6 => PacketType::Nogari,
            _ => PacketType::Status, // Default to status for safe fallback
        }
    }
}

/// AXP Packet Structure
/// [Magic(4)] [Version(1)] [Type(1)] [Reserved(2)] [PayloadLen(4)] [Payload(N)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AxonPacket {
    pub packet_type: PacketType,
    pub payload: Vec<u8>,
}

impl AxonPacket {
    pub fn new(packet_type: PacketType, payload: Vec<u8>) -> Self {
        Self {
            packet_type,
            payload,
        }
    }

    /// Encode packet to an asynchronous writer
    pub async fn write_to<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<(), Error> {
        // Write Magic
        writer.write_u32(AXP_MAGIC).await?;
        // Write Version
        writer.write_u8(AXP_VERSION).await?;
        // Write Type
        writer.write_u8(self.packet_type as u8).await?;
        // Reserved bytes
        writer.write_u16(0).await?;
        // Write Payload Length
        writer.write_u32(self.payload.len() as u32).await?;
        // Write Payload
        writer.write_all(&self.payload).await?;
        
        writer.flush().await?;
        Ok(())
    }

    /// Decode packet from an asynchronous reader
    pub async fn read_from<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self, Error> {
        // Read Magic
        let magic = reader.read_u32().await?;
        if magic != AXP_MAGIC {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid AXP Magic: 0x{:X}", magic),
            ));
        }

        // Read Version
        let version = reader.read_u8().await?;
        if version != AXP_VERSION {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unsupported AXP Version: {}", version),
            ));
        }

        // Read Type
        let packet_type = PacketType::from(reader.read_u8().await?);

        // Skip Reserved (2 bytes)
        let _ = reader.read_u16().await?;

        // Read Payload Length
        let len = reader.read_u32().await? as usize;
        
        // Safety check for payload length (e.g., 10MB limit)
        if len > 10 * 1024 * 1024 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Payload size exceeds limit (10MB)",
            ));
        }

        // Read Payload
        let mut payload = vec![0u8; len];
        reader.read_exact(&mut payload).await?;

        Ok(Self {
            packet_type,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_packet_codec() {
        let payload = b"Hello, AXON!".to_vec();
        let original = AxonPacket::new(PacketType::Data, payload.clone());
        
        let mut buffer = Vec::new();
        original.write_to(&mut buffer).await.unwrap();
        
        let mut reader = Cursor::new(buffer);
        let decoded = AxonPacket::read_from(&mut reader).await.unwrap();
        
        assert_eq!(decoded.packet_type, PacketType::Data);
        assert_eq!(decoded.payload, payload);
    }

    #[tokio::test]
    async fn test_invalid_magic() {
        let mut invalid_buffer = vec![0u8; 12];
        let mut reader = Cursor::new(invalid_buffer);
        let result = AxonPacket::read_from(&mut reader).await;
        assert!(result.is_err());
    }
}
