use thiserror::Error;

pub mod udp;
pub mod packet;
pub mod tree_update;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Handshake error: {0}")]
    HandshakeError(String),
    // Add more error types as needed

}