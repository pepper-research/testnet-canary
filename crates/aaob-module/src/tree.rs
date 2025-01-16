#![allow(dead_code)]
//! implementation of a crit-bit tree using sokoban.

use borsh::{BorshDeserialize, BorshSerialize};
use sokoban::critbit::CritbitNode;
use sokoban::{Critbit, NodeAllocatorMap};
use spicenet_shared::Side;
use std::fmt::Debug;
use {
    crate::{address::MarketId, StateType},
    bytemuck::{Pod, Zeroable},
};

// Summary of data of a given slab (tree)
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct SlabHeader {
    account_type: StateType,  // s=1
    root_node: u32,           // handle to the root node aka it's id; s=4
    total_orders: u64,        // Since each leaf is an order, its like total orders; s=8
    market_address: MarketId, // @TODO: verify the MarketId length with sov team (s=4?)
}

#[repr(C)]
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Order {
    pub order_id: u64, // can use custom id (price, timestamp, trg_id)
    pub price: u64,
    pub base_qty: u64,
    pub trg_id: u64,
}

impl Order {
    pub fn set_base_qty(&mut self, base_qty: u64) {
        self.base_qty = base_qty;
    }
}

unsafe impl Pod for Order {}
unsafe impl Zeroable for Order {}

pub const SLAB_HEADER_LEN: usize = 17;
pub const PADDED_SLAB_HEADER_LEN: usize = SLAB_HEADER_LEN + 7;

pub const MAX_SIZE: usize = 1000;
pub const NUM_NODES: usize = MAX_SIZE << 1;

/// * A slab is a data structure for a specific side of the market containing a slab header and array of nodes of a critbit tree
/// * whose leaves contain the data for an order of the orderbook. The side of the orderbook for which a slab holds data(orders) for can be identified
/// * using [`StateType`]
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(Clone, BorshDeserialize, BorshSerialize, Debug, Eq, PartialEq, Hash)]
pub struct Slab {
    header: SlabHeader,
    pub tree: Critbit<Order, NUM_NODES, MAX_SIZE>,
}

///
/// [[bids_header][asks_header][order_1]...[n]]

/// * data access methods
impl Slab {
    pub(crate) fn check(&self, side: Side) -> bool {
        match side {
            Side::Bid => self.header.account_type == StateType::Bids,
            Side::Ask => self.header.account_type == StateType::Asks,
        }
    }

    pub(crate) fn write_header(
        mut self,
        orders: Option<u64>,
        market_address: MarketId,
        account_type: StateType,
    ) {
        self.header = SlabHeader {
            root_node: self.tree.root,
            total_orders: orders.unwrap_or(0),
            market_address,
            account_type,
        }
    }

    // pub fn initialize(
    //     mut bids_state: [u8; 32],
    //     mut asks_state: [u8; 32],
    //     market_address: MarketId,
    // ) {
    //     let mut header = SlabHeader {
    //         account_type: StateType::Asks,
    //         root_node: 0,
    //         total_orders: 0,
    //         market_address,
    //     };
    //
    //     // @TODO: clear out the slab header init function
    //     header
    //         .serialize(&mut ((&mut asks_state.borrow_mut()) as &mut [u8]))
    //         .unwrap();
    //
    //     header.account_type = StateType::Bids;
    //
    //     header
    //         .serialize(&mut ((&mut bids_state.borrow_mut()) as &mut [u8]))
    //         .unwrap();
    // }
}

/// * node manipulation methods
impl Slab {
    #[inline(always)]
    pub fn capacity(&self) -> u64 {
        self.tree.len() as u64
    }

    #[inline(always)]
    pub fn get_node(&self, key: u32) -> Option<CritbitNode> {
        Some(self.tree.get_node(key))
    }

    #[inline(always)]
    pub(crate) fn get_node_mut(&mut self, key: u32) -> Option<&mut CritbitNode> {
        Some(self.tree.get_node_mut(key))
    }

    /// remove specific Order with u128 key
    #[inline(always)]
    fn remove(&mut self, key: u128) -> Option<Order> {
        self.tree.remove(&key)
    }

    /// get an Order by its u128 key
    #[inline(always)]
    fn get(&self, key: u128) -> Option<&Order> {
        self.tree.get(&key)
    }

    /// inserts a new order in the Slab with its key and Order object
    #[inline(always)]
    fn insert(&mut self, key: u128, value: Order) -> Option<u32> {
        self.tree.insert(key, value)
    }
}

impl Slab {
    #[inline(always)]
    pub fn root(&self) -> Option<u32> {
        Some(self.tree.root)
    }

    /// find the min/max key from the Slab
    fn find_min_max(&self, find_max: bool) -> Option<u32> {
        // Starting from top of the tree
        let mut root = self.root()?;

        loop {
            let is_inner_node = self.tree.is_inner_node(root);

            if is_inner_node {
                root = if find_max {
                    self.tree.get_right(root)
                } else {
                    self.tree.get_left(root)
                };
                continue;
            } else {
                return Some(root);
            }
        }
    }

    #[inline(always)]
    pub fn find_min(&self) -> Option<u32> {
        self.find_min_max(false)
    }

    #[inline(always)]
    pub fn find_max(&self) -> Option<u32> {
        self.find_min_max(true)
    }

    #[inline(always)]
    pub fn remove_by_key(&mut self, search_key: u128) -> Option<Order> {
        self.tree.remove(&search_key)
    }

    #[inline(always)]
    pub(crate) fn remove_min(&mut self) -> Option<Order> {
        let key = self.get_node(self.find_min().unwrap()).unwrap().key; // Finding the minimal (smallest order node)

        self.remove_by_key(key)
    }

    #[inline(always)]
    pub(crate) fn remove_max(&mut self) -> Option<Order> {
        let key = self.get_node(self.find_max().unwrap()).unwrap().key; // Finding the maximum (largest order node)

        self.remove_by_key(key)
    }

    #[inline(always)]
    pub fn find_by_key(&mut self, search_key: u128) -> Option<Order> {
        Some(self.tree.get(&search_key).unwrap().to_owned())
    }

    // #[cfg(test)]
    // fn check_invariants(&mut self) {
    //     let mut c = 0;
    //
    //     fn check_rec(
    //         slab: &mut Slab,
    //         key: NodeHandle,
    //         last_prefix_len: u64,
    //         last_prefix: u128,
    //         last_crit_bit: bool,
    //         count: &mut u64,
    //     ) {
    //         *count += 1;
    //
    //         let node = slab.get_node(key).unwrap();
    //
    //         assert!(node.prefix_len().unwrap() > last_prefix_len);
    //         let node_key = node.key().unwrap();
    //
    //         assert_eq!(
    //             last_crit_bit,
    //             (node_key & ((1u128 << 127) >> last_prefix_len)) != 0
    //         );
    //
    //         let prefix_mask = (((((1u128) << 127) as i128) >> last_prefix_len) as u128) << 1;
    //
    //         assert_eq!(last_prefix & prefix_mask, node.key().unwrap() & prefix_mask);
    //
    //         if let Some(x) = node.children() {
    //             check_rec(
    //                 slab,
    //                 x[0],
    //                 node.prefix_len().unwrap(),
    //                 node_key,
    //                 false,
    //                 count,
    //             );
    //             check_rec(
    //                 slab,
    //                 x[1],
    //                 node.prefix_len().unwrap(),
    //                 node_key,
    //                 true,
    //                 count,
    //             );
    //         }
    //     }
    //     if let Some(root) = self.root() {
    //         c += 1;
    //
    //         let node = self.get_node(root).unwrap();
    //
    //         let node_key = node.key().unwrap();
    //
    //         if let Some(x) = node.children() {
    //             check_rec(
    //                 self,
    //                 x[0],
    //                 node.prefix_len().unwrap(),
    //                 node_key,
    //                 false,
    //                 &mut c,
    //             );
    //             check_rec(
    //                 self,
    //                 x[1],
    //                 node.prefix_len().unwrap(),
    //                 node_key,
    //                 true,
    //                 &mut c,
    //             );
    //         }
    //     }
    //
    //     assert_eq!(
    //         c + self.header.free_list_len as u64,
    //         identity(self.header.bump_index)
    //     );
    //
    //     let mut free_nodes_remaining = self.header.free_list_len;
    //
    //     let mut next_free_node = self.header.free_list_head;
    //
    //     loop {
    //         let contents;
    //         match free_nodes_remaining {
    //             0 => break,
    //             1 => {
    //                 contents = self.get_node(next_free_node).unwrap();
    //
    //                 assert!(matches!(contents, NodeRef::LastFree(_)))
    //             }
    //             _ => {
    //                 contents = self.get_node(next_free_node).unwrap();
    //                 assert!(matches!(contents, NodeRef::Free(_)))
    //             }
    //         };
    //         let free_node = match contents {
    //             NodeRef::Free(x) | NodeRef::LastFree(x) => x.clone(),
    //             _ => unreachable!(),
    //         };
    //
    //         next_free_node = free_node.next;
    //         free_nodes_remaining -= 1;
    //     }
    // }
}
