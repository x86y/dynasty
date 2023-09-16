use binance::ws_model::TradesEvent;
use iced::futures::FutureExt;
use iced::subscription::{self, Subscription};
use iced_futures::futures;

use binance::{websockets::*, ws_model::WebsocketEvent};
use futures::channel::mpsc;
use futures::sink::SinkExt;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub fn connect(token: String) -> Subscription<Event> {
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
                web_socket.connect(&book_ticker).await.unwrap();
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
    MessageReceived(TradesEvent),
}

#[derive(Debug, Clone)]
pub struct Connection(mpsc::Sender<Event>);
