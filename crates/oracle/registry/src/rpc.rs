use jsonrpsee::core::RpcResult;
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::{Address, ApiStateAccessor, Spec, StateReader};
use thiserror::Error;

use crate::OracleRegistry;

// #[derive(Debug, Error)]
// enum RpcError {
//     #[error("The oracle node is not found.")]
//     OracleNodeNotFound,
// }

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
#[serde(bound = "S: Spec")]
pub struct OracleNodeResponse<S: Spec> {
    pub amount_staked: Option<u64>,
    pub accumulated_penalty: Option<u64>,
    pub address: Option<Address<S>>,
}

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
#[serde(bound = "S: Spec")]
pub struct WhitelistedUserResponse<S: Spec> {
    pub address: Option<Address<S>>,
    pub whitelisted_ts: Option<u64>,
    pub is_oracle_node: Option<bool>,
}

#[rpc_gen(client, server, namespace = "oracle_registry")]
impl<S: Spec> OracleRegistry<S> {
    #[rpc_method(name = "getOracleNode")]
    pub fn get_oracle_node(
        &self,
        state: &mut ApiStateAccessor<S>,
        address: Address<S>,
    ) -> RpcResult<OracleNodeResponse<S>> {
        let oracle_node = self.oracle_nodes.get(&address, state).unwrap();

        Ok(OracleNodeResponse {
            amount_staked: oracle_node.as_ref().map(|node| node.amount_staked),
            accumulated_penalty: oracle_node.as_ref().map(|node| node.accumulated_penalty),
            address: oracle_node.as_ref().map(|node| node.address),
        })
    }

    #[rpc_method(name = "getWhitelistedUser")]
    pub fn get_whitelisted_user(
        &self,
        state: &mut ApiStateAccessor<S>,
        address: Address<S>,
    ) -> RpcResult<WhitelistedUserResponse<S>> {
        let whitelisted_user = self.whitelisted_users.get(&address, state).unwrap();

        Ok(WhitelistedUserResponse {
            address: whitelisted_user.as_ref().map(|user| user.address),
            whitelisted_ts: whitelisted_user.as_ref().map(|user| user.whitelisted_ts),
            is_oracle_node: whitelisted_user.as_ref().map(|user| user.is_oracle_node),
        })
    }
}
