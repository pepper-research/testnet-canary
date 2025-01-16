use jsonrpsee::core::RpcResult;
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::rest::utils::{ApiResult, Path, Query};
use sov_modules_api::rest::{ApiState, HasCustomRestApi};
use sov_modules_api::{ApiStateAccessor, Spec, StateReader};
use sov_state::User;
use spicenet_shared::Fractional;
use std::fmt::{self, Display};
use axum::routing::get;
use sov_modules_api::prelude::axum;

use crate::LookupTable;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AggrDataResponse {
    pub ok: bool,
}

impl Display for AggrDataResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ok)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PricesDataResponse {
    pub prices: Vec<Fractional>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OnePriceDataResponse {
    pub price: Fractional,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AggrConfIntervalsResponse {
    pub aggregate_conf_intervals: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OneAggrConfIntervalResponse {
    pub aggregate_conf_interval: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct EmaDataResponse {
    pub ema: Vec<Fractional>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OneEmaResponse {
    pub ema: Fractional,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TickDataResponse {
    pub price_ticks: Vec<Vec<Fractional>>,
    pub aggregate_conf_interval_ticks: Vec<Vec<u32>>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OneTickResponse {
    pub price_ticks: Vec<Fractional>,
    pub aggregate_conf_interval_ticks: Vec<u32>,
}

impl<S: Spec> LookupTable<S> {
    pub fn get_all_prices<Reader: StateReader<User>>(
        &self,
        state: &mut Reader,
    ) -> RpcResult<PricesDataResponse> {
        Ok(PricesDataResponse {
            prices: self.prices.get(state).unwrap().unwrap().to_vec(),
        })
    }

    pub fn get_all_aggr_conf_intervals<Reader: StateReader<User>>(
        &self,
        state: &mut Reader,
    ) -> RpcResult<AggrConfIntervalsResponse> {
        Ok(AggrConfIntervalsResponse {
            aggregate_conf_intervals: self
                .aggregate_conf_intervals
                .get(state)
                .unwrap()
                .unwrap()
                .to_vec(),
        })
    }

    pub fn get_price<Reader: StateReader<User>>(
        &self,
        index: usize,
        state: &mut Reader,
    ) -> RpcResult<OnePriceDataResponse> {
        Ok(OnePriceDataResponse {
            price: self.prices.get(state).unwrap().unwrap()[index],
        })
    }

    pub fn get_aggregate_conf_interval<Reader: StateReader<User>>(
        &self,
        index: usize,
        state: &mut Reader,
    ) -> RpcResult<OneAggrConfIntervalResponse> {
        Ok(OneAggrConfIntervalResponse {
            aggregate_conf_interval: self.aggregate_conf_intervals.get(state).unwrap().unwrap()
                [index],
        })
    }

    pub fn get_many_prices<Reader: StateReader<User>>(
        &self,
        indices: Vec<usize>,
        state: &mut Reader,
    ) -> RpcResult<PricesDataResponse> {
        let all_prices = self.prices.get(state).unwrap_or_default();
        Ok(PricesDataResponse {
            prices: indices.iter().map(|i| all_prices.unwrap()[*i]).collect(),
        })
    }

    pub fn get_many_aggregate_conf_intervals<Reader: StateReader<User>>(
        &self,
        indices: Vec<usize>,
        state: &mut Reader,
    ) -> RpcResult<AggrConfIntervalsResponse> {
        let all_aggregate_conf_intervals =
            self.aggregate_conf_intervals.get(state).unwrap_or_default();
        Ok(AggrConfIntervalsResponse {
            aggregate_conf_intervals: indices
                .iter()
                .map(|i| all_aggregate_conf_intervals.unwrap()[*i])
                .collect(),
        })
    }

    pub fn get_ema<Reader: StateReader<User>>(
        &self,
        index: usize,
        state: &mut Reader,
    ) -> RpcResult<OneEmaResponse> {
        Ok(OneEmaResponse {
            ema: self.price_emas.get(state).unwrap().unwrap()[index],
        })
    }

    pub fn get_all_ema<Reader: StateReader<User>>(
        &self,
        state: &mut Reader,
    ) -> RpcResult<EmaDataResponse> {
        Ok(EmaDataResponse {
            ema: self.price_emas.get(state).unwrap().unwrap().to_vec(),
        })
    }

    // returns the ticks for the speicifc second for all markets, this is not market index
    pub fn get_tick<Reader: StateReader<User>>(
        &self,
        index: usize,
        state: &mut Reader,
    ) -> RpcResult<OneTickResponse> {
        Ok(OneTickResponse {
            price_ticks: self
                .price_ticks
                .get(state)
                .unwrap()
                .unwrap()
                .get(index)
                .unwrap()
                .to_vec(),
            aggregate_conf_interval_ticks: self
                .aggregate_conf_interval_ticks
                .get(state)
                .unwrap()
                .unwrap()
                .get(index)
                .unwrap()
                .to_vec(),
        })
    }

    pub fn get_all_ticks<Reader: StateReader<User>>(
        &self,
        state: &mut Reader,
    ) -> RpcResult<TickDataResponse> {
        Ok(TickDataResponse {
            price_ticks: self
                .price_ticks
                .get(state)
                .unwrap()
                .unwrap()
                .to_nested_vec(),
            aggregate_conf_interval_ticks: self
                .aggregate_conf_interval_ticks
                .get(state)
                .unwrap()
                .unwrap()
                .to_nested_vec(),
        })
    }
}

#[rpc_gen(client, server, namespace = "lut")]
impl<S: Spec> LookupTable<S> {
    // fetch prices
    #[rpc_method(name = "getAllPrices")]
    pub fn get_all_prices_rpc(
        &self,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<PricesDataResponse> {
        self.get_all_prices(state)
    }

    // fetch aggr conf
    #[rpc_method(name = "getAllAggrConfIntervals")]
    pub fn get_all_aggr_conf_intervals_rpc(
        &self,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<AggrConfIntervalsResponse> {
        self.get_all_aggr_conf_intervals(state)
    }

    // fetch one price by index
    #[rpc_method(name = "getPrice")]
    pub fn get_price_rpc(
        &self,
        index: usize,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<OnePriceDataResponse> {
        self.get_price(index, state)
    }

    // fetch one aggr conf by index
    #[rpc_method(name = "getAggrConfInterval")]
    pub fn get_aggregate_conf_interval_rpc(
        &self,
        index: usize,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<OneAggrConfIntervalResponse> {
        self.get_aggregate_conf_interval(index, state)
    }

    // fetch many prices by indices
    #[rpc_method(name = "getManyPrices")]
    pub fn get_many_prices_rpc(
        &self,
        indices: Vec<usize>,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<PricesDataResponse> {
        self.get_many_prices(indices, state)
    }

    // fetch many aggr conf by indices
    #[rpc_method(name = "getManyAggrConfIntervals")]
    pub fn get_many_aggregate_conf_intervals_rpc(
        &self,
        indices: Vec<usize>,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<AggrConfIntervalsResponse> {
        self.get_many_aggregate_conf_intervals(indices, state)
    }

    #[rpc_method(name = "getAllEma")]
    pub fn get_all_ema_rpc(&self, state: &mut ApiStateAccessor<S>) -> RpcResult<EmaDataResponse> {
        self.get_all_ema(state)
    }

    #[rpc_method(name = "getEma")]
    pub fn get_ema_rpc(
        &self,
        index: usize,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<OneEmaResponse> {
        self.get_ema(index, state)
    }

    #[rpc_method(name = "getAllTicks")]
    pub fn get_all_ticks_rpc(
        &self,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<TickDataResponse> {
        self.get_all_ticks(state)
    }

    #[rpc_method(name = "getTick")]
    pub fn get_tick_rpc(
        &self,
        index: usize,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<OneTickResponse> {
        self.get_tick(index, state)
    }
}

// impl<S: Spec> Bank<S> {
//     async fn route_balance(
//         state: ApiState<S, Self>,
//         mut accessor: ApiStateAccessor<S>,
//         Path((token_id, user_address)): Path<(TokenId, S::Address)>,
//     ) -> ApiResult<Coins> {
//         let amount = state
//             .get_balance_of(&user_address, token_id, &mut accessor)
//             .unwrap_infallible()
//             .ok_or_else(|| errors::not_found_404("Balance", user_address))?;

//         Ok(Coins { amount, token_id }.into())
//     }

//     async fn route_total_supply(
//         state: ApiState<S, Self>,
//         mut accessor: ApiStateAccessor<S>,
//         Path(token_id): Path<TokenId>,
//     ) -> ApiResult<Coins> {
//         let amount = state
//             .get_total_supply_of(&token_id, &mut accessor)
//             .unwrap_infallible()
//             .ok_or_else(|| errors::not_found_404("Token", token_id))?;

//         Ok(Coins { amount, token_id }.into())
//     }

//     async fn route_find_token_id(
//         params: Query<types::FindTokenIdQueryParams<S::Address>>,
//     ) -> ApiResult<types::TokenIdResponse> {
//         let token_id = get_token_id::<S>(&params.token_name, &params.sender);
//         Ok(types::TokenIdResponse { token_id }.into())
//     }

//     async fn route_admins(
//         state: ApiState<S, Self>,
//         mut accessor: ApiStateAccessor<S>,
//         Path(token_id): Path<TokenId>,
//     ) -> ApiResult<types::AdminsResponse<S>> {
//         let admins = state
//             .tokens
//             .get(&token_id, &mut accessor)
//             .unwrap_infallible()
//             .ok_or_else(|| errors::not_found_404("Token", token_id))?
//             .admins;
//         Ok(types::AdminsResponse { admins }.into())
//     }
// }

// impl<S: Spec> HasCustomRestApi for Bank<S> {
//     type Spec = S;

//     fn custom_rest_api(&self, state: ApiState<S>) -> axum::Router<()> {
//         axum::Router::new()
//             .route(
//                 "/tokens/:tokenId/balances/:address",
//                 get(Self::route_balance),
//             )
//             .route(
//                 "/tokens/:tokenId/total-supply",
//                 get(Self::route_total_supply),
//             )
//             .route("/tokens/:tokenId/admins", get(Self::route_admins))
//             .route("/tokens", get(Self::route_find_token_id))
//             .with_state(state.with(self.clone()))
//     }

//     fn custom_openapi_spec(&self) -> Option<OpenApi> {
//         let mut open_api: OpenApi =
//             serde_yaml::from_str(include_str!("../openapi-v3.yaml")).expect("Invalid OpenAPI spec");
//         // Because https://github.com/juhaku/utoipa/issues/972
//         for path_item in open_api.paths.paths.values_mut() {
//             path_item.extensions = None;
//         }
//         Some(open_api)
//     }
// }

impl<S: Spec> LookupTable<S> {
    async fn route_get_all_prices(
        state: ApiState<S, Self>,
    ) -> ApiResult<PricesDataResponse> {
        Ok(state.get_all_prices_rpc().unwrap().into())
    }

    async fn route_get_all_aggr_conf_intervals(
        state: ApiState<S, Self>,
    ) -> ApiResult<AggrConfIntervalsResponse> {
        Ok(state.get_all_aggr_conf_intervals_rpc().unwrap().into())
    }

    async fn route_get_price(
        state: ApiState<S, Self>,
        Path(index): Path<usize>,
    ) -> ApiResult<OnePriceDataResponse> {
        Ok(state.get_price_rpc(index).unwrap().into())
    }

    async fn route_get_aggregate_conf_interval(
        state: ApiState<S, Self>,
        Path(index): Path<usize>,
    ) -> ApiResult<OneAggrConfIntervalResponse> {
        Ok(state.get_aggregate_conf_interval_rpc(index).unwrap().into())
    }

    async fn route_get_many_prices(
        state: ApiState<S, Self>,
        Query(indices): Query<Vec<usize>>,
    ) -> ApiResult<PricesDataResponse> {
        Ok(state.get_many_prices_rpc(indices).unwrap().into())
    }

    async fn route_get_many_aggregate_conf_intervals(
        state: ApiState<S, Self>,
        Query(indices): Query<Vec<usize>>,
    ) -> ApiResult<AggrConfIntervalsResponse> {
        Ok(state.get_many_aggregate_conf_intervals_rpc(indices).unwrap().into())
    }

    async fn route_get_all_ema(state: ApiState<S, Self>) -> ApiResult<EmaDataResponse> {
        Ok(state.get_all_ema_rpc().unwrap().into())
    }

    async fn route_get_ema(
        state: ApiState<S, Self>,
        Path(index): Path<usize>,
    ) -> ApiResult<OneEmaResponse> {
        Ok(state.get_ema_rpc(index).unwrap().into())
    }

    async fn route_get_all_ticks(state: ApiState<S, Self>) -> ApiResult<TickDataResponse> {
        Ok(state.get_all_ticks_rpc().unwrap().into())
    }

    async fn route_get_tick(
        state: ApiState<S, Self>,
        Path(index): Path<usize>,
    ) -> ApiResult<OneTickResponse> {
        Ok(state.get_tick_rpc(index).unwrap().into())
    }
}

impl<S: Spec> HasCustomRestApi for LookupTable<S> {
    type Spec = S;

    fn custom_rest_api(&self, state: ApiState<S>) -> axum::Router<()> {
        axum::Router::new()
            .route("/prices", get(Self::route_get_all_prices))
            .route("/aggr-conf-intervals", get(Self::route_get_all_aggr_conf_intervals))
            .route("/prices/:index", get(Self::route_get_price))
            .route(
                "/aggr-conf-intervals/:index",
                get(Self::route_get_aggregate_conf_interval),
            )
            .route("/prices/many", get(Self::route_get_many_prices))
            .route("/aggr-conf-intervals/many", get(Self::route_get_many_aggregate_conf_intervals))
            .route("/ema", get(Self::route_get_all_ema))
            .route("/ema/:index", get(Self::route_get_ema))
            .route("/ticks", get(Self::route_get_all_ticks))
            .route("/ticks/:index", get(Self::route_get_tick))
            .with_state(state.with(self.clone()))
    }
}