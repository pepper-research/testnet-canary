
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    pub id: u64,
    pub transactions: Vec<Transaction>,
    pub sequencer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    // Define transaction structure
}