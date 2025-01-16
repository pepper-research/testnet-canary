use std::ops::Sub;

use crate::event::Event;
use crate::state::{OracleNode, WhitelistedUser};
use crate::OracleRegistry;
use anyhow::{bail, Result};
use sov_bank::{Coins, IntoPayable, GAS_TOKEN_ID};
use sov_modules_api::{Address, CallResponse, Context, EventEmitter, ModuleInfo, Spec, TxState};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    schemars(bound = "S::Address: ::schemars::JsonSchema", rename = "CallMessage")
)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary, proptest_derive::Arbitrary)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub enum CallMessage<S: Spec> {
    Register {
        node_address: Address<S>,
        user_address: Address<S>,
        amount: u64,
    },
    Deposit {
        node_address: Address<S>,
        user_address: Address<S>,
        amount: u64,
    },
    Exit {
        node_address: Address<S>,
        user_address: Address<S>,
    },
    Whitelist {
        user_address: Address<S>,
    },
}

impl<S: Spec> OracleRegistry<S> {
    pub(crate) fn whitelist(
        &self,
        user_address: Address<S>,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<CallResponse> {
        let sender = context.sender();
        let registry_authority = match self.registry_authority.get(state)? {
            Some(authority) => authority,
            None => bail!("Registry authority not set"),
        };

        if sender.as_ref() != registry_authority.as_ref() {
            bail!("Sender is not the registry authority");
        }

        self.whitelisted_users.set(
            &user_address,
            &WhitelistedUser {
                address: user_address,
                whitelisted_ts: self.time.get_time(state)?.unix_timestamp,
                is_oracle_node: false,
            },
            state,
        )?;

        self.emit_event(state, Event::UserWhitelisted { user_address });

        Ok(CallResponse::default())
    }

    pub(crate) fn register(
        &self,
        node_address: Address<S>,
        user_address: Address<S>,
        amount: u64,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<CallResponse> {
        let sender = context.sender();

        if sender.as_ref() != user_address.as_ref() {
            bail!("Sender is not the user address");
        }

        let whitelisted_user = match self.whitelisted_users.get(&user_address, state)? {
            Some(user) => user,
            None => bail!("User not whitelisted"),
        };

        if whitelisted_user.is_oracle_node {
            bail!("Oracle node already registered");
        }

        let min_bond_amt = match self.minimum_bond_amt.get(state)? {
            Some(min_bond_amt) => min_bond_amt,
            None => bail!("Min bond amount not set"),
        };

        if amount < min_bond_amt {
            bail!("Bond amount is less than minimum bond amount");
        }

        self.bank
            .transfer_from(sender, self.id().to_payable(), gas_coins(amount), state)?;

        self.whitelisted_users.set(
            &user_address,
            &WhitelistedUser {
                address: whitelisted_user.address,
                whitelisted_ts: whitelisted_user.whitelisted_ts,
                is_oracle_node: true,
            },
            state,
        )?;

        self.oracle_nodes.set(
            &node_address,
            &OracleNode {
                address: node_address,
                accumulated_penalty: 0,
                amount_staked: amount,
            },
            state,
        )?;

        self.emit_event(
            state,
            Event::NodeRegistered {
                node_address,
                amount,
            },
        );

        Ok(CallResponse::default())
    }

    pub(crate) fn deposit(
        &self,
        node_address: Address<S>,
        user_address: Address<S>,
        amount: u64,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<CallResponse> {
        let sender = context.sender();

        if sender.as_ref() != user_address.as_ref() {
            bail!("Sender is not the user address");
        }

        let whitelisted_user = match self.whitelisted_users.get(&user_address, state)? {
            Some(user) => user,
            None => bail!("User not whitelisted"),
        };

        if !whitelisted_user.is_oracle_node {
            bail!("User is not an oracle node");
        }

        let oracle_node = match self.oracle_nodes.get(&node_address, state)? {
            Some(node) => node,
            None => bail!("Oracle node not registered"),
        };

        self.bank
            .transfer_from(sender, self.id().to_payable(), gas_coins(amount), state)?;

        self.oracle_nodes.set(
            &node_address,
            &OracleNode {
                address: oracle_node.address,
                accumulated_penalty: oracle_node.accumulated_penalty,
                amount_staked: oracle_node.amount_staked + amount,
            },
            state,
        )?;

        self.emit_event(
            state,
            Event::NodeDeposited {
                node_address,
                amount,
            },
        );

        Ok(CallResponse::default())
    }

    pub(crate) fn exit(
        &self,
        node_address: Address<S>,
        user_address: Address<S>,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<CallResponse> {
        let sender = context.sender();

        if sender.as_ref() != user_address.as_ref() {
            bail!("Sender is not the user address");
        }

        let whitelisted_user = match self.whitelisted_users.get(&user_address, state)? {
            Some(user) => user,
            None => bail!("User not whitelisted"),
        };

        if !whitelisted_user.is_oracle_node {
            bail!("User is not an oracle node");
        }

        let oracle_node = match self.oracle_nodes.get(&node_address, state)? {
            Some(node) => node,
            None => bail!("Oracle node not registered"),
        };

        self.bank.transfer_from(
            self.id().to_payable(),
            sender,
            gas_coins(
                oracle_node
                    .amount_staked
                    .sub(oracle_node.accumulated_penalty),
            ),
            state,
        )?;

        self.oracle_nodes.remove(&node_address, state)?;
        self.whitelisted_users.set(
            &user_address,
            &WhitelistedUser {
                address: whitelisted_user.address,
                whitelisted_ts: whitelisted_user.whitelisted_ts,
                is_oracle_node: false,
            },
            state,
        )?;

        self.emit_event(state, Event::NodeExited { node_address });

        Ok(CallResponse::default())
    }
}

fn gas_coins(amount: u64) -> Coins {
    Coins {
        amount,
        token_id: GAS_TOKEN_ID,
    }
}
