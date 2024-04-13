use std::sync::atomic::AtomicBool;

use binance::{
    api::Binance, userstream::UserStream, websockets::WebSockets, ws_model::WebsocketEvent,
};
use iced::subscription::{self, Subscription};

use super::WsUpdate;

pub fn connect(public: String) -> Subscription<WsUpdate> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let keep_running = AtomicBool::new(true);
            let user_stream: UserStream = Binance::new(Some(public), None);
            loop {
                if let Ok(answer) = user_stream.start().await {
                    let listen_key = answer.listen_key;

                    let mut web_socket: WebSockets<'_, WebsocketEvent> =
                        WebSockets::new(|event: WebsocketEvent| {
                            let _ = output.try_send(WsUpdate::User(event));
                            Ok(())
                        });
                    loop {
                        match web_socket.connect(&listen_key).await {
                            Ok(()) => loop {
                                if let Err(e) = web_socket.event_loop(&keep_running).await {
                                    eprintln!("User Stream error: {e:?}");
                                    break;
                                };
                            },
                            Err(e) => {
                                eprintln!("WebSocket connection error: {e:?}");
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
