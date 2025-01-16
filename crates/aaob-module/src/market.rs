use std::marker::PhantomData;

use anyhow::anyhow;
use sov_modules_api::{Context, Spec, StateAccessor, StateMap};

use crate::get_market_id;
use crate::MarketId;
use crate::OrderbookId;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
pub struct Market<S: Spec> {
    pub name: String, // Added this as seed for id derivation, can be changed

    // pub account_tag: u64,
    // pub orderbook_id: OrderbookId,

    // pub caller_authority: MarketCallerAuthority<S>,
    /// * the length of an order actor's callback identifier
    // pub callback_id_len: u64,

    /// * the length of an order actor's callback metadata
    // pub callback_info_len: u64,
    pub fee_budget: u64,

    /// * the minimum order size that is left after matching, that can be inserted into the book
    pub min_base_size: u64,

    pub tick_size: u64,

    // pub cranker_reward: u64,
    #[cfg_attr(
        feature = "native",
        serde(skip_serializing, default)
    )]
    _phantom: PhantomData<S>,
}

impl<S: Spec> Market<S> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        market_name: &str,
        // account_tag: u64,
        // orderbook_id: OrderbookId,
        // caller_authority: MarketCallerAuthority<S>,
        // callback_id_len: u64,
        // callback_info_len: u64,
        fee_budget: u64,
        min_base_size: u64,
        tick_size: u64,
        // cranker_reward: u64,
        markets: &StateMap<MarketId, Market<S>>,
        _context: &Context<S>,
        state: &mut impl StateAccessor,
    ) -> anyhow::Result<(MarketId, Market<S>)> {
        let market_id = get_market_id::<S>(market_name);
        let market = markets.get(&market_id, state)?;
        if market.is_some() {
            Err(anyhow!("Market with name: {} already exists.", market_name,))
        } else {
            Ok((
                market_id,
                Market::<S> {
                    name: market_name.to_owned(),
                    // account_tag,
                    // orderbook_id,
                    // caller_authority,
                    // callback_id_len,
                    // callback_info_len,
                    fee_budget,
                    min_base_size,
                    tick_size,
                    // cranker_reward,
                    _phantom: Default::default(),
                },
            ))
        }
    }
}
