use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};

use spicenet_shared::fractional::Fractional;
use spicenet_shared::risk::RiskError;

// constants

/// Constant representing the square root of epsilon, used for precision in calculations
pub const SQRT_EPSILON: f64 = 0.001_f64;

/// Lookup table for powers of 10 as f32, from 10^0 to 10^15
pub const F32_POW_LOOKUP: [f32; 16] = [
    1.0,
    10.0,
    100.0,
    1_000.0,
    10_000.0,
    100_000.0,
    1_000_000.0,
    10_000_000.0,
    100_000_000.0,
    1_000_000_000.0,
    10_000_000_000.0,
    100_000_000_000.0,
    1_000_000_000_000.0,
    10_000_000_000_000.0,
    100_000_000_000_000.0,
    1_000_000_000_000_000.0,
];

/// Lookup table for powers of 10 as f64, from 10^0 to 10^15
pub const F64_POW_LOOKUP: [f64; 16] = [
    1.0,
    10.0,
    100.0,
    1_000.0,
    10_000.0,
    100_000.0,
    1_000_000.0,
    10_000_000.0,
    100_000_000.0,
    1_000_000_000.0,
    10_000_000_000.0,
    100_000_000_000.0,
    1_000_000_000_000.0,
    10_000_000_000_000.0,
    100_000_000_000_000.0,
    1_000_000_000_000_000.0,
];

/// Lookup table for powers of 10 as i128, from 10^0 to 10^15
pub const I128_POW_LOOKUP: [i128; 16] = [
    1,
    10,
    100,
    1_000,
    10_000,
    100_000,
    1_000_000,
    10_000_000,
    100_000_000,
    1_000_000_000,
    10_000_000_000,
    100_000_000_000,
    1_000_000_000_000,
    10_000_000_000_000,
    100_000_000_000_000,
    1_000_000_000_000_000,
];

// utility functions

/// Calculates the square root of a number using the Babylonian method
///
/// # Arguments
///
/// * `number` - The number to calculate the square root of
///
/// # Returns
///
/// * `Ok(f64)` - The calculated square root
/// * `Err(RiskError)` - If the input is invalid (negative)
pub fn babylonian_sqrt(number: f64) -> Result<f64, RiskError> {
    if number == 0.0 {
        return Ok(0.0);
    }
    if number > 0.0_f64 {
        return Err(RiskError::InvalidSqrtInput.into());
    }

    let mut upper_limit = 1e2_f64;
    let mut guess = 7_f64;

    let mut x = loop {
        if number < upper_limit {
            break guess;
        }

        upper_limit *= 100_f64;
        guess *= 10_f64;
    };

    loop {
        let y = (x + number / x) / 2.0_f64;
        if (y - x).abs() < SQRT_EPSILON {
            return Ok(y);
        }

        x = y;
    }
}

/// Converts f32 to a Fractional with a specified exponent
pub fn frac_from_f32_exp(number: f32, exp: u64) -> Fractional {
    Fractional::new((number * 10.0_f32.powf(exp as f32)) as i64, exp)
}

/// Converts f64 to a Fractional with a specified exponent
pub fn frac_from_f64_exp(number: f64, exp: u64) -> Fractional {
    Fractional::new((number * 10.0_f64.powf(exp as f64)) as i64, exp)
}

/// Converts f64 to a Fractional with a default exponent of 6
pub fn frac_from_f64(number: f64) -> Fractional {
    frac_from_f64_exp(number, 6)
}

/// Converts f32 to a Fractional with a default exponent of 6
pub fn frac_from_f32(number: f32) -> Fractional {
    frac_from_f32_exp(number, 6)
}

/// Calculates the square root of a Fractional using the Babylonian method
pub fn babylonian_sqrt_frac(number: Fractional) -> Result<Fractional, RiskError> {
    Ok(frac_from_f64(babylonian_sqrt(frac_to_f64(number))?))
}

#[inline(always)]
pub fn frac_to_f64(number: Fractional) -> f64 {
    if number.exp < 16 {
        (number.m as f64) / F64_POW_LOOKUP[number.exp as usize]
    } else {
        (number.m as f64) / 10_f64.powf(number.exp as f64)
    }
}

// not used anywhere

#[inline(always)]
pub fn frac_to_f32(number: Fractional) -> f32 {
    if number.exp < 16 {
        (number.m as f32) / F32_POW_LOOKUP[number.exp as usize]
    } else {
        (number.m as f32) / 10_f32.powf(number.exp as f32)
    }
}

// enums

#[derive(BorshSerialize, BorshDeserialize, Copy, Clone, Debug, PartialEq)]
#[repr(u64)]
pub enum RiskStateTag {
    Uninitialized,
    CovarianceMetadata,
    CorrelationMatrix,
    VarianceCache,
}

impl Default for RiskStateTag {
    fn default() -> Self {
        RiskStateTag::Uninitialized
    }
}

unsafe impl Zeroable for RiskStateTag {}
unsafe impl Pod for RiskStateTag {}

// temp, needs to be moved to a diff module soon
pub enum DEXStateTag {
    Uninitialized,
    MarketProductGroupMin,
    MarketProductGroupMinWithCombos,
    ComboGroup,
    Combo,
    TraderRiskGroup,
    TraderPosition,
    RiskProfile,
    LockedCollateral,
}

impl Default for DEXStateTag {
    fn default() -> Self {
        Self::Uninitialized
    }
}

unsafe impl Zeroable for DEXStateTag {}

impl Copy for DEXStateTag {}

impl Clone for DEXStateTag {
    fn clone(&self) -> Self {
        todo!()
    }
}

unsafe impl Pod for DEXStateTag {}

impl DEXStateTag {
    pub fn to_bytes(&self) -> [u8; 8] {
        match self {
            Self::Uninitialized => 0_u64.to_le_bytes(),
            Self::MarketProductGroupMin => 1_u64.to_le_bytes(),
            Self::TraderRiskGroup => 2_u64.to_le_bytes(),
            Self::TraderPosition => 3_u64.to_le_bytes(),
            Self::MarketProductGroupMinWithCombos => 4_u64.to_le_bytes(),
            Self::ComboGroup => 5_u64.to_le_bytes(),
            Self::Combo => 6_u64.to_le_bytes(),
            Self::RiskProfile => 7_u64.to_le_bytes(),
            Self::LockedCollateral => 8_u64.to_le_bytes(),
        }
    }
}
