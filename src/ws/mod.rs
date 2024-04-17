use std::{error::Error, sync::atomic::AtomicBool, time::Duration};

use binance::{
    websockets::WebSockets,
    ws_model::{TradesEvent, WebsocketEvent},
};
use futures::{channel::mpsc as mpsc_futures, FutureExt, SinkExt};
use serde::de::DeserializeOwned;
use tokio::sync::mpsc as mpsc_tokio;

use self::{book::OrderBookDetails, prices::AssetDetails};

pub mod book;
pub mod prices;
pub mod trades;
pub mod user;
pub mod util;

#[derive(Debug, Clone)]
pub(crate) enum WsEvent<In, Out> {
    /// Websocket created
    ///
    /// Contains channel for sending input messages
    Created(mpsc_tokio::UnboundedSender<In>),

    /// Connected successfully
    Connected,

    /// Connection closed
    Disconnected,

    /// Websocket message
    Message(Out),
}

#[derive(Debug, Clone)]
pub(crate) enum WsMessage {
    Trade(WsEvent<trades::Message, TradesEvent>),
    Book(WsEvent<book::Message, OrderBookDetails>),
    Price(WsEvent<(), AssetDetails>),
    User(WsEvent<user::Message, WebsocketEvent>),
}

pub(crate) trait WsListener {
    type Event: Send + DeserializeOwned;
    type Input;
    type Output;

    /// Wrap `WsEvent` in correct variant of `WsMessage`
    fn message(&self, msg: WsEvent<Self::Input, Self::Output>) -> WsMessage;

    /// Endpoint given to `web_socket.connect`
    async fn endpoint(&self) -> Result<String, Box<dyn Error + Send>>;

    /// Handle websocket event
    fn handle_event(&self, event: Self::Event) -> Self::Output;

    /// Handle message from input channel
    ///
    /// `keep_running` can disconnect websocket if set to false
    fn handle_input(&mut self, input: Self::Input, keep_running: &mut AtomicBool);

    /// Main entrypoint
    async fn run(&mut self, mut output: mpsc_futures::Sender<WsMessage>) -> ! {
        // forward messages out of websocket callback
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let mut web_socket = WebSockets::new(|event| {
            let _ = tx.send(event);

            Ok(())
        });

        let (input_tx, mut input_rx) = mpsc_tokio::unbounded_channel();

        let connected = self.message(WsEvent::Created(input_tx));
        let _ = output.send(connected).await;

        loop {
            let mut keep_running = AtomicBool::new(true);

            // input might alter endpoint result
            if let Ok(input) = input_rx.try_recv() {
                self.handle_input(input, &mut keep_running);
                continue;
            }

            let endpoint = match self.endpoint().await {
                Ok(endpoint) => endpoint,
                Err(e) => {
                    eprintln!("Endpoint error: {e:?}");

                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };

            if let Err(e) = web_socket.connect(&endpoint).await {
                eprintln!("WebSocket connection error: {e:?}");

                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }

            let connected = self.message(WsEvent::Connected);
            let _ = output.send(connected).await;

            loop {
                futures::select! {
                    ws_closed = web_socket.event_loop(&keep_running).fuse() => {
                        if let Err(e) = ws_closed {
                            eprintln!("WebSocket stream error: {e:?}");
                        }
                        break;
                    }
                    event = rx.recv().fuse() => {
                        let handled = self.handle_event(event.expect("nobody should be closing channel"));
                        let message = self.message(WsEvent::Message(handled));
                        let _ = output.send(message).await;
                    }
                    input = input_rx.recv().fuse() => {
                        if let Some(input) = input {
                            self.handle_input(input, &mut keep_running);
                        }
                    }
                }
            }

            let disconnected = self.message(WsEvent::Disconnected);
            let _ = output.send(disconnected).await;
        }
    }
}
