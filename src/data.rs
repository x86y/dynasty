use std::collections::{BTreeMap, HashMap, VecDeque};

use binance::rest_model::{Balance, Order};

use crate::{views::components::loading::Loader, ws::trades::TradesEvent};

#[derive(Debug, Default)]
pub(crate) struct AppData {
    pub(crate) prices: HashMap<String, f32>,
    pub(crate) book: (String, BTreeMap<String, f64>, BTreeMap<String, f64>),
    pub(crate) trades: VecDeque<TradesEvent>,
    pub(crate) balances: Vec<Balance>,
    pub(crate) orders: Vec<Order>,
    pub(crate) quote: String,
    pub(crate) loader: Loader,
}
