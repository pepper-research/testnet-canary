use sov_modules_api::{Address, Spec};

use spicenet_shared::{Fractional, NAME_LEN};

use crate::bitpair::BitPair;
use crate::state::IsInitialized;
use crate::state::dex::price_ewma::PriceEwma;

use super::{
    DexError,
    ExchangeResult, products::{ComboProduct, OutrightProduct, Product, ProductId, ProductsArray, ProductStatus},
};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub enum MPGType<S: Spec> {
    Uninitialized,
    MPG,
    MPGWithCombos,
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct MarketProductGroup<S: Spec> {
    // Address
    pub address: Address<S>,

    /// MPG Type
    pub mpg_type: MPGType<S>,

    /// Name of the Market Product Group.
    pub name: [u8; NAME_LEN],

    /// Collected fees till date. This means that each MPG has a separate fee collector authority.
    pub collected_fees: Fractional,

    /// Standard market decimals for products in MPG.
    pub decimals: u64,

    ///
    pub active_flags_products: BitPair,
    pub ewma_windows: [u64; 4],

    /// Num active products
    pub active_products: ProductsArray<S>,

    /// Maximum fees(in BPS) that can be charged from makers.
    pub max_maker_fee_bps: i16,

    /// Minimum fees(in BPS) that can be charged from makers.
    pub min_maker_fee_bps: i16,

    /// Maximum fees(in BPS) that can be charged from takers.
    pub max_taker_fee_bps: i16,

    /// Minimum fees(in BPS) that can be charged from takers.
    pub min_taker_fee_bps: i16,
    pub sequence_number: u128,

    /// If set to `true`, the MPG is considered to be dysfunctional.
    pub is_mpg_killed: bool,

    /// If set to `true`, the MPG is controlled by a central admin authority.
    pub in_admin_mode: bool,

    // added fields
    /// Fee model configuration for the MPG.
    pub fee_model_config: FeeModelConfiguration, // TODO: make type (struct)

    /// Risk engine configuration for the MPG.
    pub risk_model_config: RiskModelConfiguration, // TODO: make type (struct)

    /// Output register/log for fee collections.
    pub fee_output_register: FeeOutputRegister, // TODO: make type (struct)

    /// Output register/log for risk engine statuses
    pub risk_output_register: RiskOutputRegister, // TODO: make type (struct)

    // module ids for risk and fee modules.
    pub risk_engine_module_id: RiskEngineModuleId, // TODO: fill in type from module
    pub fee_model_module_id: FeeModelModuleId,     // TODO: fill in type from module
}

impl<S: Spec> IsInitialized for MarketProductGroup<S> {
    fn is_initialized(&self) -> bool {
        match self.mpg_type {
            MPGType::Uninitialized => false,
            _ => true,
        }
    }
}

impl<S: Spec> MarketProductGroup<S> {
    pub fn are_mpg_instruments_expiring_or_expired(&self, product: &Product<S>) -> bool {
        match product {
            Product::Outright { outright_product } => outright_product.is_expired_or_expiring(),
            Product::Combo { combo_product } => combo_product.legs().iter().any(|leg| {
                self.active_products[leg.product_index]
                    .try_to_outright()
                    .unwrap()
                    .is_expired_or_expiring()
            }),
        }
    }

    pub fn is_expiring(&self, product: &Product<S>) -> bool {
        match product {
            Product::Outright { outright_product } => outright_product.is_expired(),
            Product::Combo { combo_product } => combo_product.legs().iter().any(|leg| {
                self.active_products[leg.product_index]
                    .try_to_outright()
                    .unwrap()
                    .is_expired()
            }),
        }
    }

    /// Finds the product indices using the `get_active_products` fn, which fetches the active market products using the `get_product_status` fn.
    /// Then, we find the product Id of the found products, and match it with the product key param provided, to obtain the result(a combination of the product id and the Product)
    pub fn find_product_index(&self, product_key: &ProductId) -> Option<(usize, &Product<S>)> {
        self.get_active_products().find(|(_, product)| {
            let product_id = match product {
                Product::Outright { outright_product } => &outright_product.product_id,
                Product::Combo { combo_product } => &combo_product.product_id,
            };

            *product_id == *product_key
        })
    }

    /// Finds the product indices using the active products array present in the MarketProductGroup metadata.
    /// Then, we find the product Id of the found products, and match it with the product key param provided, to obtain the result(a combination of the product id and the Product)
    pub fn find_product_index_among_all(
        &self,
        product_key: &ProductId,
    ) -> ExchangeResult<(usize, &Product<S>)> {
        self.active_products
            .iter()
            .enumerate()
            .find(|(_, product)| {
                // im quite sure of this logic
                let product_id = match product {
                    Product::Outright { outright_product } => &outright_product.product_id,
                    Product::Combo { combo_product } => &combo_product.product_id,
                };

                *product_id == *product_key
            })
            .ok_or(DexError::MissingMarketProduct.into())
    }

    /// Finds the product id and Product using the `find_product_index` fn. Then, try to convert the product using the `try_to_outright` fn.
    /// If outright, we construct the `OutrightProduct` and return it. Otherwise, the `ExchangeResult` type handles it.
    pub fn find_outright(
        &self,
        product_key: &ProductId,
    ) -> ExchangeResult<(usize, &OutrightProduct<S>)> {
        let (idx, product) = self.find_product_index(product_key)?;

        Ok((idx, product.try_to_outright()?))
    }

    /// Finds the product id and Product using the `find_product_index_among_all` fn. Then, try to convert the product using the `try_to_outright` fn.
    /// If outright, we construct the `OutrightProduct` and return it. Otherwise, the `ExchangeResult` type handles it.
    pub fn find_outright_among_all(
        &self,
        product_key: &ProductId,
    ) -> ExchangeResult<(usize, &OutrightProduct<S>)> {
        let (idx, product) = self.find_product_index_among_all(product_key)?;

        Ok((idx, product.try_to_outright()?))
    }

    /// Finds the product id and Product using the `find_product_index_among_all` fn. Then, try to convert the product using the `try_to_combo` fn.
    /// If combo, we construct the `ComboProduct` and return it. Otherwise, the `ExchangeResult` type handles it.
    pub fn find_combo(&self, product_key: &ProductId) -> ExchangeResult<(usize, &ComboProduct<S>)> {
        let (idx, product) = self.find_product_index(product_key)?;

        Ok((idx, product.try_to_combo()?))
    }

    /// Returns the active products by first obtaining the status of the product using the `get_product_status` fn and filtering products whose status is Ininitialized.
    pub fn get_active_products(&self) -> impl Iterator<Item = (usize, &Product<S>)> {
        self.active_products
            .iter()
            .enumerate()
            .filter(|(_idx, product)| {
                product.get_product_status(self) == ProductStatus::Initialized
            })
    }

    /// Returns the active and expiring products by first obtaining the status of the product using the `get_product_status` fn and filtering products whose status is Ininitialized or Expiring
    pub fn get_active_and_expiring_products(&self) -> impl Iterator<Item = (usize, &Product<S>)> {
        self.active_products
            .iter()
            .enumerate()
            .filter(|(_idx, product)| {
                let status = product.get_product_status(self);
                status == ProductStatus::Initialized || status == ProductStatus::Expiring
                //
            })
    }

    /// Returns the expiring products by first obtaining the status of the product using the `get_product_status` fn and filtering products whose status is Expiring.
    pub fn get_expiring_products(&self) -> impl Iterator<Item = (usize, &Product<S>)> {
        self.active_products
            .iter()
            .enumerate()
            .filter(|(_idx, product)| product.get_product_status(self) == ProductStatus::Expiring)
    }

    /// Returns the active outright products by first obtaining the status of the product and trying to construct the Outright product using
    /// the `try_to_outright` fn.
    pub fn get_active_outrights(&self) -> impl Iterator<Item = (usize, &OutrightProduct<S>)> {
        self.get_active_products()
            .filter_map(|(idx, product)| Some((idx, product.try_to_outright().ok()?)))
    }

    /// Returns the active and expired outright products by first obtaining the status of the product and trying to construct the Outright product using
    /// the `try_to_outright` fn. Then, we break if the `outright.is_uninitialized` fn returns false. Otherwise, we return the outright product
    pub fn get_active_and_expired_outrights(
        &self,
    ) -> impl Iterator<Item = (usize, &OutrightProduct<S>)> {
        self.active_products
            .iter()
            .enumerate()
            .filter_map(|(idx, product)| {
                let outright = product.try_to_outright().ok()?;
                if outright.is_unitialized() {
                    None
                } else {
                    Some((idx, outright))
                }
            })
    }

    /// Returns the expiring outright products by first obtaining the status of the product and trying to construct the Outright product using
    /// the `try_to_outright` fn. Then, we return the outright product if the product status is expiring, or break.
    pub fn get_expiring_outrights(&self) -> impl Iterator<Item = (usize, &OutrightProduct<S>)> {
        self.active_products
            .iter()
            .enumerate()
            .filter_map(|(idx, product)| {
                let outright = product.try_to_outright().ok()?;
                if outright.product_status == ProductStatus::Expiring {
                    Some((idx, outright))
                } else {
                    None
                }
            })
    }

    pub fn active_combo_products(&self) -> impl Iterator<Item = (usize, &ComboProduct<S>)> {
        self.get_active_products()
            .filter_map(|(idx, product)| Some((idx, product.try_to_combo().ok()?)))
    }

    pub fn active_and_expiring_combos(&self) -> impl Iterator<Item = (usize, &ComboProduct<S>)> {
        self.get_active_products().filter_map(|(idx, product)| {
            let combo_product = product.try_to_combo().ok()?;
            let combo_status = combo_product.get_combo_status(self);

            if combo_status == ProductStatus::Uninitialized {
                None
            } else {
                Some((idx, combo_product))
            }
        })
    }

    pub fn get_expired_products(&self) -> impl Iterator<Item = (usize, &Product<S>)> {
        self.active_products
            .iter()
            .enumerate()
            .filter_map(|(idx, product)| {
                if product.get_product_status(self) == ProductStatus::Expired {
                    Some((idx, product))
                } else {
                    None
                }
            })
    }

    /// We first obtain the product using the `find_product_index_among_all` and passing in trhe product key. Then, we construct the outright product and mark it's status as expired after.
    pub fn deactivate_product(&mut self, product_key: ProductId) -> ExchangeResult {
        let (idx, _) = self.find_product_index_among_all(&product_key)?;
        if let Ok(outright_product) = self.active_products[idx].try_to_outright_mut() {
            outright_product.product_status == ProductStatus::Expired;
        }
        Ok(())
    }

    /// We first obtain the product using the `find_product_index_among_all` and passing in trhe product key. Then, we construct the outright product,
    /// and assert if the product is expired using it's product status. If not, we return a `DexError`, otherwise, we set the default value to the product at that specific `idx`
    pub fn remove_product(&mut self, product_key: &ProductId) -> ExchangeResult {
        let (idx, _) = self.find_product_index_among_all(&product_key)?;
        if let Ok(outright) = self.active_products[idx].try_to_outright_mut() {
            assert_eq!(outright.is_expired(), true);
            // do we need this?
            if !outright.is_expired() {
                return DexError::ContractIsNotExpired.into();
            }
        }
        self.active_products[idx] = Default::default();
        Ok(())
    }

    /// We first check if there is a product existing and if so, return a `DexError`, otherwise, we fetch the product whose status is uninitialized, and return its `idx`, thereby adding a new product.
    pub fn add_product(&mut self, product: &Product<S>) -> ExchangeResult {
        let p = self.get_active_products().find(|(_, p)| p == &product);
        if p.is_some() {
            return Err(DexError::DuplicateProductNameError.into());
        }

        let index = match self
            .active_products
            .iter()
            .enumerate()
            .find(|(_idx, product)| {
                product.get_product_status(self) == ProductStatus::Uninitialized
            }) {
            Some((idx, _product)) => idx,
            None => return Err(DexError::FullMarketProductGroup.into()),
        };

        self.active_products[index] = **product;
        Ok(())
    }

    pub fn get_product_prices(&mut self, idx: usize) -> &mut PriceEwma<S> {
        match &mut self.active_products[idx] {
            Product::Outright { outright_product } => &mut outright_product.metadata.prices,
            Product::Combo { combo_product } => &mut combo_product.metadata.prices,
        }
    }
}
