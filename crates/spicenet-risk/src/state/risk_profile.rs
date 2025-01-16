use sov_modules_api::Spec;

use spicenet_shared::{FastInt, ZERO_FAST_INT};

use spicenet_shared::dex::TraderRiskGroup;
use spicenet_shared::risk::{HealthStatus, VarianceCache, MAX_TRADER_POSITIONS};

use crate::utils::babylonian_sqrt;

/// The minimum threshold required to place open orders
pub const ORDER_PLACEMENT_SDS: FastInt = FastInt {
    value: 3_000_000_i128,
};

/// The minimum threshold to be safe from liquidation
pub const LIQUIDATION_SDS: FastInt = FastInt {
    value: 1_500_000_i128,
};

/// The liquidation price is set to LIQUIDATION_PRICE_PROPORTION * portfolio_value
pub const LIQUIDATION_PRICE_PROPORTION: FastInt = FastInt {
    value: 333_333_i128,
};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct RiskProfile {
    pub net_cash: FastInt,
    pub pnl: FastInt,
    pub position_value: FastInt,
    pub portfolio_value: FastInt,
    pub portfolio_std_dev: FastInt,
    pub portfolio_open_order_std_dev: FastInt,
    // abs_position_value: [FastInt; MAX_TRADER_POSITIONS],
    pub abs_position_value: Vec<FastInt>,
    pub total_abs_position_value: FastInt,
    pub deposited_collateral: FastInt,
}

impl RiskProfile {
    /// The threshold, which when broken, can cause the account to be deemed liquidatable
    pub fn get_liquidation_threshold(&self) -> FastInt {
        self.portfolio_std_dev * LIQUIDATION_SDS
    }

    /// Calculates liquidation threshold as a % of portfolio value. Higher the ratio, higher the risk.
    pub fn get_risk_ratio(&self) -> FastInt {
        self.get_liquidation_threshold() / self.portfolio_value
    }

    /// The threshold until which open orders can be allowed
    pub fn get_order_placement_threshold(&self) -> FastInt {
        self.portfolio_open_order_std_dev / ORDER_PLACEMENT_SDS
    }

    pub fn get_health_status(&self) -> HealthStatus {
        let liquidation_value = self.get_liquidation_threshold();
        let health_value = self.get_order_placement_threshold();

        if self.portfolio_value < liquidation_value {
            HealthStatus::Liquidatable
        } else if self.portfolio_value < health_value {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Healthy
        }
    }
}

impl<S: Spec> From<(&VarianceCache, &TraderRiskGroup<S>)> for RiskProfile {
    fn from(t: (&VarianceCache, &TraderRiskGroup<S>)) -> Self {
        let abs_position_value = [ZERO_FAST_INT; MAX_TRADER_POSITIONS];
        let total_abs_position_value = ZERO_FAST_INT;

        let (variance_cache, trader_risk_group) = t;

        let net_cash = FastInt::from(trader_risk_group.cash_balance)
            + FastInt::from(trader_risk_group.pending_cash_balance);

        // total portfolio value (position + net_cash)
        let portfolio_value = variance_cache.derivative_position_value + net_cash;

        // amount of currently deposited collateral
        let deposited_collateral = FastInt::from(trader_risk_group.total_deposited)
            - FastInt::from(trader_risk_group.total_withdrawn);

        // current trader PnL
        let pnl = portfolio_value - deposited_collateral;

        // one day standard deviation of absolute returns of the portfolio
        // TODO: this sucks, hacking in for now
        let portfolio_std_dev =
            FastInt::from(babylonian_sqrt(variance_cache.total_variance_traded.to_f64()).unwrap());

        // one day standard deviation of absolute returns of the portfolio + worst case open order per instrument
        let portfolio_open_order_std_dev = FastInt::from(
            babylonian_sqrt(
                (variance_cache.total_variance_traded + variance_cache.open_order_variance)
                    .to_f64(),
            )
            .unwrap(),
        );

        RiskProfile {
            net_cash,
            pnl,
            position_value: variance_cache.derivative_position_value,
            portfolio_value,
            portfolio_std_dev,
            portfolio_open_order_std_dev,
            abs_position_value: abs_position_value.to_vec(),
            total_abs_position_value,
            deposited_collateral,
        }
    }
}
