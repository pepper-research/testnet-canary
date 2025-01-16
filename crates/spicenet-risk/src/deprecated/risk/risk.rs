use {
    crate::{
        covariance_matrix::CovarianceMatrix,
        error::ExchangeError


        ,
    },
    spicenet_shared::{
        fast_int::{FastInt, ZERO_FAST_INT},
        fractional::Fractional,
    }
    , // TODO: crate::temp::trg_minimal::TraderRiskGroupMin
};

/// Calculates the total variance of a portfolio, considering a potential position modification.
///
/// This function computes the variance (a measure of risk) for a given set of product positions,
/// taking into account a possible change in one of the positions. It uses the covariance matrix
/// to determine the relationships between different products in the portfolio.
///
/// # Parameters
///
/// - `product_positions` - A vector of tuples representing product positions. Each tuple contains:
///   - Product index (usize)
///   - Current position (FastInt)
///   - Two additional FastInt values (purpose not specified in the current implementation)
/// - `covariance_matrix` - A reference to the `CovarianceMatrix` providing covariance values between products
/// - `idx` - The index of the product whose position might be modified
/// - `position_modification` - The potential change in position for the product at index `idx`
///
/// # Returns
/// - `Ok(FastInt)`: The calculated total variance as a `FastInt`
/// - `Err(ExchangeError)`: If an error occurs during the calculation
///
fn calculate_total_variance(
    product_positions: &Vec<(usize, FastInt, FastInt, FastInt)>,
    covariance_matrix: &CovarianceMatrix,
    idx: usize,
    position_modification: FastInt,
) -> Result<FastInt, ExchangeError> {
    let mut port_variance = ZERO_FAST_INT;

    for (idx_i, (position_idx_1, position_1, _, _)) in product_positions.iter().enumerate() {
        let pos_1 = match idx_i == idx {
            true => *position_1 + position_modification,
            false => *position_1,
        };

        for (mut idx_j, (position_idx_2, position_2, _, _)) in product_positions.iter().enumerate() {
            let covariance = covariance_matrix
                .get_covariance_from_product_indexes(*position_idx_1, *position_idx_2)?;

            let pos_2 = match idx_j == idx {
                true => *position_2 + position_modification,
                false => *position_2,
            };

            port_variance += pos_1 * covariance * pos_2;
        }
    }

    Ok(port_variance)
}
