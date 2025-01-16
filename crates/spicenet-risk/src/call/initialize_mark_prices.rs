use anyhow::Result;
use sov_modules_api::{Address, Context, EventEmitter, Spec, TxState};

use spicenet_shared::dex::MarketProductGroup;

use crate::event::Event;
use crate::state::MarkPricesArray;
use crate::RiskModule;
use spicenet_shared::risk::RiskError;

impl<S: Spec> RiskModule<S> {
    pub(crate) fn initialize_mark_prices(
        &self,
        mpg: &MarketProductGroup<S>,
        is_hardcoded_oracle: bool,
        hardcoded_oracle_id: Option<Address<S>>, // TODO(!oracle): change to something like OracleId once oracle is done
        // hardcoded_oracle_type: u8, // TODO(!oracle): change to OracleType once oracle is done
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        // let mpg = self
        //     .dex
        //     .get_mpg(state, &mpg_id)
        //     .or_else(|_| Err(DexError::MarketProductGroupDoesNotExist.into()))?;

        if context.sender().as_ref() != mpg.mpg_authority.as_ref() {
            return Err(RiskError::InvalidAuthority.into());
        }

        match self.mark_prices.get(&mpg.id, state).unwrap() {
            Some(_) => return Err(RiskError::MarkPricesAlreadyInitialized.into()),
            None => (),
        }

        self.mark_prices
            .set(
                &mpg.id,
                &MarkPricesArray {
                    array: Vec::new(),
                    hardcoded_oracle_id: if is_hardcoded_oracle {
                        hardcoded_oracle_id
                    } else {
                        None
                    },
                },
                state,
            )
            .unwrap();

        self.emit_event(
            state,
            Event::MarkPricesInitialized {
                mpg_id: mpg.id,
                is_hardcoded_oracle,
                hardcoded_oracle_id,
            },
        );

        Ok(())
    }
}
