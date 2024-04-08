use iced::subscription::{self, Subscription};

use binance::{api::Binance, userstream::UserStream, websockets::*, ws_model::WebsocketEvent};
use futures::sink::SinkExt;
use futures::FutureExt;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use super::WsUpdate;

pub fn connect(public: String) -> Subscription<WsUpdate> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let keep_running = AtomicBool::new(true);
            let user_stream: UserStream = Binance::new(Some(public), None);
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
                        match web_socket.connect(&listen_key).await {
                            Ok(_) => loop {
                                futures::select! {
                                    recv = web_socket.event_loop(&keep_running).fuse() => {
                                        if recv.is_err() {
                                            eprintln!("User Stream error: {:?}", recv.unwrap_err());
                                            break;
                                        }
                                    },
                                    recv2 = r.recv().fuse() => {
                                        if let Some(i) = recv2 {
                                            output.send(WsUpdate::User(i)).await.unwrap();
                                        } else {
                                            eprintln!("User Stream error");
                                            break;
                                        }
                                    },
                                };
                            },
                            Err(e) => {
                                eprintln!("WebSocket connection error: {:?}", e);
                                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                                break;
                            }
                        }
                    }
                }
            }
        },
    )
}
