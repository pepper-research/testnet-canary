/// Time module events
#[derive(
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
    Debug,
    PartialEq,
    Clone,
)]
pub enum Event {
    /// Update time state event
    UpdateSlot {
        slot: u64,
    },
    UpdateTime {
        unix_timestamp: u64,
    },
}
