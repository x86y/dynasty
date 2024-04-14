use std::sync::atomic::AtomicBool;

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
pub(crate) enum WsEvent<T, M> {
    /// Connection established
    ///
    /// Optionally contains initialization data (currently channel for sending commands)
    Connected(mpsc_tokio::UnboundedSender<M>),

    /// Connection closed
    Disconnected,

    /// Websocket message
    Message(T),
}

#[derive(Debug, Clone)]
pub(crate) enum WsMessage {
    Trade(WsEvent<TradesEvent, trades::Message>),
    Book(WsEvent<OrderBookDetails, book::Message>),
    Price(WsEvent<AssetDetails, ()>),
    User(WsEvent<WebsocketEvent, ()>),
}

pub(crate) trait WsListener {
    type Event: DeserializeOwned;
    type Input;
    type Output: Send;

    /// Get iced output handle
    fn output(&mut self) -> &mut mpsc_futures::Sender<WsMessage>;

    /// Endpoint given to `web_socket.connect`
    async fn endpoint(&self) -> String;

    /// Turn websocket event into output
    fn handle_event(event: Self::Event) -> Self::Output;

    /// Handle message from input channel
    fn handle_input(&mut self, input: Self::Input, keep_running: &mut AtomicBool);

    /// Wrap `WsEvent` into correct variant of WsMessage
    fn message(&self, msg: WsEvent<Self::Output, Self::Input>) -> WsMessage;

    /// Main entrypoint
    async fn run(&mut self) -> ! {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let mut web_socket = WebSockets::new(|event| {
            let _ = tx.send(Self::handle_event(event));

            Ok(())
        });

        loop {
            let mut keep_running = AtomicBool::new(true);
            let endpoint = self.endpoint().await;

            match web_socket.connect(&endpoint).await {
                Ok(()) => {
                    let (input_tx, mut input_rx) = mpsc_tokio::unbounded_channel();

                    let connect_msg = self.message(WsEvent::Connected(input_tx));
                    let _ = self.output().send(connect_msg).await;

                    loop {
                        futures::select! {
                            ws_closed = web_socket.event_loop(&keep_running).fuse() => {
                                if let Err(e) = ws_closed {
                                    eprintln!("Stream error: {e:?}");
                                }
                                break;
                            }
                            event = rx.recv().fuse() => {
                                let event = event.expect("nobody should be closing channel");
                                let message = self.message(WsEvent::Message(event));
                                let _ = self.output().send(message).await;
                            }
                            input_message = input_rx.recv().fuse() => {
                                if let Some(input) = input_message {
                                    self.handle_input(input, &mut keep_running);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("WebSocket connection error: {e:?}");
                }
            }
            let message = self.message(WsEvent::Disconnected);
            let _ = self.output().send(message).await;
        }
    }
}
