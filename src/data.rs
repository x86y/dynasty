use ahash::AHashMap;
use std::{
    collections::BTreeMap,
    mem::MaybeUninit,
    time::{Duration, Instant},
};
use tracing::trace;

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
            PriceFilter::Matches(filters) => filters.iter().any(|filter| value == *filter),
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
    buffer: Vec<(String, f32)>,
    buffer_drained_at: Instant,
    map: AHashMap<String, f32>,
    ordered: Vec<(String, f32)>,
    sort_descending: bool,
    filter: PriceFilter,
}

impl Default for Prices {
    fn default() -> Self {
        Self::new()
    }
}

impl Prices {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            buffer_drained_at: Instant::now() - Duration::from_secs(1),
            map: AHashMap::default(),
            ordered: Vec::new(),
            sort_descending: true,
            filter: PriceFilter::Matches(Vec::new()),
        }
    }

    pub(crate) fn price(&self, name: &str) -> f32 {
        *self.map.get(name).unwrap_or(&0.0)
    }

    fn filter_now(&mut self) {
        self.ordered = self.map.iter().map(|(k, v)| (k.to_owned(), *v)).collect();
        self.ordered = self
            .ordered
            .iter()
            .filter_map(|(name, price)| {
                if self.filter.apply(name) {
                    Some((name.clone(), *price))
                } else {
                    None
                }
            })
            .collect();

        self.sort_now();
    }

    fn sort_now(&mut self) {
        let sort_pred = if self.sort_descending {
            |(_, p1): &(_, f32), (_, p2): &(_, f32)| p2.partial_cmp(p1).unwrap()
        } else {
            |(_, p1): &(_, f32), (_, p2): &(_, f32)| p1.partial_cmp(p2).unwrap()
        };
        self.ordered.sort_by(sort_pred);
    }

    pub(crate) fn add(&mut self, name: String, price: f32) {
        self.buffer.push((name, price));

        if self.buffer_drained_at.elapsed().as_secs() < 1 {
            return;
        }
        self.buffer_drained_at = Instant::now();

        trace!("draining {} prices", self.buffer.len());

        self.map.extend(self.buffer.drain(..));

        self.filter_now();
    }

    /// Sets filter
    ///
    /// This immediately applies
    pub(crate) fn set_filter(&mut self, filter: PriceFilter) {
        trace!("filtering by {:?}", filter);

        self.filter = filter;
        self.filter_now();
    }

    /// Inverts sorting
    ///
    /// This immediately applies
    pub(crate) fn flip_sort(&mut self) {
        self.sort_descending = !self.sort_descending;
        self.ordered.reverse();
    }

    /// Restores original sorting order
    ///
    /// This does not immediately apply
    pub(crate) fn reset_sort(&mut self) {
        self.sort_descending = true;
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub(crate) fn all(&self) -> impl Iterator<Item = (&String, &f32)> {
        self.map.iter()
    }

    pub(crate) fn sorted_and_filtered(&self) -> impl Iterator<Item = &(String, f32)> {
        self.ordered.iter()
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
    pub(crate) price_chart: StaticLocalRb<f64, 500>,
}
