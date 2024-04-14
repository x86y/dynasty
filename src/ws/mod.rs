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
    /// Connection established
    ///
    /// Optionally contains initialization data (currently channel for sending commands)
    Connected(M),

    /// Connection closed
    Disconnected,

    /// Websocket message
    Message(T),
}

#[derive(Debug, Clone)]
pub(crate) enum WsMessage {
    Trade(WsEvent<TradesEvent, mpsc::UnboundedSender<trades::Message>>),
    Book(WsEvent<OrderBookDetails, mpsc::UnboundedSender<book::Message>>),
    Price(WsEvent<AssetDetails, ()>),
    User(WebsocketEvent),
}
