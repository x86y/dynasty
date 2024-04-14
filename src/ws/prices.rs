use std::sync::atomic::AtomicBool;

use binance::{
    websockets::{all_ticker_stream, WebSockets},
    ws_model::WebsocketEvent,
};
use iced::subscription::{self, Subscription};
use tokio::sync::mpsc;

use crate::ws::WsEvent;

use super::WsMessage;

#[derive(Debug, Clone)]
pub struct AssetDetails {
    pub name: String,
    pub price: f32,
}

pub fn connect() -> Subscription<WsMessage> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let keep_running = AtomicBool::new(true);

            let mut output_clone = output.clone();
            let mut web_socket = WebSockets::new(|events: Vec<WebsocketEvent>| {
                for ev in events {
                    if let WebsocketEvent::DayTicker(tick_event) = ev {
                        let asset = AssetDetails {
                            name: tick_event.symbol,
                            price: tick_event.best_bid.parse::<f32>().unwrap(),
                        };
                        let _ = output_clone.try_send(WsMessage::Price(WsEvent::Message(asset)));
                    }
                }
                Ok(())
            });

            loop {
                match web_socket.connect(all_ticker_stream()).await {
                    Ok(()) => loop {
                        let (tx, _rx) = mpsc::unbounded_channel();
                        let _ = output.try_send(WsMessage::Price(WsEvent::Connected(tx)));

                        if let Err(e) = web_socket.event_loop(&keep_running).await {
                            eprintln!("Prices Stream error: {e:?}");
                            break;
                        }
                    },
                    Err(e) => {
                        eprintln!("WebSocket connection error: {e:?}");
                    }
                }
                let _ = output.try_send(WsMessage::Price(WsEvent::Disconnected));
            }
        },
    )
}
