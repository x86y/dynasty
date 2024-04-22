use std::{error::Error, sync::atomic::AtomicBool};

use binance::websockets::agg_trade_stream;
use iced::subscription::{self, Subscription};
use serde::{de, Deserialize, Deserializer};

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

fn str_as_f32<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <&str>::deserialize(deserializer)?;
    s.parse::<f32>().map_err(de::Error::custom)
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct TradesEvent {
    #[serde(rename = "p", deserialize_with = "str_as_f32")]
    pub(crate) price: f32,

    #[serde(rename = "q", deserialize_with = "str_as_f32")]
    pub(crate) qty: f32,

    #[serde(rename = "T")]
    pub(crate) trade_order_time: u64,

    #[serde(rename = "m")]
    pub(crate) is_buyer_maker: bool,
}

impl WsListener for TradesWs {
    type Event = TradesEvent;
    type Input = Message;
    type Output = TradesEvent;

    fn message(&self, msg: WsEvent<Self::Input, Self::Output>) -> WsMessage {
        WsMessage::Trade(msg)
    }

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
}

pub(crate) fn connect(pair: String) -> Subscription<WsMessage> {
    struct Connect;

    subscription::channel(std::any::TypeId::of::<Connect>(), 100, |output| async {
        TradesWs::new(pair).run(output).await
    })
}
