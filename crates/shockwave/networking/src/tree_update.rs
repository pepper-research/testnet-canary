use crate::NetworkError;
use crate::udp::UdpNetwork;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use ripple::epidemic_tree::TreeUpdate;

#[derive(Debug, Serialize, Deserialize)]
pub struct TreeUpdatePacket {
    pub update: TreeUpdate, // This is the TreeUpdate struct from the ripple crate
}

impl TreeUpdatePacket {
    pub fn new(update: TreeUpdate) -> Self {
        Self { update }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, NetworkError> {
        Ok(bincode::serialize(self)?)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, NetworkError> {
        Ok(bincode::deserialize(bytes)?)
    }
}

pub struct TreeUpdateHandler {
    network: UdpNetwork,
}

impl TreeUpdateHandler {
    pub fn new(network: UdpNetwork) -> Self {
        Self { network }
    }

    pub async fn listen_for_updates(&self, callback: impl Fn(TreeUpdate) -> Result<(), NetworkError>) -> Result<(), NetworkError> {
        let mut buf = vec![0u8; 65507]; // Max UDP packet size
        loop {
            let (size, _) = self.network.receive(&mut buf).await?;
            let packet = TreeUpdatePacket::deserialize(&buf[..size])?;
            callback(packet.update)?;
        }
    }
}
