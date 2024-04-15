use std::{error::Error, sync::atomic::AtomicBool};

use binance::websockets::diff_book_depth_stream;
use iced::subscription::{self, Subscription};
use std::collections::BTreeMap;

use super::{WsEvent, WsListener, WsMessage};

#[derive(Debug, Clone)]
pub(crate) struct OrderBookDetails {
    pub(crate) sym: String,
    pub(crate) bids: BTreeMap<String, f64>,
    pub(crate) asks: BTreeMap<String, f64>,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    NewPair(String),
}

#[derive(Debug)]
struct BookWs {
    pair: String,
}

impl BookWs {
    fn new(pair: String) -> Self {
        Self { pair }
    }
}

impl WsListener for BookWs {
    type Event = binance::ws_model::DepthOrderBookEvent;
    type Input = Message;
    type Output = OrderBookDetails;

    async fn endpoint(&self) -> Result<String, Box<dyn Error + Send>> {
        Ok(diff_book_depth_stream(&self.pair, 1000))
    }

    fn handle_event(&self, event: Self::Event) -> Self::Output {
        let binance::ws_model::DepthOrderBookEvent {
            event_time: _,
            symbol,
            first_update_id: _,
            final_update_id: _,
            bids,
            asks,
        } = event;
        let mut b: BTreeMap<String, f64> = BTreeMap::new();
        let mut a: BTreeMap<String, f64> = BTreeMap::new();

        for bid in bids {
            let price = bid.price;
            let quantity = bid.qty;
            if quantity == 0.0 {
                b.remove(&price.to_string());
            } else {
                b.insert(price.to_string(), quantity);
            }
        }

        for ask in asks {
            let price = ask.price;
            let quantity = ask.qty;
            if quantity == 0.0 {
                a.remove(&price.to_string());
            } else {
                a.insert(price.to_string(), quantity);
            }
        }

        OrderBookDetails {
            sym: symbol,
            bids: b,
            asks: a,
        }
    }

    fn handle_input(&mut self, input: Self::Input, keep_running: &mut AtomicBool) {
        match input {
            Message::NewPair(new_pair) => {
                self.pair = new_pair;
                keep_running.store(false, std::sync::atomic::Ordering::Relaxed);
            }
        };
    }

    fn message(&self, msg: WsEvent<Self::Input, Self::Output>) -> WsMessage {
        WsMessage::Book(msg)
    }
}

pub fn connect(pair: String) -> Subscription<WsMessage> {
    struct Connect;

    subscription::channel(std::any::TypeId::of::<Connect>(), 100, |output| async {
        BookWs::new(pair).run(output).await
    })
}
