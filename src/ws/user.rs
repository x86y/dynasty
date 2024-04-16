use std::{error::Error, sync::atomic::AtomicBool};

use binance::{api::Binance, userstream::UserStream, ws_model::WebsocketEvent};
use iced::subscription::{self, Subscription};

use crate::ws::WsEvent;

use super::{WsListener, WsMessage};

#[derive(Debug, Clone)]
pub(crate) enum Message {
    NewApiKey(String),
}

#[derive(Debug)]
pub(crate) struct UserWs {
    api_key: String,
}

impl UserWs {
    fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl WsListener for UserWs {
    type Event = WebsocketEvent;
    type Input = Message;
    type Output = WebsocketEvent;

    fn message(&self, msg: WsEvent<Self::Input, Self::Output>) -> WsMessage {
        WsMessage::User(msg)
    }

    async fn endpoint(&self) -> Result<String, Box<dyn Error + Send>> {
        let user_stream: UserStream = Binance::new(Some(self.api_key.clone()), None);

        user_stream
            .start()
            .await
            .map(|answer| answer.listen_key)
            .map_err(|e| Box::new(e) as _)
    }

    fn handle_event(&self, event: Self::Event) -> Self::Output {
        event
    }

    fn handle_input(&mut self, input: Self::Input, keep_running: &mut AtomicBool) {
        match input {
            Message::NewApiKey(new_key) => {
                self.api_key = new_key;
                keep_running.store(false, std::sync::atomic::Ordering::Relaxed);
            }
        };
    }
}

pub fn connect(api_key: String) -> Subscription<WsMessage> {
    struct Connect;

    subscription::channel(std::any::TypeId::of::<Connect>(), 100, |output| async {
        UserWs::new(api_key).run(output).await
    })
}
