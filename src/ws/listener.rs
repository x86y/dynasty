use std::{error::Error, sync::atomic::AtomicBool, time::Duration};

use binance::websockets::WebSockets;
use iced_futures::futures::{channel::mpsc as mpsc_futures, SinkExt};
use serde::de::DeserializeOwned;
use tokio::sync::mpsc as mpsc_tokio;
use tracing::info;

use super::{WsEvent, WsHandle, WsMessage};

pub(crate) trait WsListener {
    type Event: Send + DeserializeOwned;
    type Input;
    type Output;

    /// Websocket recv timeout
    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

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
        let (tx, mut rx) = mpsc_tokio::unbounded_channel();

        let mut web_socket = WebSockets::new(|event| {
            tx.send(event)
                .map_err(|e| binance::errors::Error::Msg(e.to_string()))
        });

        let (input_tx, mut input_rx) = mpsc_tokio::unbounded_channel();

        let connected = self.message(WsEvent::Created(WsHandle(input_tx)));
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
                    tracing::error!("endpoint error: {e}");

                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };

            if let Err(e) = web_socket.connect(&endpoint).await {
                tracing::error!("{} connection error: {}", &endpoint, e);

                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }

            info!("connected {}", &endpoint);
            let connected = self.message(WsEvent::Connected);
            let _ = output.send(connected).await;

            let timeout = self.timeout();

            loop {
                tokio::select! {
                    biased;

                    ws_closed = web_socket.event_loop(&keep_running) => {
                        if let Err(e) = ws_closed {
                            tracing::error!("stream error: {e}");
                        }
                        break;
                    }
                    input = input_rx.recv() => {
                        self.handle_input(input.expect("channel closed"), &mut keep_running);
                    }
                    maybe_event = tokio::time::timeout(timeout, rx.recv()) => {
                        let event = match maybe_event {
                            Ok(event) => event,
                            Err(_) => {
                                tracing::error!("{} hung, disconnecting", &endpoint);

                                break;
                            }
                        };
                        let handled = self.handle_event(event.expect("channel closed"));
                        let message = self.message(WsEvent::Message(handled));
                        let _ = output.send(message).await;
                    }
                }
            }

            info!("disconnected {}", &endpoint);
            let disconnected = self.message(WsEvent::Disconnected);
            let _ = output.send(disconnected).await;
        }
    }
}
