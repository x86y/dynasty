use std::sync::atomic::AtomicBool;

use binance::{
    websockets::{all_ticker_stream, WebSockets},
    ws_model::WebsocketEvent,
};
use futures::sink::SinkExt;
use iced::{
    futures::FutureExt,
    subscription::{self, Subscription},
};
use tokio::sync::mpsc;

use crate::ws::WsEvent;

use super::WsUpdate;

#[derive(Debug, Clone)]
pub struct AssetDetails {
    pub name: String,
    pub price: f32,
}

pub fn connect() -> Subscription<WsUpdate> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let keep_running = AtomicBool::new(true);
            let book_ticker: &'static str = all_ticker_stream();

            let (s, mut r) = mpsc::unbounded_channel();

            let mut web_socket: WebSockets<'_, Vec<WebsocketEvent>> =
                WebSockets::new(|events: Vec<WebsocketEvent>| {
                    for ev in &events {
                        if let WebsocketEvent::DayTicker(tick_event) = ev {
                            let asset = AssetDetails {
                                name: tick_event.symbol.clone(),
                                price: tick_event.best_bid.parse::<f32>().unwrap(),
                            };
                            let _ = s.send(asset);
                        }
                    }
                    Ok(())
                });

            loop {
                match web_socket.connect(book_ticker).await {
                    Ok(()) => loop {
                        futures::select! {
                            recv = web_socket.event_loop(&keep_running).fuse() => {
                                if recv.is_err() {
                                    eprintln!("Prices Stream error: {:?}", recv.unwrap_err());
                                    break;
                                }
                            },
                            message = r.recv().fuse() => {
                                if let Some(i) = message {
                                    output.send(WsUpdate::Price(WsEvent::Message(i))).await.unwrap();
                                } else {
                                    break;
                                }
                            }
                        };
                    },
                    Err(e) => {
                        eprintln!("WebSocket connection error: {e:?}");
                    }
                }
            }
        },
    )
}
