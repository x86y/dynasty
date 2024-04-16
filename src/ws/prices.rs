use std::{error::Error, sync::atomic::AtomicBool};

use binance::ws_model::DayTickerEvent;
use iced::subscription::{self, Subscription};

use crate::ws::WsEvent;

use super::{WsListener, WsMessage};

#[derive(Debug, Clone)]
pub struct AssetDetails {
    pub name: String,
    pub price: f32,
}

#[derive(Debug)]
pub(crate) struct PricesWs {}

impl PricesWs {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl WsListener for PricesWs {
    type Event = DayTickerEvent;
    type Input = ();
    type Output = AssetDetails;

    fn message(&self, msg: WsEvent<Self::Input, Self::Output>) -> WsMessage {
        WsMessage::Price(msg)
    }

    async fn endpoint(&self) -> Result<String, Box<dyn Error + Send>> {
        Ok("!ticker".to_owned())
    }

    fn handle_event(&self, event: Self::Event) -> Self::Output {
        AssetDetails {
            name: event.symbol,
            price: event.best_bid.parse::<f32>().unwrap(),
        }
    }

    fn handle_input(&mut self, _: Self::Input, _: &mut AtomicBool) {}
}

pub fn connect() -> Subscription<WsMessage> {
    struct Connect;

    subscription::channel(std::any::TypeId::of::<Connect>(), 100, |output| async {
        PricesWs::new().run(output).await
    })
}
