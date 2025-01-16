PROJECT_ROOT := $(shell git rev-parse --show-toplevel)
SOV_CLI_REL_PATH := $(PROJECT_ROOT)/target/debug/starter-cli-wallet
SPICENET_NODE_REL_PATH := $(PROJECT_ROOT)/target/debug/node

CELESTIA_CONFIG := $(PROJECT_ROOT)/celestia_rollup_config.toml

CREDENTIALS_DIR := $(PROJECT_ROOT)/credentials

ZKVM := risc0

# at height 3 the credits will already belong to the keys
START_HEIGHT := 4000000
TRUSTED_HASH := 0C2B8C2F5F3E38A61808E9BDF2B36BE41B859D7A27381579FE024AB965772A81
KEY_NAME := bridge-0
RPC_PORT := 26658

get_token := celestia bridge auth admin --p2p.network mocha

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
    SED = sed -i
else ifeq ($(UNAME_S),Darwin)
    SED = sed -i ''
else
    $(error Unsupported operating system: $(UNAME_S))
endif

initialize-celestia : 
	@echo "Initialize celestia network"
	cd ~/celestia-node
	if [ ! -d ~/.celestia-bridge-mocha-4 ]; then \
		celestia bridge init --core.ip rpc-mocha.pops.one:26657 --p2p.network mocha; \
		$(SED) 's/TrustedHash = ""/TrustedHash = "$(TRUSTED_HASH)"/g' ~/.celestia-bridge-mocha-4/config.toml; \
	fi

import-celestia-key: initialize-celestia
	@echo "Import celestia key"
	cd ~/celestia-node && ./cel-key import $(KEY_NAME) $(CREDENTIALS_DIR)/$(KEY_NAME).key --p2p.network mocha --node.type bridge


# start the celestia network and generate a new config
start-celestia : import-celestia-key
	celestia bridge start --core.ip rpc-mocha.pops.one:26657 --p2p.network mocha --keyring.keyname bridge-0

start-celestia-docker:
	@echo "Starting celestia with docker..."
	mkdir -p ./celestia-data-store
	docker run -e NODE_TYPE=bridge -e P2P_NETWORK=mocha --mount type=bind,source=$(PROJECT_ROOT)/celestia-data-store,target=/home/celestia --name celestia-bridge -d -p 26658:26658 ghcr.io/celestiaorg/celestia-node:v0.20.4-mocha celestia bridge start --core.ip rpc-mocha.pops.one:26657 --p2p.network mocha --keyring.keyname bridge-0 --node.store /home/celestia

celestia-bridge-auth:
	@echo "Get celestia bridge auth token"
	@TOKEN=$$(celestia bridge auth admin --p2p.network mocha); \
	$(SED) 's/^\(celestia_rpc_auth_token = \)"[^"]*"/\1"'$$TOKEN'"/' $(CELESTIA_CONFIG); \

clean-db:
	rm -rf ../../rollup-starter-data
	rm -rf mock_da.sqlite
	rm -rf demo_data

build-sov-cli:
	cargo build --bin starter-cli-wallet

build-node:
	cd crates/rollup; \
	cargo build --bin node

run-node: celestia-bridge-auth build-node
	$(SPICENET_NODE_REL_PATH) --da-layer celestia --rollup-config-path ./celestia_rollup_config.toml --genesis-config-dir ./test-data/genesis/celestia

run-node-mock: build-node
	$(SPICENET_NODE_REL_PATH) --rollup-config-path ./mock_rollup_config.toml --genesis-config-dir ./test-data/genesis/mock

test-create-token: build-sov-cli
	$(SOV_CLI_REL_PATH) transactions clean
	$(SOV_CLI_REL_PATH) node set-url http://127.0.0.1:12346
	$(SOV_CLI_REL_PATH) keys import --skip-if-present --nickname DANGER__DO_NOT_USE_WITH_REAL_MONEY --path ../../test-data/keys/token_deployer_private_key.json
	$(SOV_CLI_REL_PATH) transactions import from-file bank --chain-id 4321 --max-fee 100000000 --path ../../test-data/requests/transfer.json
	@echo "Submitting a batch"
	$(SOV_CLI_REL_PATH) transactions list
	$(SOV_CLI_REL_PATH) node submit-batch --wait-for-processing by-nickname DANGER__DO_NOT_USE_WITH_REAL_MONEY
	sleep 5


test-bank-supply-of:
	curl -sS -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"supply_of","params":{"token_id":"token_1nyl0e0yweragfsatygt24zmd8jrr2vqtvdfptzjhxkguz2xxx3vs0y07u7"},"id":1}' http://127.0.0.1:12345

