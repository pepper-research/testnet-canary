# Covariance Matrix Module

This module implements the covariance matrix used to calculate the variance of a cross-margined portfolio

## Overview

The covariance matrix is a critical component as it enables the determination of the maintenance margin for a portfolio. A higher variance in the portfolio correlates to a higher overall maintenance margin requirement.

## Key Components

### CovarianceMatrix Struct

Represents the covariance matrix for a set of products.

```rust
pub struct CovarianceMatrix<'a> {
    pub covariance_metadata: &'a CovarianceMetadata,

    pub correlations: &'a CorrelationMatrix,

    mappings: [u16; MAX_PRODUCTS], // Array of product index mappings
}
```

#### Methods
- `new(metadata, correlations, mappings)`: Constructs a new `CovarianceMatrix`
- `get_covariance(pubkey_1, pubkey_2)`: Retrieves covariance between two products
- `get_covariance_from_product_indexes(product_index_1, product_index_2)`: Retrieves covariance using product indexes

### MutableCovarianceMatrix Struct

A mutable version of the covariance matrix, allowing for updates to the underlying data.

```rust
pub struct MutableCovarianceMatrix<'a> {
    pub covariance_metadata: &'a mut CovarianceMetadata,
    pub correlations: &'a mut CorrelationMatrix,
}
```

#### Methods
- `to_covariance_matrix(mappings)`: Converts to an immutable `CovarianceMatrix`
- `set_covariance(product_keys, std, correlations)`: Updates the covariance matrix with new data

## Constants

- `MAX_PRODUCTS`: Maximum number of products (256)

## Usage

The `CovarianceMatrix` is used to retrieve covariance values between products, while `MutableCovarianceMatrix` allows for updating the covariance data. These structures work together with `CovarianceMetadata` and `CorrelationMatrix` to provide a complete representation of the covariance relationships between products in the system.