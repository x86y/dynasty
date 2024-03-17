use binance::{websockets::*, ws_model::WebsocketEventUntag};
use futures::channel::mpsc;
use futures::sink::SinkExt;
use iced::futures::FutureExt;
use iced::subscription::{self, Subscription};
use iced_futures::futures;
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[derive(Debug, Clone)]
pub struct OrderBookDetails {
    pub sym: String,
    pub bids: BTreeMap<String, f64>,
    pub asks: BTreeMap<String, f64>,
}

pub fn connect(token: String) -> Subscription<BookEvent> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let keep_running = AtomicBool::new(true);
            let order_book: String = diff_book_depth_stream(&token.clone(), 1000);

            let (s, mut r): (
                UnboundedSender<OrderBookDetails>,
                UnboundedReceiver<OrderBookDetails>,
            ) = unbounded_channel();

            let mut web_socket: WebSockets<'_, binance::ws_model::DepthOrderBookEvent> =
                WebSockets::new(|events: binance::ws_model::DepthOrderBookEvent| {
                    if let binance::ws_model::DepthOrderBookEvent {
                        event_time: du,
                        symbol,
                        first_update_id,
                        final_update_id,
                        bids,
                        asks,
                    } = events
                    {
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

                        let _ = s.send(OrderBookDetails {
                            sym: symbol,
                            bids: b,
                            asks: a,
                        });
                    };
                    Ok(())
                });

            loop {
                web_socket.connect(&order_book).await.unwrap();
                loop {
                    futures::select! {
                        _recv = web_socket.event_loop(&keep_running).fuse() => continue,
                        recv2 = r.recv().fuse() => {
                            if let Some(i) = recv2 {
                                output
                                    .send(BookEvent::MessageReceived(i))
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
pub enum BookEvent {
    MessageReceived(OrderBookDetails),
}

#[derive(Debug, Clone)]
pub struct Connection(mpsc::Sender<OrderBookDetails>);
