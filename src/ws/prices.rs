use iced::futures::FutureExt;
use iced::subscription::{self, Subscription};

use binance::{websockets::*, ws_model::WebsocketEvent};
use futures::sink::SinkExt;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[derive(Debug, Clone)]
pub struct AssetDetails {
    pub name: String,
    pub price: f32,
}

pub fn connect() -> Subscription<Event> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let keep_running = AtomicBool::new(true);
            let book_ticker: &'static str = all_ticker_stream();

            let (s, mut r): (
                UnboundedSender<AssetDetails>,
                UnboundedReceiver<AssetDetails>,
            ) = unbounded_channel();

            let mut web_socket: WebSockets<'_, Vec<WebsocketEvent>> =
                WebSockets::new(|events: Vec<WebsocketEvent>| {
                    for ev in &events {
                        if let WebsocketEvent::DayTicker(tick_event) = ev {
                            let asset = AssetDetails {
                                name: tick_event.symbol.clone(),
                                price: tick_event.best_bid.parse::<f32>().unwrap(),
                            };
                            s.send(asset).unwrap();
                        }
                    }
                    Ok(())
                });

            loop {
                web_socket.connect(book_ticker).await.unwrap();
                loop {
                    futures::select! {
                        _recv = web_socket.event_loop(&keep_running).fuse() => continue ,
                        recv2 = r.recv().fuse() => {
                                if let Some(i) = recv2 {
                                    output
                                        .send(Event::MessageReceived(i))
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
pub enum Event {
    MessageReceived(AssetDetails),
}
