## The repo structure:

- `crates/stf`: The `STF` is derived from the `Runtime` and is used in the `rollup` and `provers` crates.
- `crates/provers`: This crate is responsible for creating proofs for the `STF`.
- `crates/rollup`: This crate runs the `STF` and offers additional full-node functionalities.

(!) Note for using WIP repo.
This repo utilizes private [Sovereign SDK repo](https://github.com/Sovereign-Labs/sovereign-sdk-wip) and default cargo needs this environment variable to use an SSH key:

```
export CARGO_NET_GIT_FETCH_WITH_CLI=true
```

# Running the sov-rollup-starter

## How to run the sov-rollup-starter with mock-da

1. Change the working directory:

```shell,test-ci
$ cd crates/rollup/
```

2. If you want to run a fresh rollup, clean the database:

```sh,test-ci
$ make clean-db
```

3. Start the rollup node:

This will compile and start the rollup node:

```shell,test-ci,bashtestmd:long-running,bashtestmd:wait-until=RPC
$ cargo run --bin node
```

4. Submit a token creation transaction to the `bank` module:

```sh,test-ci
$ make test-create-token
```

5. Note the transaction hash from the output of the above command
   ```text
   Submitting tx: 0: 0xb244716ba0dad484e955c5e682814d88d9e2f58d743845c6a1585b49c73ede96
   Transaction 0xb244716ba0dad484e955c5e682814d88d9e2f58d743845c6a1585b49c73ede96 has been submitted: AcceptTxResponse { data: TxInfo { id: TxHash("0xa02ed59b5c698d49ad088584b86aff2134fd8e96746c1fce57b2518eb7c843e2"), status: Submitted }, meta: {} }
   Triggering batch publishing
   Your batch was submitted to the sequencer for publication. Response: SubmittedBatchInfo { da_height: 2, num_txs: 1 }
   Going to wait for target slot number 2 to be processed, up to 300s
   Rollup has processed target DA height=2!
   ```
6. To get the token address, fetch the events of the transaction hash from #5

```bash,test-ci
$ curl -sS http://127.0.0.1:12346/ledger/txs/0xb244716ba0dad484e955c5e682814d88d9e2f58d743845c6a1585b49c73ede96 | jq
{
  "data": {
    "type": "tx",
    "number": 0,
    "hash": "0xb244716ba0dad484e955c5e682814d88d9e2f58d743845c6a1585b49c73ede96",
    "event_range": {
      "start": 0,
      "end": 1
    },
    "body": "",
    "receipt": {
      "result": "successful",
      "data": {
        "gas_used": [
          3296,
          3296
        ]
      }
    },
    "events": [],
    "batch_number": 0
  },
  "meta": {}
}
$ curl -sS http://127.0.0.1:12346/ledger/txs/0xb244716ba0dad484e955c5e682814d88d9e2f58d743845c6a1585b49c73ede96/events | jq
{
  "data": [
    {
      "type": "event",
      "number": 0,
      "key": "Bank/TokenCreated",
      "value": {
        "TokenCreated": {
          "token_name": "sov-test-token",
          "coins": {
            "amount": 1000000,
            "token_id": "token_126x5str6mkes6ve8j92cnz579azyqlmrk74l6a4fg4zvd076hdxspqs3pc"
          },
          "minter": {
            "User": "sov15vspj48hpttzyvxu8kzq5klhvaczcpyxn6z6k0hwpwtzs4a6wkvqwr57gc"
          },
          "authorized_minters": [
            {
              "User": "sov1l6n2cku82yfqld30lanm2nfw43n2auc8clw7r5u5m6s7p8jrm4zqrr8r94"
            },
            {
              "User": "sov15vspj48hpttzyvxu8kzq5klhvaczcpyxn6z6k0hwpwtzs4a6wkvqwr57gc"
            }
          ]
        }
      },
      "module": {
        "type": "moduleRef",
        "name": "bank"
      }
    }
  ],
  "meta": {}
}
```

7. Get a total supply of the token:

```bash,test-ci,bashtestmd:compare-output
$ curl -Ss http://127.0.0.1:12346/modules/bank/tokens/token_126x5str6mkes6ve8j92cnz579azyqlmrk74l6a4fg4zvd076hdxspqs3pc/total-supply | jq -c -M
{"data":{"amount":1000000,"token_id":"token_126x5str6mkes6ve8j92cnz579azyqlmrk74l6a4fg4zvd076hdxspqs3pc"},"meta":{}}
```

## How to run the sov-rollup-starter using Celestia Da

1. Change the working directory:
   ```bash
   $ cd crates/rollup/
   ```
2. If you want to run a fresh rollup, clean the database:
   ```bash
   $ make clean
   ```
3. Start the Celestia local docker service. (make sure you have docker daemon running).
   ```bash
   $ make start
   ```
4. Start the rollup node with the feature flag building with the celestia adapter. To build with the sp1 prover, you may replace `risc0` with `sp1`.
   This will compile and start the rollup node:
   ```bash
   $ cargo run --bin node --no-default-features --features celestia_da,risc0
   ```
5. Submit a token creation transaction to the `bank` module.
   To build with the sp1 prover, you may replace `risc0` with `sp1`.
   Using `CELESTIA=1` will enable the client to be built with Celestia support and submit the test token
   ```bash
   $ CELESTIA=1 ZKVM=risc0 make test-create-token
   ```
6. Note the transaction hash from the output of the above command
   ```text
   Submitting tx: 0: 0xb244716ba0dad484e955c5e682814d88d9e2f58d743845c6a1585b49c73ede96
   Transaction 0xb244716ba0dad484e955c5e682814d88d9e2f58d743845c6a1585b49c73ede96 has been submitted: AcceptTxResponse { data: TxInfo { id: TxHash("0xa02ed59b5c698d49ad088584b86aff2134fd8e96746c1fce57b2518eb7c843e2"), status: Submitted }, meta: {} }
   Triggering batch publishing
   Your batch was submitted to the sequencer for publication. Response: SubmittedBatchInfo { da_height: 2, num_txs: 1 }
   Going to wait for target slot number 2 to be processed, up to 300s
   Rollup has processed target DA height=2!
   ```
7. To get the token address, fetch the events of the transaction hash from #5

   ```bash
   $ curl -sS http://127.0.0.1:12346/ledger/txs/0xb244716ba0dad484e955c5e682814d88d9e2f58d743845c6a1585b49c73ede96
   # Output omitted, should be similar to what has been seen in mock-da section
   ```

8. Get a total supply of the token:

```bash,test-ci,bashtestmd:compare-output
$ curl -Ss http://127.0.0.1:12346/modules/bank/tokens/token_126x5str6mkes6ve8j92cnz579azyqlmrk74l6a4fg4zvd076hdxspqs3pc/total-supply | jq -c -M
{"data":{"amount":1000000,"token_id":"token_126x5str6mkes6ve8j92cnz579azyqlmrk74l6a4fg4zvd076hdxspqs3pc"},"meta":{}}
```
