use sov_modules_api::{Address, Spec};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
/// Buffer attached to aaob events to tie owner to events
pub struct CallBackInfo<S: Spec> {
    pub user_account: Address<S>, // TODO: Dummy for now, need to confirm
    pub open_orders_idx: u64,
    pub order_nonce: u128, // global nonce per order. 128 bits because we're designing smart contracts for the future :)
    pub client_order_id: u64,
}

impl<S: Spec> CallBackInfo<S> {
    pub fn to_vec(&self) -> Vec<u8> {
        [
            self.user_account.as_bytes().to_vec(),
            self.open_orders_idx.to_le_bytes().to_vec(),
            self.order_nonce.to_le_bytes().to_vec(),
            self.client_order_id.to_le_bytes().to_vec(),
        ]
        .concat()
    }
}
