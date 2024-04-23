use binance::rest_model::{Order, OrderStatus};
use iced::{Command, Subscription};
use tokio::sync::mpsc;

use self::listener::WsListener;
use crate::{data::AppData, message::Message, views::dashboard::DashboardView};

mod book;
mod listener;
pub(crate) mod prices;
pub(crate) mod trades;
mod user;

#[derive(Debug, Clone)]
pub(crate) enum WsEvent<In, Out> {
    /// Websocket created
    ///
    /// Contains channel for sending input messages
    Created(WsHandle<In>),

    /// Connected successfully
    Connected,

    /// Connection closed
    Disconnected,

    /// Websocket message
    Message(Out),
}

#[derive(Debug, Clone)]
pub(crate) enum WsMessage {
    Trade(
        WsEvent<<trades::TradesWs as WsListener>::Input, <trades::TradesWs as WsListener>::Output>,
    ),
    Book(WsEvent<<book::BookWs as WsListener>::Input, <book::BookWs as WsListener>::Output>),
    Price(
        WsEvent<<prices::PricesWs as WsListener>::Input, <prices::PricesWs as WsListener>::Output>,
    ),
    User(WsEvent<<user::UserWs as WsListener>::Input, <user::UserWs as WsListener>::Output>),
}

/// Allows communicating with websocket. If you drop this, ws will spin endlessly on closed channel
#[derive(Debug, Clone)]
pub(crate) struct WsHandle<T>(mpsc::UnboundedSender<T>);

impl<T> WsHandle<T> {
    pub(crate) fn send(&self, msg: T) {
        self.0.send(msg).unwrap();
    }
}
pub(crate) struct Websockets {
    currency_pair: String,
    api_key: String,
    user: Option<WsHandle<user::Message>>,
    prices: Option<WsHandle<()>>,
    book: Option<WsHandle<book::Message>>,
    trade: Option<WsHandle<trades::Message>>,
}

impl Websockets {
    pub(crate) fn new(api_key: String, currency_pair: String) -> Self {
        Self {
            user: None,
            prices: None,
            book: None,
            trade: None,
            api_key,
            currency_pair: currency_pair.to_lowercase(),
        }
    }

    pub(crate) fn relogin_user(&self, api_key: &str) {
        if let Some(ws_user) = &self.user {
            ws_user.send(user::Message::NewApiKey(api_key.to_owned()));
        };
    }

    pub(crate) fn track_new_currency_pair(&self, pair: &str) {
        let pair = pair.to_lowercase();

        if let Some(book_ws) = &self.book {
            book_ws.send(book::Message::NewPair(pair.clone()));
        };
        if let Some(ws_trade) = &self.trade {
            ws_trade.send(trades::Message::NewPair(pair));
        };
    }

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            trades::connect(self.currency_pair.clone()).map(Message::from),
            book::connect(self.currency_pair.clone()).map(Message::from),
            prices::connect().map(Message::from),
            user::connect(self.api_key.clone()).map(Message::from),
        ])
    }

    pub(crate) fn update(
        &mut self,
        msg: WsMessage,
        data: &mut AppData,
        dashboard: &mut DashboardView,
    ) -> Command<Message> {
        match msg {
            WsMessage::Book(event) => {
                match event {
                    WsEvent::Created(handle) => self.book = Some(handle),
                    WsEvent::Message(bt) => {
                        data.book = (bt.sym, bt.bids, bt.asks);
                    }
                    WsEvent::Connected | WsEvent::Disconnected => (),
                };
            }
            WsMessage::Trade(event) => match event {
                WsEvent::Created(handle) => self.trade = Some(handle),
                WsEvent::Message(te) => {
                    if data.trades.len() >= 1000 {
                        data.trades.pop_back();
                    }
                    data.trades.push_front(te);
                }
                WsEvent::Connected | WsEvent::Disconnected => (),
            },
            WsMessage::User(event) => match event {
                WsEvent::Created(handle) => self.user = Some(handle),
                WsEvent::Message(msg) => match msg {
                    binance::ws_model::WebsocketEvent::AccountPositionUpdate(p) => {
                        for b in p.balances.into_iter() {
                            let ib = data.balances.iter_mut().find(|a| a.asset == b.asset);
                            if let Some(uib) = ib {
                                *uib = unsafe { std::mem::transmute(b) }
                            }
                        }
                    }
                    binance::ws_model::WebsocketEvent::OrderUpdate(o) => {
                        let existing_order = data.orders.iter_mut().find(|order| {
                            // order.client_order_id == o.order_id&&
                            order.symbol == o.symbol
                                && order.side == o.side
                                && order.status == OrderStatus::PartiallyFilled
                        });

                        if let Some(order) = existing_order {
                            // Update the existing order with the new values
                            order.executed_qty += o.qty_last_executed;
                            order.cummulative_quote_qty += o.qty;
                            order.update_time = o.trade_order_time;
                        } else {
                            data.orders.insert(
                                0,
                                Order {
                                    symbol: o.symbol,
                                    order_id: o.order_id,
                                    order_list_id: o.order_list_id as i32,
                                    client_order_id: o.client_order_id.unwrap(),
                                    price: o.price,
                                    orig_qty: o.qty,
                                    executed_qty: o.qty_last_executed,
                                    cummulative_quote_qty: o.qty,
                                    status: o.current_order_status,
                                    time_in_force: o.time_in_force,
                                    order_type: o.order_type,
                                    side: o.side,
                                    stop_price: o.stop_price,
                                    iceberg_qty: o.iceberg_qty,
                                    time: o.event_time,
                                    update_time: o.trade_order_time,
                                    is_working: false,
                                    orig_quote_order_qty: o.qty,
                                },
                            );
                        }
                    }
                    binance::ws_model::WebsocketEvent::BalanceUpdate(_p) => {
                        // not needed imo?
                    }
                    binance::ws_model::WebsocketEvent::ListOrderUpdate(_lo) => {
                        // not needed imo?
                    }
                    _ => unreachable!(),
                },
                WsEvent::Connected | WsEvent::Disconnected => (),
            },
            WsMessage::Price(m) => {
                match m {
                    WsEvent::Created(handle) => self.prices = Some(handle),
                    WsEvent::Message(asset) => {
                        dashboard.chart_pair_price(&asset);
                        data.prices.insert(asset.name.to_owned(), asset.price);
                    }
                    WsEvent::Connected | WsEvent::Disconnected => (),
                };
            }
        }
        Command::none()
    }
}
