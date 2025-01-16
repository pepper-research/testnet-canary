use sov_modules_api::{
    Context, Error, GenesisState, Module, ModuleId, ModuleInfo, Spec, StateMap,
    TxState, DaSpec,
};

use lut::LookupTable;
use spicenet_shared::{MPGId, ProductId, TrgId};
use spicenet_time::TimeModule;

use crate::call::CallMessage;
// use crate::event::Event;
use crate::state::{CovarianceMatrix, MarkPricesArray, RiskProfile};

use spicenet_shared::risk::{RiskEngineOutput, SocialLossInfo, VarianceCache};

pub mod bitpair;
pub mod call;
pub mod event;
pub mod genesis;
pub mod helpers;
pub mod rpc;
pub mod state;
mod two_iterators;
pub mod utils;

#[derive(Clone, ModuleInfo, sov_modules_api::ModuleRestApi)]
pub struct RiskModule<S: Spec> {
    #[id]
    id: ModuleId,

    #[state]
    covariance_matrix: StateMap<MPGId, CovarianceMatrix>,

    #[state]
    mark_prices: StateMap<MPGId, MarkPricesArray<S>>,

    #[state]
    variance_caches: StateMap<TrgId<S>, VarianceCache>,

    #[module]
    time_module: TimeModule<S>,

    #[module]
    lut: LookupTable<S>,
}

impl<S: Spec> Module for RiskModule<S> {
    type Spec = S;
    type Config = genesis::RiskModuleConfig;
    type CallMessage = call::CallMessage<S>;
    type Event = event::Event<S>;

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
        msg: CallMessage<S>,
        context: &Context<Self::Spec>,
        state: &mut impl TxState<S>,
    ) -> Result<(), Error> {
        let call_result = match msg {
            CallMessage::InitializeCovarianceMatrix { mpg } => {
                self.initialize_covariance_matrix(&mpg, context, state)
            }

            CallMessage::InitializeMarkPrices {
                mpg,
                hardcoded_oracle_id,
                is_hardcoded_oracle,
            } => self.initialize_mark_prices(
                &mpg,
                is_hardcoded_oracle,
                hardcoded_oracle_id,
                context,
                state,
            ),

            CallMessage::UpdateCovarianceMatrix {
                mpg,
                correlations,
                product_keys,
                standard_deviations,
            } => self.update_covariance_matrix(
                &mpg,
                product_keys,
                standard_deviations,
                correlations,
                context,
                state,
            ),

            CallMessage::UpdateMarkPrices {
                mpg,
                products_to_update,
            } => self.update_mark_prices(&mpg, products_to_update, context, state),

            CallMessage::CollectMarkPricesGarbage {
                mpg,
                max_products_to_examine,
            } => self.collect_mark_prices_garbage(&mpg, max_products_to_examine, context, state),

            CallMessage::RemoveMarketProductIndexFromVarianceCache {
                trg,
                mpg,
                market_product_index,
            } => self.remove_market_product_index_from_variance_cache(
                &mpg,
                &trg,
                market_product_index,
                context,
                state,
            ),

            CallMessage::DeleteMarkPrices { mpg } => self.delete_mark_prices(&mpg, context, state),
        };

        Ok(call_result?)
    }
}
