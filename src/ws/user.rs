use std::{sync::atomic::AtomicBool, time::Duration};

use binance::{api::Binance, userstream::UserStream, ws_model::WebsocketEvent};
use futures::channel::mpsc;
use iced::subscription::{self, Subscription};

use crate::ws::WsEvent;

use super::{WsListener, WsMessage};

struct UserWs {
    api_key: String,
    output: mpsc::Sender<WsMessage>,
}

impl UserWs {
    fn new(api_key: String, output: mpsc::Sender<WsMessage>) -> Self {
        Self { api_key, output }
    }
}

impl WsListener for UserWs {
    type Event = WebsocketEvent;
    type Input = ();
    type Output = WebsocketEvent;

    fn output(&mut self) -> &mut mpsc::Sender<WsMessage> {
        &mut self.output
    }

    async fn endpoint(&self) -> String {
        let user_stream: UserStream = Binance::new(Some(self.api_key.clone()), None);
        loop {
            match user_stream.start().await {
                Ok(answer) => return answer.listen_key,
                Err(e) => eprintln!("Unable to get user stream key: {e:?}"),
            };

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    fn handle_event(event: Self::Event) -> Self::Output {
        event
    }

    fn message(&self, msg: WsEvent<Self::Output, Self::Input>) -> WsMessage {
        WsMessage::User(msg)
    }

    fn handle_input(&mut self, _: Self::Input, _: &mut AtomicBool) {}
}

pub fn connect(api_key: String) -> Subscription<WsMessage> {
    struct Connect;

    subscription::channel(std::any::TypeId::of::<Connect>(), 100, |output| async {
        UserWs::new(api_key, output).run().await
    })
}
