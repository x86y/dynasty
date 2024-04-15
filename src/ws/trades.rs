use std::{error::Error, sync::atomic::AtomicBool};

use binance::websockets::agg_trade_stream;
use binance::ws_model::TradesEvent;
use iced::subscription::{self, Subscription};

use crate::ws::WsEvent;

use super::{WsListener, WsMessage};

#[derive(Debug, Clone)]
pub(crate) enum Message {
    NewPair(String),
}

#[derive(Debug)]
pub(crate) struct TradesWs {
    pair: String,
}

impl TradesWs {
    pub(crate) fn new(pair: String) -> Self {
        Self { pair }
    }
}

impl WsListener for TradesWs {
    type Event = TradesEvent;
    type Input = Message;
    type Output = TradesEvent;

    async fn endpoint(&self) -> Result<String, Box<dyn Error + Send>> {
        Ok(agg_trade_stream(&self.pair))
    }

    fn handle_event(&self, event: Self::Event) -> Self::Output {
        event
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
        WsMessage::Trade(msg)
    }
}

pub(crate) fn connect(pair: String) -> Subscription<WsMessage> {
    struct Connect;

    subscription::channel(std::any::TypeId::of::<Connect>(), 100, |output| async {
        TradesWs::new(pair).run(output).await
    })
}
