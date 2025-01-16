use spicenet_shared::derivative::DerivativeID;

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
    DerivativeInitalized { id: DerivativeID },
    DerivativeSettled { id: DerivativeID },
}
