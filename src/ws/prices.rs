use std::{error::Error, sync::atomic::AtomicBool};

use binance::websockets::all_ticker_stream;
use iced::subscription::{self, Subscription};
use serde::{de, Deserialize, Deserializer};

use crate::ws::WsEvent;

use super::{WsListener, WsMessage};

fn str_as_f32<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <&str>::deserialize(deserializer)?;
    s.parse::<f32>().map_err(de::Error::custom)
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct AssetDetails {
    #[serde(rename = "s")]
    pub(crate) name: String,

    #[serde(rename = "b", deserialize_with = "str_as_f32")]
    pub(crate) price: f32,
}

#[derive(Debug)]
pub(crate) struct PricesWs {}

impl PricesWs {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl WsListener for PricesWs {
    type Event = Vec<AssetDetails>;
    type Input = ();
    type Output = Vec<AssetDetails>;

    fn message(&self, msg: WsEvent<Self::Input, Self::Output>) -> WsMessage {
        WsMessage::Price(msg)
    }

    async fn endpoint(&self) -> Result<String, Box<dyn Error + Send>> {
        Ok(all_ticker_stream().to_owned())
    }

    fn handle_event(&self, event: Self::Event) -> Self::Output {
        event
    }

    fn handle_input(&mut self, _: Self::Input, _: &mut AtomicBool) {}
}

pub fn connect() -> Subscription<WsMessage> {
    struct Connect;

    subscription::channel(std::any::TypeId::of::<Connect>(), 100, |output| async {
        PricesWs::new().run(output).await
    })
}
