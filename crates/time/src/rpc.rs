use axum::routing::get;
use jsonrpsee::core::RpcResult;
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::prelude::axum;
use sov_modules_api::rest::utils::{ApiResult, Path};
use sov_modules_api::rest::{ApiState, HasCustomRestApi};
use sov_modules_api::{ApiStateAccessor, Spec, StateReader};
use sov_state::User;

use crate::{Slot, TimeModule};

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TimeResponse {
    pub unix_timestamp: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SlotResponse {
    pub slot: Slot,
}

impl<S: Spec> TimeModule<S> {
    pub fn get_time<Reader: StateReader<User>>(
        &self,
        state: &mut Reader,
    ) -> RpcResult<TimeResponse> {
        let unix_timestamp = self.unix_timestamp.get(state).unwrap();

        Ok(TimeResponse {
            unix_timestamp: unix_timestamp.unwrap(),
        })
    }

    pub fn get_slot<Reader: StateReader<User>>(
        &self,
        state: &mut Reader,
    ) -> RpcResult<SlotResponse> {
        let slot = self.slot.get(state).unwrap();

        Ok(SlotResponse {
            slot: slot.unwrap(),
        })
    }

    pub fn get_slot_and_time<Reader: StateReader<User>>(
        &self,
        state: &mut Reader,
    ) -> RpcResult<(SlotResponse, TimeResponse)> {
        let slot = self.slot.get(state).unwrap();
        let unix_timestamp = self.unix_timestamp.get(state).unwrap();

        Ok((
            SlotResponse {
                slot: slot.unwrap(),
            },
            TimeResponse {
                unix_timestamp: unix_timestamp.unwrap(),
            },
        ))
    }
}

#[rpc_gen(client, server, namespace = "time")]
impl<S: Spec> TimeModule<S> {
    #[rpc_method(name = "getTime")]
    pub fn get_time_rpc(&self, state: &mut ApiStateAccessor<S>) -> RpcResult<TimeResponse> {
        let unix_timestamp = self.unix_timestamp.get(state).unwrap();

        Ok(TimeResponse {
            unix_timestamp: unix_timestamp.unwrap(),
        })
    }

    #[rpc_method(name = "getSlot")]
    pub fn get_slot_rpc(&self, state: &mut ApiStateAccessor<S>) -> RpcResult<SlotResponse> {
        let slot = self.slot.get(state).unwrap();

        Ok(SlotResponse {
            slot: slot.unwrap(),
        })
    }

    #[rpc_method(name = "getSlotAndTime")]
    pub fn get_slot_and_time_rpc(
        &self,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<(SlotResponse, TimeResponse)> {
        let slot = self.slot.get(state).unwrap();
        let unix_timestamp = self.unix_timestamp.get(state).unwrap();

        Ok((
            SlotResponse {
                slot: slot.unwrap(),
            },
            TimeResponse {
                unix_timestamp: unix_timestamp.unwrap(),
            },
        ))
    }
}

impl<S: Spec> TimeModule<S> {
    async fn route_time(
        state: ApiState<S, Self>,
        mut accessor: ApiStateAccessor<S>,
        Path(()): Path<()>,
    ) -> ApiResult<TimeResponse> {
        let unix_timestamp = state.get_time(&mut accessor).unwrap();

        Ok(TimeResponse {
            unix_timestamp: unix_timestamp.unix_timestamp,
        }
        .into())
    }

    async fn route_slot(
        state: ApiState<S, Self>,
        mut accessor: ApiStateAccessor<S>,
        Path(()): Path<()>,
    ) -> ApiResult<SlotResponse> {
        let slot = state.get_slot(&mut accessor).unwrap();

        Ok(SlotResponse { slot: slot.slot }.into())
    }
}

impl<S: Spec> HasCustomRestApi for TimeModule<S> {
    type Spec = S;

    fn custom_rest_api(&self, state: ApiState<S>) -> axum::Router<()> {
        axum::Router::new()
            .route("/time/unix-timestamp", get(Self::route_time))
            .route("/time/slot", get(Self::route_slot))
            .with_state(state.with(self.clone()))
    }
}
