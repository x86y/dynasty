use std::sync::atomic::AtomicBool;

use binance::websockets::agg_trade_stream;
use binance::{websockets::WebSockets, ws_model::WebsocketEvent};
use futures::sink::SinkExt;
use iced::futures::FutureExt;
use iced::subscription::{self, Subscription};
use tokio::sync::mpsc;

use crate::ws::WsEvent;

use super::WsUpdate;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    NewPair(String),
}

#[allow(clippy::large_enum_variant)]
enum State<'a> {
    Disconnected,
    Connected(
        WebSockets<'a, binance::ws_model::WebsocketEvent>,
        mpsc::UnboundedReceiver<Message>,
    ),
}

pub fn connect(mut pair: String) -> Subscription<WsUpdate> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let mut state = State::Disconnected;

            let keep_running = AtomicBool::new(true);

            loop {
                match &mut state {
                    State::Disconnected => {
                        let mut output_clone = output.clone();
                        let mut web_socket: WebSockets<'_, WebsocketEvent> =
                            WebSockets::new(move |events: WebsocketEvent| {
                                if let WebsocketEvent::AggTrade(tick_event) = events {
                                    let _ = output_clone
                                        .try_send(WsUpdate::Trade(WsEvent::Message(*tick_event)));
                                };

                                Ok(())
                            });

                        let book_ticker: String = agg_trade_stream(&pair);
                        match web_socket.connect(&book_ticker).await {
                            Ok(()) => {
                                let (sender, receiver) = mpsc::unbounded_channel();

                                let _ = output
                                    .send(WsUpdate::Trade(WsEvent::Connected(sender)))
                                    .await;
                                state = State::Connected(web_socket, receiver);
                            }
                            Err(e) => {
                                eprintln!("WebSocket connection error: {e:?}");
                            }
                        };
                    }
                    State::Connected(web_socket, input) => {
                        futures::select! {
                            ws_closed = web_socket.event_loop(&keep_running).fuse() => {
                                if ws_closed.is_err() {
                                    eprintln!("Trade stream error: {:?}", ws_closed.unwrap_err());
                                }
                                let _ = output
                                    .send(WsUpdate::Trade(WsEvent::Disconnected))
                                    .await;
                                state = State::Disconnected;
                            },
                            input_message = input.recv().fuse() => {
                                if let Some(input) = input_message {
                                    match input {
                                        Message::NewPair(new_pair) => {
                                            pair = new_pair;
                                        }
                                    };
                                }

                                let _ = output
                                    .send(WsUpdate::Trade(WsEvent::Disconnected))
                                    .await;
                                state = State::Disconnected;
                            }
                        };
                    }
                }
            }
        },
    )
}
