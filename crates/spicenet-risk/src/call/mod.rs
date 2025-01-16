use sov_modules_api::{Address, Spec};

use spicenet_shared::dex::{MarketProductGroup, TraderRiskGroup};
use spicenet_shared::{FastInt, ProductId};
use update_mark_prices::ProductMarkPriceUpdate;

pub mod collect_mark_prices_garbage;
pub mod delete_mark_prices;
pub mod initialize_covariance_matrix;
pub mod initialize_mark_prices;
pub mod internal;
pub mod remove_market_product_index_from_variance_cache;
pub mod update_covariance_matrix;
pub mod update_mark_prices;
// pub mod update_risk_authority;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(bound = "S::Address: ::schemars::JsonSchema", rename = "CallMessage")
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
pub enum CallMessage<S: Spec> {
    InitializeCovarianceMatrix {
        mpg: MarketProductGroup<S>,
    },
    InitializeMarkPrices {
        mpg: MarketProductGroup<S>,
        is_hardcoded_oracle: bool,
        hardcoded_oracle_id: Option<Address<S>>,
    },
    UpdateCovarianceMatrix {
        mpg: MarketProductGroup<S>,
        product_keys: Vec<ProductId>,
        standard_deviations: Vec<FastInt>,
        correlations: Vec<Vec<FastInt>>,
    },
    UpdateMarkPrices {
        mpg: MarketProductGroup<S>,
        products_to_update: Vec<ProductMarkPriceUpdate>,
    },
    CollectMarkPricesGarbage {
        mpg: MarketProductGroup<S>,
        max_products_to_examine: u8,
    },
    RemoveMarketProductIndexFromVarianceCache {
        mpg: MarketProductGroup<S>,
        trg: TraderRiskGroup<S>,
        market_product_index: usize,
    },
    DeleteMarkPrices {
        mpg: MarketProductGroup<S>,
    },
}
