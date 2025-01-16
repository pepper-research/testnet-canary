use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(
    Copy, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, Eq, PartialEq, Debug,
)]
pub struct OracleId(u64);