use ahash::AHashMap;
use ringbuf::ring_buffer::RbBase;
use std::{
    collections::BTreeMap,
    mem::MaybeUninit,
    time::{Duration, Instant},
};
use tracing::debug;

use binance::rest_model::{Balance, Order};

use crate::ws::trades::TradesEvent;

/// Stack-allocated thread-local ring buffer with static capacity.
pub(crate) type StaticLocalRb<T, const N: usize> = ringbuf::LocalRb<T, [MaybeUninit<T>; N]>;

/// Filter strategy
#[derive(Debug)]
pub(crate) enum PriceFilter {
    /// Matches exactly one of values
    Matches(Vec<String>),

    /// Includes value
    Contains(String),

    /// No filtering
    All,
}

impl PriceFilter {
    fn apply(&self, value: &str) -> bool {
        match self {
            PriceFilter::Matches(filters) => {
                filters.iter().find(|filter| value == *filter).is_some()
            }
            PriceFilter::Contains(filter) => value.contains(filter),
            PriceFilter::All => true,
        }
    }
}

/// Precomputes price data
///
/// Data is pushed to buffer before being available. Buffer is drained on pushes no more often than
/// one second
pub(crate) struct Prices {
    map: AHashMap<String, f32>,
    ordered: Vec<(String, f32)>,
    filters: PriceFilter,
    filtered: Vec<(String, f32)>,
    buffer: Vec<(String, f32)>,
    buffer_last_drained: Instant,
}

impl Default for Prices {
    fn default() -> Self {
        Self::new()
    }
}

impl Prices {
    fn new() -> Self {
        Self {
            map: AHashMap::default(),
            ordered: Vec::new(),
            filters: PriceFilter::Matches(Vec::new()),
            filtered: Vec::new(),
            buffer: Vec::new(),
            buffer_last_drained: Instant::now() - Duration::from_secs(1),
        }
    }

    pub(crate) fn price(&self, name: &str) -> f32 {
        *self.map.get(name).unwrap_or(&0.0)
    }

    /// Immediately applies filter to update UI
    fn run_filter_now(&mut self) {
        self.filtered = self
            .ordered
            .iter()
            .filter_map(|(name, price)| {
                if self.filters.apply(name) {
                    Some((name.clone(), *price))
                } else {
                    None
                }
            })
            .collect();
    }

    pub(crate) fn add(&mut self, name: String, price: f32) {
        self.buffer.push((name, price));

        if self.buffer_last_drained.elapsed().as_secs() < 1 {
            return;
        }
        self.buffer_last_drained = Instant::now();

        debug!("draining {} prices", self.buffer.len());

        self.map.extend(self.buffer.drain(..));

        self.ordered = self.map.iter().map(|(k, v)| (k.to_owned(), *v)).collect();
        self.ordered
            .sort_by(|(_, p1), (_, p2)| p2.partial_cmp(p1).unwrap());

        self.run_filter_now();
    }

    pub(crate) fn apply_filter(&mut self, filter: PriceFilter) {
        self.filters = filter;
        self.run_filter_now();
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.ordered.is_empty()
    }

    pub(crate) fn descending(&self) -> impl Iterator<Item = &(String, f32)> {
        self.ordered.iter()
    }

    pub(crate) fn descending_filtered(&self) -> impl Iterator<Item = &(String, f32)> {
        self.filtered.iter()
    }
}

#[derive(Default)]
pub(crate) struct AppData {
    pub(crate) prices: Prices,
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
