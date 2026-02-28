#[allow(dead_code)]

/// AXP (AXON Protocol) Header
/// 4-byte Magic Number: [0x41, 0x58, 0x4f, 0x4e] (ASCII "AXON")
/// 1-byte Packet Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxpHeader {
    pub magic: [u8; 4],
    pub packet_type: PacketType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketType {
    /// Agent to Daemon: Code Submission
    Submit = 0x01,
    /// Daemon to Agent: Hold execution (Pause)
    Hold = 0x02,
    /// Daemon to Agent: Resume execution
    Resume = 0x03,
    /// Agent to Daemon: Lounge/Nogari message
    Lounge = 0x04,
    /// System: Heartbeat/Status update
    Status = 0x05,
    /// Unknown packet type
    Unknown = 0xFF,
}

impl From<u8> for PacketType {
    fn from(byte: u8) -> Self {
        match byte {
            0x01 => PacketType::Submit,
            0x02 => PacketType::Hold,
            0x03 => PacketType::Resume,
            0x04 => PacketType::Lounge,
            0x05 => PacketType::Status,
            _ => PacketType::Unknown,
        }
    }
}

pub const AXON_MAGIC: [u8; 4] = [0x41, 0x58, 0x4f, 0x4e];

impl AxpHeader {
    pub fn new(packet_type: PacketType) -> Self {
        Self {
            magic: AXON_MAGIC,
            packet_type,
        }
    }

    pub fn to_bytes(&self) -> [u8; 5] {
        let mut bytes = [0u8; 5];
        bytes[0..4].copy_from_slice(&self.magic);
        bytes[4] = self.packet_type as u8;
        bytes
    }

    pub fn from_bytes(bytes: &[u8; 5]) -> Option<Self> {
        if bytes[0..4] != AXON_MAGIC {
            return None;
        }
        Some(Self {
            magic: AXON_MAGIC,
            packet_type: PacketType::from(bytes[4]),
        })
    }
}

#[derive(Debug, Clone)]
pub struct AxpPacket {
    pub header: AxpHeader,
    pub payload: Vec<u8>,
}

impl AxpPacket {
    pub fn new(packet_type: PacketType, payload: Vec<u8>) -> Self {
        Self {
            header: AxpHeader::new(packet_type),
            payload,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(5 + self.payload.len());
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 5 {
            return None;
        }
        let header_bytes: [u8; 5] = bytes[0..5].try_into().ok()?;
        let header = AxpHeader::from_bytes(&header_bytes)?;
        let payload = bytes[5..].to_vec();
        Some(Self { header, payload })
    }
}

#[allow(dead_code)]
pub mod types;
