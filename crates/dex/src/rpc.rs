use sov_modules_api::{macros::rpc_gen, ApiStateAccessor, Spec};

use spicenet_shared::addresses::TrgId;
use spicenet_shared::{MPGId, MarketProductGroup, TraderRiskGroup};

use crate::Dex;

#[rpc_gen(client, server, namespace = "dex")]
impl<S: Spec> Dex<S> {
    #[rpc_method(name = "getTRG")]
    pub fn get_trg(
        &self,
        state: &mut ApiStateAccessor<S>,
        trg_id: &TrgId<S>,
    ) -> Result<TraderRiskGroup<S>> {
        let trg = self.trader_risk_groups.get(trg_id, state).unwrap();
        Ok(trg.unwrap())
    }

    #[rpc_method(name = "getMPG")]
    pub fn get_mpg(
        &self,
        state: &mut ApiStateAccessor<S>,
        mpg_id: &MPGId,
    ) -> Result<MarketProductGroup<S>> {
        let mpg = self.market_product_groups.get(mpg_id, state).unwrap();
        Ok(mpg.unwrap())
    }
}
