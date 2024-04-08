use binance::ws_model::{TradesEvent, WebsocketEvent};

use self::{book::OrderBookDetails, prices::AssetDetails};

pub mod book;
pub mod prices;
pub mod trades;
pub mod user;
pub mod util;

#[derive(Debug, Clone)]
pub(crate) enum WsUpdate {
    Trade(TradesEvent),
    Book(OrderBookDetails),
    Price(AssetDetails),
    User(WebsocketEvent),
}
