use sov_modules_api::{Address, Spec};

use spicenet_shared::risk::{HealthOutput, RiskInfo};
use spicenet_shared::{FastInt, MPGId, ProductId, TrgId};

#[derive(
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
    Debug,
    PartialEq,
    Clone,
)]
#[serde(
    bound = "Address<S>: serde::Serialize + serde::de::DeserializeOwned, TrgId<S>: serde::Serialize + serde::de::DeserializeOwned"
)]
pub enum Event<S: Spec> {
    CovarianceMatrixInitialized {
        mpg_id: MPGId,
    },
    MarkPricesInitialized {
        mpg_id: MPGId,
        is_hardcoded_oracle: bool,
        hardcoded_oracle_id: Option<Address<S>>, // TODO(!oracle): change to something like OracleId once oracle is done
    },
    CovarianceMatrixUpdated {
        mpg_id: MPGId,
        product_keys: Vec<ProductId>,
        standard_deviations: Vec<FastInt>,
        correlations: Vec<Vec<FastInt>>,
    },
    MarketProductIndexRemovedFromVarianceCache {
        mpg_id: MPGId,
        trg_id: TrgId<S>,
        market_product_index: usize,
    },
    MarkPricesDeleted {
        mpg_id: MPGId,
    },
    MarkPricesGarbageCollected {
        mpg_id: MPGId,
        max_products_to_examine: u8,
    },
    AccountHealthValidation {
        mpg_id: MPGId,
        trg_id: TrgId<S>,
        risk_info_params: RiskInfo,
        block_withdrawal: bool,
        health_output: HealthOutput,
    },
    AccountLiquidationValidation {
        mpg_id: MPGId,
        trg_id: TrgId<S>,
        risk_info_params: RiskInfo,
        health_output: HealthOutput,
    },
    VarianceCacheInitialized {
        trg_id: TrgId<S>,
    },
    VarianceCacheDeleted {
        trg_id: TrgId<S>,
    },
}
