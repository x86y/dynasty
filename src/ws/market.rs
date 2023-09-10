use iced::futures::FutureExt;
use iced::subscription::{self, Subscription};
use iced_futures::futures;

use binance::{websockets::*, ws_model::WebsocketEventUntag};
use futures::channel::mpsc;
use futures::sink::SinkExt;
use std::fmt;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[derive(Debug, Clone)]
pub struct BookTickerDetails {
    pub bid: f64,
    pub ask: f64,
    pub bid_qty: f64,
    pub ask_qty: f64,
}

pub fn connect(token: String) -> Subscription<MarketEvent> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let keep_running = AtomicBool::new(true);
            let book_ticker: String = book_ticker_stream(&token.clone());
            let (s, mut r): (
                UnboundedSender<BookTickerDetails>,
                UnboundedReceiver<BookTickerDetails>,
            ) = unbounded_channel();

            let mut web_socket: WebSockets<'_, WebsocketEventUntag> =
                WebSockets::new(|events: WebsocketEventUntag| {
                    if let binance::ws_model::WebsocketEventUntag::BookTicker(te) = events {
                        let _ = s.send(BookTickerDetails { bid: te.best_bid, ask: te.best_ask, bid_qty: te.best_bid_qty, ask_qty: te.best_ask_qty });
                    };
                    Ok(())
                });

            loop {
                web_socket.connect(&book_ticker).await.unwrap();
                loop {
                    futures::select! {
                        _recv = web_socket.event_loop(&keep_running).fuse() => continue ,
                        recv2 = r.recv().fuse() => {
                                if let Some(i) = recv2 {
                                    output
                                        .send(MarketEvent::MessageReceived(Message::BookTicker(i)))
                                        .await
                                        .unwrap();
                                };
                         }
                    };
                }
            }
        },
    )
}

#[derive(Debug, Clone)]
pub enum MarketEvent {
    MessageReceived(Message),
}

#[derive(Debug, Clone)]
pub struct Connection(mpsc::Sender<Message>);

#[derive(Debug, Clone)]
pub enum Message {
    BookTicker(BookTickerDetails),
    Input(String),
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::BookTicker(message) => write!(f, "{message:?}"),
            _ => unreachable!(),
        }
    }
}
