use crate::{dex::constants::*, DexError, DexResult};

use schemars::JsonSchema;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
pub struct OpenOrdersMetadata {
    pub ask_qty_in_book: i64,
    pub bid_qty_in_book: i64,
    pub head_index: u16,
    pub num_open_orders: u16,
}

// Old stuff, unused
// #[cfg_attr(
//     feature = "native",
//     derive(serde::Serialize),
//     derive(serde::Deserialize)
// )]
// #[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
// pub struct OpenOrdersMetadataOld {
//     pub ask_qty_in_book: Fractional,
//     pub bid_qty_in_book: Fractional,
//     pub head_index: usize,
//     pub num_open_orders: u64,
// }

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
pub struct OpenOrders {
    pub free_list_head: u16,
    pub total_open_orders: u16,
    pub max_open_orders: u16,

    #[cfg_attr(
        feature = "native",
        serde(deserialize_with = "serde_arrays::deserialize"),
        serde(serialize_with = "serde_arrays::serialize"),
        schemars(with = "OpenOrdersMetadata", length(equal = "MAX_PRODUCTS"))
    )]
    pub products: [OpenOrdersMetadata; MAX_PRODUCTS],

    #[cfg_attr(
        feature = "native",
        serde(deserialize_with = "serde_arrays::deserialize"),
        serde(serialize_with = "serde_arrays::serialize"),
        schemars(with = "OpenOrdersNode", length(equal = "MAX_OPEN_ORDERS"))
    )]
    pub orders: [OpenOrdersNode; MAX_OPEN_ORDERS],
}

// Unused
// #[cfg_attr(
//     feature = "native",
//     derive(serde::Serialize),
//     derive(serde::Deserialize)
// )]
// #[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
// pub struct OpenOrdersOld {
//     pub free_list_head: usize,
//     pub total_open_orders: u64,
//     pub products: [OpenOrdersMetadataOld; MAX_PRODUCTS],
//     pub orders: [OpenOrdersNodeOld; 1024],
// }

impl OpenOrders {
    pub fn initialize(&mut self) {
        self.free_list_head = 1;
        self.max_open_orders = MAX_OPEN_ORDERS as u16;
        for product_meta in self.products.iter_mut() {
            product_meta.ask_qty_in_book = 0;
            product_meta.bid_qty_in_book = 0;
            product_meta.head_index = 0;
            product_meta.num_open_orders = 0;
        }
    }

    pub fn get_order_index(&self, product_index: usize, order_id: u128) -> DexResult<usize> {
        let mut i = self.products[product_index].head_index;
        while i != SENTINEL {
            let node = unsafe { self.orders.get_unchecked(i as usize) };
            if node.id == order_id {
                return Ok(i as usize);
            }
            i = node.next;
        }
        Err(DexError::OrderNotFound.into())
    }

    pub fn get_order_index_and_id_by_client_order_id(
        &self,
        product_index: usize,
        client_order_id: u64,
    ) -> DexResult<(usize, u128)> {
        let mut i = self.products[product_index].head_index;
        while i != SENTINEL {
            let node = unsafe { self.orders.get_unchecked(i as usize) };
            if node.client_id == client_order_id {
                return Ok((i as usize, node.id));
            }
            i = node.next;
        }
        Err(DexError::OrderNotFound.into())
    }

    pub fn has_open_order(&self, product_index: usize, order_id: u128) -> bool {
        return match self.get_order_index(product_index, order_id) {
            Ok(_) => true,
            _ => false,
        };
    }

    pub fn decrement_order_size_by_index(&mut self, i: usize, qty: u64) -> DexResult {
        // assert(i < self.max_open_orders as usize, DexError::InvalidOrderID)?;
        unsafe {
            self.orders.get_unchecked_mut(i).qty -= qty;
        }
        Ok(())
    }

    pub fn decrement_order_size(&mut self, index: usize, order_id: u128, qty: u64) -> DexResult {
        let order_index = self.get_order_index(index, order_id)?;
        unsafe {
            self.orders.get_unchecked_mut(order_index).qty -= qty;
        }
        Ok(())
    }

    #[inline(always)]
    pub fn get_next_index(&self) -> usize {
        self.free_list_head as usize
    }

    pub fn add_open_order(
        &mut self,
        index: usize,
        order_id: u128,
        qty: u64,
        client_id: u64,
    ) -> DexResult {
        let head_index = &mut self.products[index].head_index;
        let i = *head_index;
        // Fetch the index of the free node to write to
        let free_list_head = self.free_list_head;
        let free_node = unsafe { self.orders.get_unchecked_mut(free_list_head as usize) };
        let next_free_node = free_node.next;
        // Add the order id to free node
        free_node.id = order_id;
        free_node.client_id = client_id;
        free_node.qty = qty;
        free_node.next = i;
        free_node.prev = SENTINEL;
        // Assign this node as the new head for the index
        *head_index = free_list_head;
        if i != SENTINEL {
            // If there are existing open orders for this index, we set the current head
            // to point to the updated head
            unsafe {
                self.orders.get_unchecked_mut(i as usize).prev = free_list_head;
            }
        }
        if next_free_node == SENTINEL {
            // If there are no more free nodes, this means that the linked list is densely packed.
            // The next free node will just be the next index.
            // assert(
            //     free_list_head + 1 < self.max_open_orders,
            //     DexError::TooManyOpenOrdersError,
            // )?;
            self.free_list_head = free_list_head + 1;
        } else {
            // If there are free nodes remaining, we keep traversing the linked list.
            self.free_list_head = next_free_node;
        }
        Ok(())
    }

    fn remove_node(&mut self, index: usize, i: usize) {
        let head_index = &mut self.products[index].head_index;
        let free_list_head = self.free_list_head;
        let node = unsafe { self.orders.get_unchecked_mut(i) };
        let next = node.next;
        let prev = node.prev;
        if prev == SENTINEL {
            // If we enter this block, we need to update the head of the index as we are deleting the current head.
            *head_index = next;
        }
        // In the process of deleting the current node, we add it to the head of the free list.
        node.id = 0;
        node.qty = 0;
        node.next = free_list_head;
        node.prev = SENTINEL;
        unsafe {
            self.orders.get_unchecked_mut(free_list_head as usize).prev = i as u16;
        }
        self.free_list_head = i as u16;
        // If the node is not the head or tail, we need to modify the pointers of the prev and next nodes.
        if next != SENTINEL {
            unsafe {
                self.orders.get_unchecked_mut(next as usize).prev = prev;
            }
        }
        if prev != SENTINEL {
            unsafe {
                self.orders.get_unchecked_mut(prev as usize).next = next;
            }
        }
    }

    pub fn remove_open_order_by_index(
        &mut self,
        index: usize,
        i: usize,
        order_id: u128,
    ) -> DexResult {
        // assert(
        //     i < self.max_open_orders as usize
        //         && unsafe { self.orders.get_unchecked(i).id == order_id },
        //     DexError::InvalidOrderID,
        // )?;
        self.remove_node(index, i);
        Ok(())
    }

    pub fn remove_open_order(&mut self, index: usize, order_id: u128) -> DexResult {
        let i = self.get_order_index(index, order_id)?;
        self.remove_node(index, i);
        Ok(())
    }

    pub fn clear(&mut self, index: usize) -> DexResult {
        let head_index = &mut self.products[index].head_index;
        let mut i = *head_index;
        while i != SENTINEL {
            let node = unsafe { self.orders.get_unchecked_mut(i as usize) };
            let next = node.next;
            self.remove_node(index, i as usize);
            i = next;
        }
        Ok(())
    }
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
pub struct OpenOrdersNode {
    pub id: u128,
    pub qty: u64,
    pub client_id: u64,
    pub prev: u16,
    pub next: u16,
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct OpenOrdersNodeOld {
    pub id: u128,
    pub qty: u64,
    pub client_id: u64,
    pub prev: usize,
    pub next: usize,
}
