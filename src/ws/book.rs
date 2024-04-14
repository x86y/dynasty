use std::sync::atomic::AtomicBool;

use binance::websockets::{diff_book_depth_stream, WebSockets};
use futures::sink::SinkExt;
use iced::futures::FutureExt;
use iced::subscription::{self, Subscription};
use std::collections::BTreeMap;
use tokio::sync::mpsc;

use super::{WsEvent, WsMessage};

#[derive(Debug, Clone)]
pub struct OrderBookDetails {
    pub sym: String,
    pub bids: BTreeMap<String, f64>,
    pub asks: BTreeMap<String, f64>,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    NewPair(String),
}

#[allow(clippy::large_enum_variant)]
enum State<'a> {
    Disconnected,
    Connected(
        WebSockets<'a, binance::ws_model::DepthOrderBookEvent>,
        mpsc::UnboundedReceiver<Message>,
    ),
}

pub fn connect(mut pair: String) -> Subscription<WsMessage> {
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
                        let order_book: String = diff_book_depth_stream(&pair.clone(), 1000);

                        let mut output_clone = output.clone();
                        let mut web_socket: WebSockets<'_, binance::ws_model::DepthOrderBookEvent> =
                            WebSockets::new(
                                move |events: binance::ws_model::DepthOrderBookEvent| {
                                    let binance::ws_model::DepthOrderBookEvent {
                                        event_time: _,
                                        symbol,
                                        first_update_id: _,
                                        final_update_id: _,
                                        bids,
                                        asks,
                                    } = events;
                                    let mut b: BTreeMap<String, f64> = BTreeMap::new();
                                    let mut a: BTreeMap<String, f64> = BTreeMap::new();

                                    for bid in bids {
                                        let price = bid.price;
                                        let quantity = bid.qty;
                                        if quantity == 0.0 {
                                            b.remove(&price.to_string());
                                        } else {
                                            b.insert(price.to_string(), quantity);
                                        }
                                    }

                                    for ask in asks {
                                        let price = ask.price;
                                        let quantity = ask.qty;
                                        if quantity == 0.0 {
                                            a.remove(&price.to_string());
                                        } else {
                                            a.insert(price.to_string(), quantity);
                                        }
                                    }

                                    let _ = output_clone.try_send(WsMessage::Book(
                                        WsEvent::Message(OrderBookDetails {
                                            sym: symbol,
                                            bids: b,
                                            asks: a,
                                        }),
                                    ));

                                    Ok(())
                                },
                            );

                        match web_socket.connect(&order_book).await {
                            Ok(()) => {
                                let (sender, receiver) = mpsc::unbounded_channel();

                                let _ = output
                                    .send(WsMessage::Book(WsEvent::Connected(sender)))
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
                                    eprintln!("Orderbook stream error: {:?}", ws_closed.unwrap_err());
                                }
                            },
                            input_message = input.recv().fuse() => {
                                if let Some(input) = input_message {
                                    match input {
                                        Message::NewPair(new_pair) => {
                                            pair = new_pair;
                                        }
                                    };
                                }
                            }
                        };

                        let _ = output.send(WsMessage::Book(WsEvent::Disconnected)).await;
                        state = State::Disconnected;
                    }
                }
            }
        },
    )
}
