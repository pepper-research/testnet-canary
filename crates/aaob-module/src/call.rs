use crate::{
    address::MarketId, fp32_div, fp32_mul, get_market_id, orderbook::OrderId,
    orderbook::OrderbookId, AAOBError, Event, Market, Order, AAOB,
};
use anyhow::{bail, Result};
use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use sokoban::NodeAllocatorMap;
use sov_modules_api::{Context, EventEmitter, SafeString, Spec, TxState};
use spicenet_shared::Side;
use std::cmp::PartialEq;
// use crate::{AAOB, MarketId, OrderId, OrderbookId, Side, Slab, Node, LeafNode};
use crate::state::*;

/// This struct is written into the event queue's register at the time of new order, or cancel existing order.
///
/// Case 1: New order
/// the quantities describe the total order amounts which were either matched against other orders or written into the book.
///
/// Case 2: Cancel existing order
/// the quantities describe what was left of the order in the orderbook
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct OrderSummary {
    /// to be provided during order cancels
    pub order_id: Option<u64>,
    pub total_base_qty: u64,
    pub total_quote_qty: u64,
    pub total_base_qty_posted: u64,
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(rename = "CallMessage"),
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
pub enum CallMessage {
    CreateOrder {
        market_id: MarketId,
        side: Side,
        max_base_qty: u64,
        max_quote_qty: u64,
        limit_price: u64,
        post_only: bool,
        post_allowed: bool,
        self_trade_behavior: SelfTradeHandler,
        match_limit: u64,
        trg_id: u64,
    },
    CancelOrder {
        market_id: MarketId,
        order_id: OrderId,
        side: Side,
    },
    CreateMarket {
        market_name: SafeString,
        fee_budget: u64,
        min_base_size: u64,
        tick_size: u64,
    },
    CloseMarket {
        market_name: SafeString,
    },
}

// impl<S: Spec> EventEmitter for AAOB<S> {
//     type Spec = ();
//     type Event = Event;

//     fn emit_event(&self, state: &mut impl EventContainer, event_key: &str, event: Self::Event) {
//         self.emit_event(state, event_key, event);
//     }
// }

impl<S: Spec> AAOB<S> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_order(
        &self,
        market_id: MarketId,
        side: Side,
        max_base_qty: u64,
        max_quote_qty: u64,
        limit_price: u64,
        post_only: bool,
        post_allowed: bool,
        self_trade_behaviour: SelfTradeHandler,
        trg_id: u64,
        _context: &Context<S>,
        state: &mut impl TxState<S>,
        mut match_limit: u64,
        // TODO: new order params
    ) -> Result<()> {
        // Get the orderbook state
        let mut orderbook_state = self.orderbooks.get(&market_id, state).unwrap().unwrap();

        let mut base_qty_remaining = max_base_qty;
        let mut quote_qty_remaining = max_quote_qty;
        // create new bid
        let mut crossed = true;
        let market: Market<S> = self
            .markets
            .get(&orderbook_state.market_id, state)
            .unwrap()
            .unwrap();

        if limit_price % market.tick_size != 0 {
            bail!("create_order: invalid limit price {limit_price}");
        }

        let min_base_order_size = market.min_base_size;

        let mut order_id: OrderId = 0;

        // don't need the `new_leaf_order_id` anymore as it is automatically done by sokoban
        // let new_leaf_order_id = 10u128; // dummy for now, earlier event queue was deriving it which we aren't using anymore
        loop {
            if match_limit == 0 {
                break;
            }
            // Find the best bid/offer on the opposite side
            let bbo_h = match orderbook_state.find_bbo(side.opposite()) {
                None => {
                    crossed = false;
                    break;
                }
                Some(h) => h,
            };

            let opposite_book_tree = orderbook_state.get_tree(side.opposite());

            let bbo_h_node = &opposite_book_tree // Getting the opposite tree (since bids and asks are separate trees)
                .get_node(bbo_h) // Getting the best price node via its key found above in bbo_h var
                .unwrap()
                .key; // critbit node key

            let mut trade = opposite_book_tree.tree.get(bbo_h_node).unwrap().to_owned();

            crossed = match side {
                Side::Bid => limit_price >= trade.price,
                Side::Ask => limit_price <= trade.price,
            };

            if post_only || !crossed {
                break;
            }

            let offer_size = trade.base_qty;
            let base_trade_qty = offer_size
                .min(base_qty_remaining) // 5
                .min(fp32_div(quote_qty_remaining, trade.price));

            if base_trade_qty == 0 {
                // not able to fill up
                break;
            }

            let quote_maker_qty = fp32_mul(base_trade_qty, trade.price);

            if quote_maker_qty == 0 {
                break;
            }

            if self_trade_behaviour != SelfTradeHandler::DecrementTake {
                let will_order_self_trade =
                    trg_id == (opposite_book_tree.tree.get(bbo_h_node).unwrap().trg_id);

                if will_order_self_trade {
                    let best_offer_id = opposite_book_tree.tree.get(bbo_h_node).unwrap().order_id;

                    if self_trade_behaviour == SelfTradeHandler::AbortTx {
                        bail!(AAOBError::WouldSelfTrade);
                    }

                    assert_eq!(self_trade_behaviour, SelfTradeHandler::CancelProvide);

                    orderbook_state
                        .get_tree(side.opposite())
                        .remove_by_key(best_offer_id.into());

                    match_limit -= 1;
                    continue;
                }
            }

            trade.set_base_qty(trade.base_qty - base_trade_qty);
            base_qty_remaining -= base_trade_qty;
            quote_qty_remaining -= quote_maker_qty;

            if trade.base_qty < min_base_order_size {
                let best_offer_id = opposite_book_tree.tree.get(bbo_h_node).unwrap().order_id;

                let curr_side = side; // current side

                orderbook_state
                    .get_tree(curr_side)
                    .remove_by_key(best_offer_id.into())
                    .unwrap();
            } else {
                orderbook_state.get_tree(side.opposite()).tree.insert(
                    *bbo_h_node,
                    Order {
                        order_id: bbo_h as u64,
                        price: limit_price,
                        base_qty: base_qty_remaining,
                        trg_id,
                    },
                );
                // TODO: need good review from @karthik about price and base_qty
                // create a leaf `bbo_h_node` on `bbo_h` node
            }

            order_id = OrderId::from(bbo_h); // TODO: need good review from @karthik about ID

            match_limit -= 1;
        }

        let base_qty_to_post = std::cmp::min(
            fp32_div(quote_qty_remaining, limit_price),
            base_qty_remaining,
        );

        if crossed || !post_allowed || base_qty_to_post < min_base_order_size {
            let out_reason = if base_qty_to_post < min_base_order_size {
                CompletedReason::Filled
            } else if crossed {
                if match_limit == 0 {
                    CompletedReason::MatchLimitExhausted
                } else {
                    CompletedReason::PostOnly
                }
            } else {
                CompletedReason::PostNotAllowed
            };

            if let CompletedReason::PostOnly = out_reason {
                self.emit_event(
                    state,
                    Event::OrderCreated {
                        order_id,
                        market_id,
                        side,
                        total_base_qty: max_base_qty - base_qty_remaining,
                        total_quote_qty: max_quote_qty - quote_qty_remaining,
                        total_base_qty_posted: 0,
                    },
                );

                // let order_summary = OrderSummary {
                //     order_id: None,
                //     total_base_qty: max_base_qty - base_qty_remaining,
                //     total_quote_qty: max_quote_qty - quote_qty_remaining,
                //     total_base_qty_posted: 0,
                // };

                return Ok(());

                // return Ok(OrderSummary {
                //     order_id: None,
                //     total_base_qty: max_base_qty - base_qty_remaining,
                //     total_quote_qty: max_quote_qty - quote_qty_remaining,
                //     total_base_qty_posted: 0,
                // });
            }

            self.emit_event(
                state,
                Event::OrderCreated {
                    order_id,
                    market_id,
                    side,
                    total_base_qty: max_base_qty - base_qty_remaining,
                    total_quote_qty: max_quote_qty - quote_qty_remaining,
                    total_base_qty_posted: 0,
                },
            );

            // let order_summary = OrderSummary {
            //     order_id: Some(order_id as u64),
            //     total_base_qty: max_base_qty - base_qty_remaining,
            //     total_quote_qty: max_quote_qty - quote_qty_remaining,
            //     total_base_qty_posted: 0,
            // };

            return Ok(());

            // return Ok(OrderSummary {
            //     order_id: Some(order_id),
            //     total_base_qty: max_base_qty - base_qty_remaining,
            //     total_quote_qty: max_quote_qty - quote_qty_remaining,
            //     total_base_qty_posted: 0,
            // });
        }

        // @TODO - also figure out the KV Id of the node to store this Order, rn kept 0
        // Inserting the order as a leaf finally
        orderbook_state.get_tree(side).tree.insert(
            0,
            Order {
                order_id: order_id.try_into().unwrap(),
                price: limit_price,
                base_qty: base_qty_to_post,
                trg_id,
            },
        );

        base_qty_remaining -= base_qty_to_post;
        quote_qty_remaining -= fp32_mul(base_qty_to_post, limit_price);
        // Emit an event
        self.emit_event(
            state,
            Event::OrderCreated {
                order_id,
                market_id,
                side,
                total_base_qty: max_base_qty - base_qty_remaining,
                total_quote_qty: max_quote_qty - quote_qty_remaining,
                total_base_qty_posted: base_qty_to_post,
            },
        );

        Ok(())
    }

    pub(crate) fn cancel_order(
        &self,
        market_id: MarketId,
        order_id: OrderId,
        side: Side,
        _context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        // Get the orderbook state
        let mut orderbook_state = self.orderbooks.get(&market_id, state).unwrap().unwrap();

        let tree = orderbook_state.get_tree(side).tree;

        // Try to find the order in the tree using its id (key)
        let order_ref = tree.get_node(order_id as u32).key;

        if order_ref == 0 {
            bail!(AAOBError::OrderNotFound);
        }

        orderbook_state
            .get_tree(side)
            .remove_by_key(order_id)
            .unwrap();

        self.emit_event(
            state,
            Event::OrderCancelled {
                order_id,
                market_id,
                side,
            },
        );

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_market(
        &self,
        market_name: &str,
        fee_budget: u64,
        min_base_size: u64,
        tick_size: u64,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        let market = Market::new(
            market_name,
            fee_budget,
            min_base_size,
            tick_size,
            &self.markets, // Map<MarketId, Market<S>>
            context,
            state,
        );
        let market_id = get_market_id::<S>(market_name);
        self.markets.set(&market_id, &market.unwrap().1, state)?;
        self.market_ids.push(&market_id, state)?;
        self.emit_event(
            state,
            Event::MarketCreated {
                market_id,
                market_name: market_name.to_string(),
            },
        );
        Ok(())
    }

    pub(crate) fn close_market(
        &self,
        market_name: &str,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        let market_id = get_market_id::<S>(market_name);
        let market = &self.markets.get(&market_id, state)?;

        if !market.is_some() {
            bail!("market with name {} does nots exists", market_name);
        } else {
            self.emit_event(
                state,
                Event::MarketClosed {
                    market_id,
                    market_name: market_name.to_string(),
                },
            );

            self.markets.remove(&market_id, state)?;

            let results: Vec<Result<MarketId, _>> = self.market_ids.iter(state).unwrap().collect();
            let mut market_ids: Vec<MarketId> =
                results.into_iter().filter_map(Result::ok).collect();

            let initial_len: usize = market_ids.len();
            market_ids.retain(|id| id != market_id);

            if market_ids.len() < initial_len {
                self.market_ids.clear(state).expect("cant clear market ids");
                self.market_ids
                    .set_all(market_ids, state)
                    .expect("cant set all market ids");
            }
        }
        Ok(())
    }
}
