use jsonrpsee::core::RpcResult;
use sov_modules_api::rest::utils::{ApiResult, Path, Query};
use sov_modules_api::rest::{ApiState, HasCustomRestApi};
use sov_modules_api::{macros::rpc_gen, ApiStateAccessor, Spec};
use spicenet_shared::TrgId;
use axum::routing::get;
use sov_modules_api::prelude::axum;

use spicenet_shared::risk::VarianceCache;

use crate::RiskModule;

#[rpc_gen(client, server, namespace = "risk")]
impl<S: Spec> RiskModule<S> {
    #[rpc_method(name = "getVarianceCache")]
    pub fn get_variance_cache(
        &self,
        state: &mut ApiStateAccessor<S>,
        trg_id: TrgId<S>,
    ) -> RpcResult<VarianceCache> {
        let variance_cache = self.variance_caches.get(&trg_id, state).unwrap();
        Ok(variance_cache.unwrap())
    }
}

impl<S: Spec> HasCustomRestApi for RiskModule<S> {
    type Spec = S;

    fn custom_rest_api(&self, state: ApiState<S>) -> axum::Router<()> {
        axum::Router::new()
            .route("/risk/variance-cache/:trgId", get(Self::get_variance_cache_rest))
            .with_state(state.with(self.clone()))
    }
}

impl<S: Spec> RiskModule<S> {
    async fn get_variance_cache_rest(
        state: ApiState<S, Self>,
        mut accessor: ApiStateAccessor<S>,
        Path(trg_id): Path<TrgId<S>>,
    ) -> ApiResult<VarianceCache> {
        let variance_cache = state.get_variance_cache(trg_id).unwrap();
        Ok(variance_cache.into())
    }
}