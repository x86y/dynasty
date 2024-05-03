use std::error::Error;

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

fn str_as_f32_as_str_formatted<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <&str>::deserialize(deserializer)?;
    let f = s.parse::<f32>().map_err(de::Error::custom)?;

    Ok(format!("{f:.2}"))
}

fn u64_as_time_formatted<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let i = <u64>::deserialize(deserializer)?;
    let timestamp = i64::try_from(i).map_err(de::Error::custom)?;
    let dt = chrono::DateTime::from_timestamp(timestamp / 1000, 0).unwrap();
    Ok(dt.format("%H:%M:%S").to_string())
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct TradesEvent {
    #[serde(rename = "p", deserialize_with = "str_as_f32_as_str_formatted")]
    pub(crate) price: String,

    #[serde(rename = "q", deserialize_with = "str_as_f32_as_str_formatted")]
    pub(crate) qty: String,

    #[serde(rename = "T", deserialize_with = "u64_as_time_formatted")]
    pub(crate) trade_order_time_formatted: String,

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

    fn handle_input(&mut self, input: Self::Input) -> bool {
        match input {
            Message::NewPair(new_pair) => {
                self.pair = new_pair;
                false
            }
        }
    }
}

pub(crate) fn connect(pair: String) -> Subscription<WsMessage> {
    struct Connect;

    subscription::channel(std::any::TypeId::of::<Connect>(), 100, |output| async {
        TradesWs::new(pair).run(output).await
    })
}
