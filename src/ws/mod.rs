use std::sync::atomic::AtomicBool;

use binance::{
    websockets::WebSockets,
    ws_model::{TradesEvent, WebsocketEvent},
};
use futures::{FutureExt, SinkExt};
use tokio::sync::mpsc;

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
    Connected(M),

    /// Connection closed
    Disconnected,

    /// Websocket message
    Message(T),
}

#[derive(Debug, Clone)]
pub(crate) enum WsMessage {
    Trade(WsEvent<TradesEvent, mpsc::UnboundedSender<trades::Message>>),
    Book(WsEvent<OrderBookDetails, mpsc::UnboundedSender<book::Message>>),
    Price(WsEvent<AssetDetails, ()>),
    User(WsEvent<WebsocketEvent, ()>),
}

pub(crate) trait WsListener {
    type Input;
    type Output: Send;

    /// Get iced output handle
    fn output(&mut self) -> &mut futures::channel::mpsc::Sender<WsMessage>;

    /// Endpoint given to `web_socket.connect`
    async fn endpoint(&self) -> String;

    /// TODO: always use channel
    fn connect(&mut self) -> Self::Input;

    /// Turn websocket event into output
    fn handle_event(event: WebsocketEvent) -> Self::Output;

    /// Wrap `WsEvent` into correct variant of WsMessage
    fn message(&self, msg: WsEvent<Self::Output, Self::Input>) -> WsMessage;

    /// Main entrypoint
    async fn run(&mut self) -> ! {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let mut web_socket = WebSockets::new(|event: WebsocketEvent| {
            let _ = tx.send(Self::handle_event(event));

            Ok(())
        });

        let keep_running = AtomicBool::new(true);
        loop {
            let endpoint = self.endpoint().await;

            match web_socket.connect(&endpoint).await {
                Ok(()) => loop {
                    let input_data = self.connect();
                    let connect_msg = self.message(WsEvent::Connected(input_data));
                    let _ = self.output().send(connect_msg).await;

                    futures::select! {
                        ws_closed = web_socket.event_loop(&keep_running).fuse() => {
                            if let Err(e) = ws_closed{
                                eprintln!("User Stream error: {e:?}");
                            }
                        }
                        event = rx.recv().fuse() => {
                            let event = event.expect("nobody should be closing channel");
                            let message = self.message(WsEvent::Message(event));
                            let _ = self.output().send(message).await;
                        }
                    }
                },
                Err(e) => {
                    eprintln!("WebSocket connection error: {e:?}");
                }
            }
            let message = self.message(WsEvent::Disconnected);
            let _ = self.output().send(message).await;
        }
    }
}
