use iced::subscription::{self, Subscription};
use iced_futures::futures;

use binance::{api::Binance, userstream::UserStream, websockets::*, ws_model::WebsocketEvent};
use futures::sink::SinkExt;
use futures::FutureExt;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::api::PUB;

pub fn connect() -> Subscription<WsUpdate> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let keep_running = AtomicBool::new(true);
            let user_stream: UserStream = Binance::new(PUB.clone(), None);
            let (s, mut r): (
                UnboundedSender<WebsocketEvent>,
                UnboundedReceiver<WebsocketEvent>,
            ) = unbounded_channel();

            loop {
                if let Ok(answer) = user_stream.start().await {
                    let listen_key = answer.listen_key;

                    let mut web_socket: WebSockets<'_, WebsocketEvent> =
                        WebSockets::new(|event: WebsocketEvent| {
                            let _ = s.send(event);
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
                                                .send(WsUpdate::UpdateReceived(i))
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
pub enum WsUpdate {
    UpdateReceived(WebsocketEvent),
}
