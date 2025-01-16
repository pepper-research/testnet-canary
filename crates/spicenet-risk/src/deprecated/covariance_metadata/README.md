# Covariance Metadata Module

This module defines the `CovarianceMetadata` struct, which provides key datapoints used to construct the covariance matrix.

## Overview

The covariance metadata is a crucial component that stores essential information for computing the covariance matrix. It includes data such as standard deviations of products, which are used in the calculation of covariance values.

## Key Components

### CovarianceMetadata Struct

Stores metadata required for constructing and using the covariance matrix.

```rust
pub struct CovarianceMetadata {
    /// State verifier for variance cache.
    /// [`RiskStateTag::VarianceCache]
    pub state_identifer: RiskStateTag,

    /// The last time that the variance cache was updated
    pub update_offset: u64,

    /// The number of active products(not settled/expired) products
    pub num_active_products: usize,

    /// Array of product keys of the type [`ProductId`]
    pub product_keys: [ProductId; MAX_OUTRIGHTS],

    /// Array of standard deviations of the products, of the type [`FastInt`]
    pub standard_deviations: [FastInt; MAX_OUTRIGHTS],
}
```

#### Methods
- `get_product_index(product_key)`: Retrieves the index of a product in the metadata
- `get_std(product_key)`: Retrieves the standard deviation of a product

## Constants

- `MAX_OUTRIGHTS`: Maximum number of outright products (128)

## Usage

The `CovarianceMetadata` struct is used in conjunction with the `CovarianceMatrix` to provide a complete representation of the covariance relationships between products. It stores essential data like product keys and their standard deviations, which are used in covariance calculations.

## Notes

- The `CovarianceMetadata` struct has a fixed size, determined by `CovarianceMetadata::LEN`.
- It uses `FastInt` for efficient integer arithmetic in standard deviation calculations.
- The module relies on the `ProductId` type from the temporary products module, which may be subject to change in future implementations.