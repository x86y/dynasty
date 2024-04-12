use binance::ws_model::{TradesEvent, WebsocketEvent};
use tokio::sync::mpsc;

use self::{book::OrderBookDetails, prices::AssetDetails};

pub mod book;
pub mod prices;
pub mod trades;
pub mod user;
pub mod util;

#[derive(Debug, Clone)]
pub(crate) enum WsEvent<T, M> {
    Connected(mpsc::UnboundedSender<M>),
    Disconnected,
    Message(T),
}

#[derive(Debug, Clone)]
pub(crate) enum WsUpdate {
    Trade(WsEvent<TradesEvent, trades::Message>),
    Book(WsEvent<OrderBookDetails, book::Message>),
    Price(WsEvent<AssetDetails, ()>),
    User(WebsocketEvent),
}
