use crate::fixed_ring_buffer::FixedRingBuffer;
use crate::{Event, LookupTable};
use anyhow::{bail, Result};
use sov_modules_api::{Context, EventEmitter, Spec, TxState};
use spicenet_shared::db::price_ticks::insert_price_ticks;
use spicenet_shared::oracle::NUMBER_OF_MARKETS;
use spicenet_shared::{Fractional, ZERO_FRAC};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    schemars(rename = "CallMessage"),
    derive(sov_modules_api::macros::UniversalWallet)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
pub enum CallMessage {
    // Mutate All
    //
    // Fields:
    // - 'prices' - Array of prices
    // - 'aggregate_conf_intervals' - Array of aggregate confidence intervals in same order of corresponding prices
    MutateAll {
        prices: [Fractional; NUMBER_OF_MARKETS],
        aggregate_conf_intervals: [u32; NUMBER_OF_MARKETS],
    },
}

pub const EMA_PERIOD: usize = 60 * 60; // 1 hour

pub fn calculate_initial_ema(
    prices: &FixedRingBuffer<[Fractional; NUMBER_OF_MARKETS], EMA_PERIOD>,
    aggregate_conf_intervals: &FixedRingBuffer<[u32; NUMBER_OF_MARKETS], EMA_PERIOD>,
) -> Result<[Fractional; NUMBER_OF_MARKETS]> {
    let multiplier = Fractional::from(2).checked_div(Fractional::from(EMA_PERIOD + 1))?;

    let mut ema = [ZERO_FRAC; NUMBER_OF_MARKETS];
    for i in 0..NUMBER_OF_MARKETS {
        for j in 0..EMA_PERIOD {
            let weight = Fractional::from(aggregate_conf_intervals.get(j).unwrap()[i])
                .checked_mul(multiplier)?;
            ema[i] = prices.get(j).unwrap()[i]
                .checked_sub(ema[i])?
                .checked_mul(weight)?
                .checked_add(ema[i])?;
        }
    }
    Ok(ema)
}

pub fn update_ema(
    prices: &[Fractional; NUMBER_OF_MARKETS],
    aggregate_conf_intervals: &[u32; NUMBER_OF_MARKETS],
    ema: [Fractional; NUMBER_OF_MARKETS],
) -> Result<[Fractional; NUMBER_OF_MARKETS]> {
    let multiplier = Fractional::from(2).checked_div(Fractional::from(EMA_PERIOD + 1))?;

    let mut new_ema = [ZERO_FRAC; NUMBER_OF_MARKETS];
    for i in 0..NUMBER_OF_MARKETS {
        let weight = Fractional::from(aggregate_conf_intervals[i]).checked_mul(multiplier)?;
        new_ema[i] = prices[i]
            .checked_sub(ema[i])?
            .checked_mul(weight)?
            .checked_add(ema[i])?;
    }
    Ok(new_ema)
}

impl<S: Spec> LookupTable<S> {
    pub(crate) fn update_state(
        &self,
        prices: [Fractional; NUMBER_OF_MARKETS],
        aggregate_conf_intervals: [u32; NUMBER_OF_MARKETS],
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        if context.sender().as_ref() != self.update_authority.get(state)?.unwrap().as_ref() {
            bail!("Sender is not the update authority");
        }

        self.prices.set(&prices, state)?;

        self.aggregate_conf_intervals
            .set(&aggregate_conf_intervals, state)?;

        let last_tick_timestamp = self.last_tick_timestamp.get(state)?;

        let timestamp = self.time_module.get_time(state)?.unix_timestamp;

        if last_tick_timestamp.is_none() || timestamp - last_tick_timestamp.unwrap() > 1000 {
            let mut price_ticks = self.price_ticks.get(state)?.unwrap();
            let mut aggregate_conf_interval_ticks =
                self.aggregate_conf_interval_ticks.get(state)?.unwrap();

            price_ticks.push_or_overwrite(prices);
            aggregate_conf_interval_ticks.push_or_overwrite(aggregate_conf_intervals);

            // TODO(dependent on new products structure): insert data to database

            self.price_ticks.set(&price_ticks, state).unwrap();
            self.aggregate_conf_interval_ticks
                .set(&aggregate_conf_interval_ticks, state)?
        }

        if self.price_emas.get(state)?.is_none() {
            let price_ticks = self.price_ticks.get(state)?.unwrap();
            let aggregate_conf_interval_ticks =
                self.aggregate_conf_interval_ticks.get(state)?.unwrap();

            let ema = calculate_initial_ema(&price_ticks, &aggregate_conf_interval_ticks)?;
            self.price_emas.set(&ema, state)?;
        } else {
            let prev_ema = self.price_emas.get(state)?.unwrap();
            let ema = update_ema(&prices, &aggregate_conf_intervals, prev_ema)?;
            self.price_emas.set(&ema, state)?;
        }

        insert_price_ticks(timestamp, prices, aggregate_conf_intervals);

        self.emit_event(
            state,
            Event::MutateAll {
                prices,
                aggregate_conf_intervals,
            },
        );

        self.aggregate_conf_intervals
            .set(&aggregate_conf_intervals, state)?;

        let last_tick_timestamp = self.last_tick_timestamp.get(state)?;

        let timestamp = self.time_module.get_time(state)?.unix_timestamp;

        if last_tick_timestamp.is_none() || timestamp - last_tick_timestamp.unwrap() > 1000 {
            let mut price_ticks = self.price_ticks.get(state)?.unwrap();
            let mut aggregate_conf_interval_ticks =
                self.aggregate_conf_interval_ticks.get(state)?.unwrap();

            price_ticks.push_or_overwrite(prices);
            aggregate_conf_interval_ticks.push_or_overwrite(aggregate_conf_intervals);

            // TODO(dependent on new products structure): insert data to database

            self.price_ticks.set(&price_ticks, state).unwrap();
            self.aggregate_conf_interval_ticks
                .set(&aggregate_conf_interval_ticks, state)?
        }

        if self.price_emas.get(state)?.is_none() {
            let price_ticks = self.price_ticks.get(state)?.unwrap();
            let aggregate_conf_interval_ticks =
                self.aggregate_conf_interval_ticks.get(state)?.unwrap();

            let ema = calculate_initial_ema(&price_ticks, &aggregate_conf_interval_ticks)?;
            self.price_emas.set(&ema, state)?;
        } else {
            let prev_ema = self.price_emas.get(state)?.unwrap();
            let ema = update_ema(&prices, &aggregate_conf_intervals, prev_ema)?;
            self.price_emas.set(&ema, state)?;
        }

        insert_price_ticks(timestamp, prices, aggregate_conf_intervals);

        self.emit_event(
            state,
            Event::MutateAll {
                prices,
                aggregate_conf_intervals,
            },
        );

        Ok(())
    }
}
