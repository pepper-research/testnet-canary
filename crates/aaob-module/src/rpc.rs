use crate::{get_market_id, MarketId, OrderbookId, Slab, AAOB};
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::error::ErrorCode;
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::rest::utils::{ApiResult, Path, Query};
use sov_modules_api::rest::{ApiState, HasCustomRestApi};
use sov_modules_api::{ApiStateAccessor, Spec, StateReader, StateReaderAndWriter};
use sov_state::User;
use spicenet_shared::Side;
use axum::routing::get;
use sov_modules_api::prelude::axum;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OrderbookDetails {
    pub id: OrderbookId,
    pub market_id: MarketId,
    pub bids: Slab,
    pub asks: Slab,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct BestBidOffer {
    pub best_bid: Option<u32>,
    pub best_ask: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct MarketBBO {
    pub market_name: String,
    pub bbo: BestBidOffer,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct MarketDetails {
    pub name: String,
    pub account_tag: u64,
    pub orderbook_id: OrderbookId,
    pub callback_id_len: u64,
    pub callback_info_len: u64,
    pub fee_budget: u64,
    pub min_base_size: u64,
    pub tick_size: u64,
    pub cranker_reward: u64,
}

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub struct getMarketBBOQuery {
    pub markets: Vec<String>,
}

impl<S: Spec> AAOB<S> {
    pub fn best_bid_offer<Reader: StateReader<User>>(
        &self,
        orderbook_id: OrderbookId,
        accessor: &mut Reader,
    ) -> Result<Option<BestBidOffer>, Reader::Error> {
        let orderbook = self.orderbooks.get(&orderbook_id, accessor)?.unwrap();

        let best_bid = orderbook.find_bbo(Side::Bid);
        let best_ask = orderbook.find_bbo(Side::Ask);
        Ok(Option::from(BestBidOffer { best_bid, best_ask }))
    }

    pub fn orderbook<Reader: StateReader<User>>(
        &self,
        orderbook_id: OrderbookId,
        accessor: &mut Reader,
    ) -> Result<Option<OrderbookDetails>, Reader::Error> {
        let ob = self.orderbooks.get(&orderbook_id, accessor)?;

        Ok(ob.map(|ob| OrderbookDetails {
            market_id: ob.market_id,
            id: ob.market_id,
            bids: ob.bids,
            asks: ob.asks,
        }))
    }

    pub fn get_markets<ReaderAndWriter: StateReaderAndWriter<User>>(
        &self,
        accessor: &mut ReaderAndWriter,
    ) -> Result<Vec<MarketDetails>, <ReaderAndWriter as StateReader<User>>::Error> {
        self.market_ids
            .iter(accessor)?
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|market_id| {
                market_id
                    .and_then(|market_id| self.markets.get(&market_id, accessor))
                    .transpose()
            })
            .map(|market| {
                market.map(|market| MarketDetails {
                    orderbook_id: get_market_id::<S>(&market.name),
                    name: market.name,
                    fee_budget: market.fee_budget,
                    min_base_size: market.min_base_size,
                    tick_size: market.tick_size,
                    // Remove fields that don't exist in Market struct
                    account_tag: 0,       // Placeholder, remove if not needed
                    callback_id_len: 0,   // Placeholder, remove if not needed
                    callback_info_len: 0, // Placeholder, remove if not needed
                    cranker_reward: 0,    // Placeholder, remove if not needed
                })
            })
            .collect()
    }

    pub fn markets_bbo<Reader: StateReader<User>>(
        &self,
        market_names: Vec<String>,
        accessor: &mut Reader,
    ) -> Result<Vec<MarketBBO>, Reader::Error> {
        let mut results = Vec::new();

        for market_name in market_names.iter() {
            let market_id = get_market_id::<S>(market_name);
            if self.markets.get(&market_id, accessor)?.is_some() {
                if let Some(bbo) = self.best_bid_offer(market_id, accessor)? {
                    results.push(MarketBBO {
                        market_name: market_name.clone(),
                        bbo,
                    });
                }
            }
        }

        Ok(results)
    }
}

#[rpc_gen(client, server, namespace = "orderbook")]
impl<S: Spec> AAOB<S> {
    #[rpc_method(name = "getBestBidOffer")]
    /// Get the best bid and offer
    pub fn get_best_bid_offer(
        &self,
        orderbook_id: OrderbookId,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<BestBidOffer> {
        self.best_bid_offer(orderbook_id, state)
            .map_err(|_| ErrorCode::InternalError.into()) // Adjusted error handling
            .and_then(|result| result.ok_or(ErrorCode::InvalidParams.into())) // Ensure proper handling of Option
    }

    #[rpc_method(name = "getMarkets")]
    pub fn get_markets_rpc(
        &self,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<Vec<MarketDetails>> {
        self.get_markets(state)
            .map_err(|_| ErrorCode::InternalError.into())
    }

    #[rpc_method(name = "getMarketsBBO")]
    pub fn get_markets_bbo(
        &self,
        market_names: Vec<String>,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<Vec<MarketBBO>> {
        self.markets_bbo(market_names, state)
            .map_err(|_| ErrorCode::InternalError.into())
    }

    #[rpc_method(name = "getOrderbook")]
    pub fn get_orderbook(
        &self,
        orderbook_id: OrderbookId,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<OrderbookDetails> {
        match self.orderbook(orderbook_id, state) {
            Ok(Some(orderbook)) => Ok(orderbook),
            _ => Err(ErrorCode::InternalError.into()),
        }
    }
}


impl<S: Spec> HasCustomRestApi for AAOB<S> {
    type Spec = S;

    fn custom_rest_api(&self, state: ApiState<S>) -> axum::Router<()> {
        axum::Router::new()
            .route("/orderbook/best-bid-offer/:orderbookId", get(Self::get_best_bid_offer_rest))
            .route("/orderbook/markets", get(Self::get_markets_rest))
            .route("/orderbook/markets-bbo/:markets", get(Self::get_markets_bbo_rest))
            .route("/orderbook/orderbook/:orderbookId", get(Self::get_orderbook_rest))
            .with_state(state.with(self.clone()))
    }
}

impl<S: Spec> AAOB<S> {
    async fn get_best_bid_offer_rest(
        state: ApiState<S, Self>,
        mut accessor: ApiStateAccessor<S>,
        Path(orderbook_id): Path<OrderbookId>,
    ) -> ApiResult<BestBidOffer> {
        let bbo = state.get_best_bid_offer(orderbook_id).unwrap();

        Ok(BestBidOffer {
            best_bid: bbo.best_bid,
            best_ask: bbo.best_ask,
        }
        .into())
    }

    async fn get_markets_rest(
        state: ApiState<S, Self>,
        mut accessor: ApiStateAccessor<S>,
        Path(()): Path<()>,
    ) -> ApiResult<Vec<MarketDetails>> {
        let markets = state.get_markets_rpc().unwrap();

        Ok(markets.into())
    }

    async fn get_markets_bbo_rest(
        state: ApiState<S, Self>,
        mut accessor: ApiStateAccessor<S>,
        params : Query<getMarketBBOQuery>,
    ) -> ApiResult<Vec<MarketBBO>> {
        let markets_bbo = state.get_markets_bbo(
            params.markets.clone(),
        ).unwrap();

        Ok(markets_bbo.into())
    }

    async fn get_orderbook_rest(
        state: ApiState<S, Self>,
        mut accessor: ApiStateAccessor<S>,
        Path(orderbook_id): Path<OrderbookId>,
    ) -> ApiResult<OrderbookDetails> {
        let orderbook = state.get_orderbook(orderbook_id).unwrap();

        Ok(orderbook.into())
    }
}