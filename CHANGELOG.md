## #244

1. Migrate orderbookId to type MarketId

## #299

1. Fix capsule rpc errors, `cargo check` passes now

## #294

1. Multiple fixes to capsule, `cargo check` passes now
2. Add support for sui, aptos and evm wallets to capsule

## #281

1. Use price indexes for LUT price data
2. Update db operations for price ticks
3. Update LUT to insert price ticks

## #267

1. Remove orpheus
2. Move orpheus features to lut
3. Add helpers for off chain cold storage of ticks
4. Add EMA to lut

## #232

1. Added cargo checks and tests as shell script to CI
2. Added changelog tests to CI
3. Added rustfmt check to CI

## #163 (partial)

1. Added benchmarks between fastint, u64, fixed and fractional

## #212

1. Fix errors in oracle registry module

## #172

1. Fixed LUT tests for updated sovereign packages

## #179

1. Add oracle registry module

## #166

1. Add Diesel ORM for database
2. Add `insert_log` helper in `spicenet-shared`

## #150

1. Done with Sokoban integration
2. Fixed call instructions of market and order
3. Fixed callback_info's (need reviews on some stuff)
4. Use bail!() for all errors from now on

## #148 (SPI-86/bolt)

1. Changed `oracle_address` from `Address` to `OracleId` defined in shared
2. Added `other_ids` in shared to define `OracleId`
3. Removed `dex` folder from risk module
4. Updated ID and struct imports to be used from the `shared` and `dex` module respectively

## #149

1. Update all RPC methods to use `market_ids` StateVec rather than `iter` method
2. Fix import/type issues
3. Make fields of OrderBookState public in order to access them

## #135

1. Add call instructions to `spicenet-risk`
2. Move shared types to `spicenet-shared`
3. Add helpers and utils for risk-related functions in `spicenet-risk`
4. Multiple bug fixes

## #132
1. Add stork integration
2. Fix conflicts from staging