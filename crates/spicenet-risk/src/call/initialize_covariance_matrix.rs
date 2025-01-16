use anyhow::Result;
use sov_modules_api::{Context, EventEmitter, Spec, TxState};

use spicenet_shared::dex::MarketProductGroup;

use crate::event::Event;
use crate::state::{CorrelationMatrix, CovarianceMatrix};
use crate::RiskModule;
use spicenet_shared::risk::covariance_metadata::CovarianceMetadata;
use spicenet_shared::risk::RiskError;

impl<S: Spec> RiskModule<S> {
    pub(crate) fn initialize_covariance_matrix(
        &self,
        mpg: &MarketProductGroup<S>,
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

        match self.covariance_matrix.get(&mpg.id, state).unwrap() {
            Some(_) => return Err(RiskError::CovarianceMatrixAlreadyInitialized.into()),
            None => (),
        }

        self.covariance_matrix
            .set(
                &mpg.id,
                &CovarianceMatrix {
                    covariance_metadata: CovarianceMetadata {
                        update_slot: 0, // 0 indicates never updated, should be ok
                        num_active_products: 0,
                        product_keys: Vec::new(),
                        standard_deviations: Vec::new(),
                    },
                    correlations: CorrelationMatrix {
                        num_active_products: 0,
                        possible_correlations: Vec::new(),
                    },
                    mappings: Vec::new(),
                },
                state,
            )
            .unwrap();

        self.emit_event(state, Event::CovarianceMatrixInitialized { mpg_id: mpg.id });

        Ok(())
    }
}
