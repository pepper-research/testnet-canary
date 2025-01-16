use anyhow::{Error, Result};
use sov_modules_api::{Context, EventEmitter, Spec, TxState};

use spicenet_shared::dex::MarketProductGroup;
use spicenet_shared::{FastInt, ProductId};

use crate::event::Event;
use crate::state::MutableCovarianceMatrix;
use crate::RiskModule;
use spicenet_shared::risk::RiskError;

impl<S: Spec> RiskModule<S> {
    pub(crate) fn update_covariance_matrix(
        &self,
        mpg: &MarketProductGroup<S>,
        product_keys: Vec<ProductId>,
        standard_deviations: Vec<FastInt>,
        correlations: Vec<Vec<FastInt>>,
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

        let mut covariance_matrix = self
            .covariance_matrix
            .get(&mpg.id, state)
            .unwrap()
            .ok_or::<Error>(RiskError::CovarianceMatrixNotInitialized.into())?;

        covariance_matrix.covariance_metadata.update_slot =
            self.time_module.get_slot(state).unwrap().slot;

        let (updated_covariance_metadata, updated_correlation_matrix) = MutableCovarianceMatrix {
            covariance_metadata: covariance_matrix.covariance_metadata,
            correlations: covariance_matrix.correlations,
        }
        .set_covariance(&product_keys, &standard_deviations, &correlations)?;

        covariance_matrix.covariance_metadata = updated_covariance_metadata;
        covariance_matrix.correlations = updated_correlation_matrix;

        self.covariance_matrix
            .set(&mpg.id, &covariance_matrix, state)
            .unwrap();

        self.emit_event(
            state,
            Event::CovarianceMatrixUpdated {
                mpg_id: mpg.id,
                product_keys,
                correlations,
                standard_deviations,
            },
        );

        Ok(())
    }
}
