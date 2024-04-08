use binance::ws_model::TradesEvent;
use iced::futures::FutureExt;
use iced::subscription::{self, Subscription};

use binance::{websockets::*, ws_model::WebsocketEvent};
use futures::sink::SinkExt;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use super::WsUpdate;

pub fn connect(token: String) -> Subscription<WsUpdate> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let keep_running = AtomicBool::new(true);
            let book_ticker: String = agg_trade_stream(&token);
            let (s, mut r): (UnboundedSender<TradesEvent>, UnboundedReceiver<TradesEvent>) =
                unbounded_channel();

            let mut web_socket: WebSockets<'_, WebsocketEvent> =
                WebSockets::new(|events: WebsocketEvent| {
                    if let WebsocketEvent::AggTrade(tick_event) = events {
                        s.send(*tick_event.clone()).unwrap();
                    }
                    Ok(())
                });

            loop {
                match web_socket.connect(&book_ticker).await {
                    Ok(_) => loop {
                        futures::select! {
                            recv = web_socket.event_loop(&keep_running).fuse() => {
                                if recv.is_err() {
                                    eprintln!("Book Ticker error: {:?}", recv.unwrap_err());
                                    break;
                                }
                            },
                            recv2 = r.recv().fuse() => {
                                if let Some(i) = recv2 {
                                    output.send(WsUpdate::Trade(i)).await.unwrap();
                                } else {
                                    break
                                }
                            }
                        };
                    },
                    Err(e) => {
                        eprintln!("WebSocket connection error: {:?}", e);
                    }
                }
            }
        },
    )
}
