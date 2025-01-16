# Correlation Matrix
## Overview
This module defines a [linear upper triangular matrix](https://en.wikipedia.org/wiki/Triangular_matrix) to store the indices to another one dimensional vector which stores the correlations between two products.


```text
// product x0  x1  x2  x3  x4
//   y0     0   1   2   3   4
//   y1         5   6   7   8
//   y2             9  10  11
//   y3                12  13
//   y4                    14
```
(fig: correlation matrix)

The `mod.rs` file implements the Correlation Matrix struct as such:
```rust
pub struct CorrelationMatrix {
    /// State verifier for variance cache.
    /// [`RiskStateTag::VarianceCache]
    pub state_identifier: RiskStateTag,

    /// The number of active products(not settled/expired) products
    pub num_active_products: usize,

    /// The maximum number of possible correlations
    pub possible_correlations: [i8; MAX_CORRELATIONS], // correlation can be negative
}
```

The correlation index of say product x=3 and y =2 would be 10 inside the correlation_index_lookup_table.rs. We would then find the correlation in the possible_correlation with the index 10. We would then further calculate the associated correlation tick and then the int representation. 

## Notes
- Correlation values inside `correlation_lookup_table.rs` are stored as ticks, where one tick represents 1/128 of the full correlation range (-1 to 1) inside `possible_correlations`.
- The `MAX_CORRELATIONS` is set to a default of 8256 product pairs which is also equal to 128 products. The value of 8256 is derived from mathematical equation for Summation of a triangular series, that is $\frac{n(n+1)}{2}$ .
- The `CORRELATION_INDEX_LOOKUP_TABLE` in `correlation_index_lookup_table.rs` is used for efficient lookup of array indices in the correlation matrix.
- The `CORRELATION_LOOKUP_TABLE` in `correlation_lookup_table.rs` is used for converting between correlation ticks and FastInt representations.

