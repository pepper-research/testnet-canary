# Setup

1. [Sidecar](#Sidecar)
2. [Aggregator](#Aggregator)
3. [Node Software](#Node Software)

**Note: The aggregator right now simply prints the median and doesn't send it to the on chain worker**

# Sidecar
1. Clone `https://github.com/skip-mev/slinky` repository locally.
2. `cd slinky` and then switch branch by `checkout terpay/side-car-compose`.
3. run `export SLINKY_CONFIG_UPDATEINTERVAL=5ms`
4. run `export USE_CORE_MARKETS=true`
5. run `make start-sidecar-dev`

The sidecar will now run on port `8080`

TODO: Fetch bid-ask spread of the orderbook alongside price.

# Aggregator
Inside spicenet repo, go to `crates/oracle/aggregator` and simply run `cargo run`

# Node Software
Go to `crates/oracle/node` and run `cargo run` to start the node software

You can run more than one by running the same command in a new terminal
