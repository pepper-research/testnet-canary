use sov_modules_api::{
    Context, Error, GenesisState, Module, ModuleId, ModuleInfo, Spec, StateMap,
    StateVec, TxState, DaSpec
};
use std::marker::PhantomData;

pub use address::*;
pub use call::*;
pub use error::*;
pub use event::*;
pub use genesis::*;
pub use market::*;
pub use orderbook::*;
#[cfg(feature = "native")]
pub use rpc::*;
pub use state::*;
pub use tree::*;
pub use utils::*;

// use crate::event::Event;
// use crate::market::Market;
// use crate::orderbook::OrderbookState;

pub mod address;
pub mod call;
pub mod error;
pub mod event;
pub mod genesis;
pub mod market;
pub mod orderbook;
#[cfg(feature = "native")]
pub mod rpc;
pub mod state;
pub mod tree;
pub mod utils;

#[derive(Clone, ModuleInfo, sov_modules_api::ModuleRestApi)]
pub struct AAOB<S: Spec> {
    #[id]
    id: ModuleId,

    #[state]
    markets: StateMap<MarketId, Market<S>>,

    #[state]
    market_ids: StateVec<MarketId>,

    #[state]
    orderbooks: StateMap<OrderbookId, OrderbookState>,

    #[phantom]
    _marker: PhantomData<S>,
}

impl<S: Spec> Module for AAOB<S> {
    type Spec = S;
    type Config = AAOBConfig<S>;
    type CallMessage = CallMessage;
    type Event = Event;

    fn genesis(
        &self,
        _genesis_rollup_header: &<<S as Spec>::Da as DaSpec>::BlockHeader,
        _validity_condition: &<<S as Spec>::Da as DaSpec>::ValidityCondition,
        config: &Self::Config,
        state: &mut impl GenesisState<S>,
    ) -> Result<(), Error> {
        Ok(self.init_module(config, state)?)
    }

    fn call(
        &self,
        msg: Self::CallMessage,
        context: &Context<Self::Spec>,
        state: &mut impl TxState<S>,
    ) -> Result<(), Error> {
        match msg {
            CallMessage::CreateOrder {
                market_id,
                side,
                post_only,
                post_allowed,
                limit_price,
                max_quote_qty,
                max_base_qty,
                self_trade_behavior,
                match_limit,
                trg_id,
            } => self.create_order(
                market_id,
                side,
                max_base_qty,
                max_quote_qty,
                limit_price,
                post_only,
                post_allowed,
                self_trade_behavior,
                trg_id,
                context,
                state,
                match_limit,
            ),

            CallMessage::CancelOrder {
                market_id,
                order_id,
                side,
            } => self.cancel_order(market_id, order_id, side, context, state),

            CallMessage::CreateMarket {
                market_name,
                fee_budget,
                min_base_size,
                tick_size,
            } => self.create_market(
                &(String::from(market_name)),
                fee_budget,
                min_base_size,
                tick_size,
                context,
                state,
            ),

            CallMessage::CloseMarket { market_name } => self.close_market(&(String::from(market_name)), state),
        };

        Ok(())
    }
}
