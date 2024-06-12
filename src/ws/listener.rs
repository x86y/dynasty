use std::{
    error::Error,
    time::{Duration, Instant},
};

use binance::websockets::WebSockets;
use iced_futures::futures::{channel::mpsc as mpsc_futures, SinkExt, StreamExt};
use tokio::sync::mpsc as mpsc_tokio;
use tokio::time::timeout;
use tracing::info;

use super::{WsEvent, WsHandle, WsMessage};

const PING_INTERVAL: u64 = 5;

async fn event_loop<WE>(
    ws: WebSockets<'_, WE>,
    output: &mut mpsc_futures::Sender<WE>,
) -> binance::errors::Result<()>
where
    WE: serde::de::DeserializeOwned,
{
    use binance::errors::Error;
    use tungstenite::Message;

    let (socket, _) = ws.socket.expect("socket not connected");
    let (mut tx, mut rx) = socket.split();

    let mut interval = tokio::time::interval(Duration::from_secs(PING_INTERVAL));
    let mut last_pong = Instant::now();

    loop {
        tokio::select! {
            msg = rx.next() => {
                let Some(msg) = msg.transpose()? else {
                    return Ok(());
                };

                match msg {
                    Message::Text(msg) => {
                        let event = serde_json::from_str(msg.as_str())?;

                        output.send(event).await.map_err(|e| Error::Msg(e.to_string()))?;
                    }
                    Message::Ping(data) => {
                        let _ = tx.send(Message::Pong(data)).await;
                    }
                    Message::Pong(_) => last_pong = Instant::now(),
                    Message::Close(e) => {
                        return Err(Error::Msg(format!("Disconnected: {e:?}")));
                    }
                    Message::Binary(_) | Message::Frame(_) => {}
                }
            }
            _ = interval.tick() => {
                if last_pong.elapsed().as_secs() > PING_INTERVAL * 2 {
                    return Err(Error::Msg(
                        format!("Did not receive ping in the last {} seconds)", PING_INTERVAL * 2)
                    ));
                }

                let msg = Message::Ping(Vec::new());
                // NOTE: unsure if timeout is needed here. This does not time out on internet
                //       disconnect
                timeout(Duration::from_secs(PING_INTERVAL), tx.send(msg))
                    .await
                    .map_err(|e| Error::Msg(format!("Ping send timed out: {e}")))?
                    .map_err(|e| Error::Msg(format!("Ping send failed: {e}")))?;
            }
        }
    }
}

pub(crate) trait WsListener {
    type Event: serde::de::DeserializeOwned;
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
    /// Return value indicates wether ws should stay connected
    fn handle_input(&mut self, _: Self::Input) -> bool {
        true
    }

    /// Main entrypoint
    async fn run(&mut self, mut output: mpsc_futures::Sender<WsMessage>) -> ! {
        // forward messages out of websocket callback
        let (mut tx, mut rx) = mpsc_futures::channel(100);

        let (input_tx, mut input_rx) = mpsc_tokio::unbounded_channel();

        let created = self.message(WsEvent::Created(WsHandle(input_tx)));
        let _ = output.send(created).await;

        loop {
            // input might alter endpoint result
            if let Ok(input) = input_rx.try_recv() {
                self.handle_input(input);
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

            let mut web_socket = WebSockets::new(|_| Ok(()));
            if let Err(e) = web_socket.connect(&endpoint).await {
                tracing::error!("connection error: {e}");

                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }

            info!("connected {}", &endpoint);
            let connected = self.message(WsEvent::Connected);
            let _ = output.send(connected).await;

            let ws_loop = event_loop(web_socket, &mut tx);
            tokio::pin!(ws_loop);

            loop {
                tokio::select! {
                    biased;

                    ws_closed = &mut ws_loop => {
                        if let Err(e) = ws_closed {
                            tracing::error!("stream error: {e}");
                        }
                        break;
                    }
                    input = input_rx.recv() => {
                        if !self.handle_input(input.expect("channel closed")) {
                            break;
                        }
                    }
                    event = rx.select_next_some() => {
                        let handled = self.handle_event(event);
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
