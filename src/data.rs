use ahash::AHashMap;
use ringbuf::ring_buffer::RbBase;
use std::{collections::BTreeMap, mem::MaybeUninit};

use binance::rest_model::{Balance, Order};

use crate::ws::trades::TradesEvent;

/// Stack-allocated thread-local ring buffer with static capacity.
pub(crate) type StaticLocalRb<T, const N: usize> = ringbuf::LocalRb<T, [MaybeUninit<T>; N]>;

#[derive(Default)]
pub(crate) struct AppData {
    pub(crate) prices: AHashMap<String, f32>,
    pub(crate) book: (String, BTreeMap<String, f64>, BTreeMap<String, f64>),
    pub(crate) trades: StaticLocalRb<TradesEvent, 1000>,
    pub(crate) balances: Vec<Balance>,
    pub(crate) orders: Vec<Order>,
    pub(crate) quote: String,
}

impl AppData {
    /// Not all data is ready yet
    pub(crate) fn is_loading(&self) -> bool {
        self.prices.is_empty()
            || self.book.1.is_empty()
            || self.trades.is_empty()
            || self.balances.is_empty()
            || self.orders.is_empty()
    }
}