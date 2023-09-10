use iced::subscription::{self, Subscription};
use iced_futures::futures;

use binance::{api::Binance, userstream::UserStream, websockets::*, ws_model::WebsocketEvent};
use futures::sink::SinkExt;
use futures::{channel::mpsc, FutureExt};
use std::fmt;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub fn connect() -> Subscription<MarketEvent> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let keep_running = AtomicBool::new(true);
            let user_stream: UserStream = Binance::new(Some("".to_string()), None);
            let (s, mut r): (UnboundedSender<()>, UnboundedReceiver<()>) = unbounded_channel();

            loop {
                if let Ok(answer) = user_stream.start().await {
                    let listen_key = answer.listen_key;

                    let mut web_socket: WebSockets<'_, WebsocketEvent> =
                        WebSockets::new(|event: WebsocketEvent| {
                            dbg!(&event);
                            if let WebsocketEvent::OrderUpdate(trade) = event {
                                println!(
                                    "Symbol: {}, Side: {:?}, Price: {}, Execution Type: {:?}",
                                    trade.symbol, trade.side, trade.price, trade.execution_type
                                );
                            };

                            Ok(())
                        });
                    loop {
                        web_socket.connect(&listen_key).await.unwrap(); // check error
                        loop {
                            futures::select! {
                                _recv = web_socket.event_loop(&keep_running).fuse() => continue ,
                                recv2 = r.recv().fuse() => {
                                        if let Some(i) = recv2 {
                                            output
                                                .send(MarketEvent::MessageReceived(Message::BookTicker(i)))
                                                .await
                                                .unwrap();
                                        };
                                 }
                            };
                        }
                    }
                }
            }
        },
    )
}

#[derive(Debug, Clone)]
pub enum MarketEvent {
    MessageReceived(Message),
}

#[derive(Debug, Clone)]
pub struct Connection(mpsc::Sender<Message>);

#[derive(Debug, Clone)]
pub enum Message {
    BookTicker(()),
    Input(String),
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::BookTicker(message) => write!(f, "{message:?}"),
            _ => unreachable!(),
        }
    }
}
