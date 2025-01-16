use serde::{Serialize, Deserialize};
use super::NetworkError;

#[derive(Debug, Serialize, Deserialize)]
pub enum PacketType {
    Data,
    Coding,
    // Add more packet types as needed
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Packet {
    pub packet_type: PacketType,
    pub data: Vec<u8>,
}

impl Packet {
    pub fn new(packet_type: PacketType, data: Vec<u8>) -> Self {
        Self { packet_type, data }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, NetworkError> {
        Ok(bincode::serialize(self)?)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, NetworkError> {
        Ok(bincode::deserialize(bytes)?)
    }
}