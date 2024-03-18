pub mod m {
    macro_rules! con {
        ($event:ty, $stream:expr, $connect_expr:expr, $process_msg:expr) => {
            use futures::sink::SinkExt;
            use futures::FutureExt;
            use iced::subscription::{self, Subscription};
            use iced_futures::futures;
            use std::sync::atomic::AtomicBool;
            use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

            pub fn connect() -> Subscription<$event> {
                struct Connect;

                subscription::channel(
                    std::any::TypeId::of::<Connect>(),
                    100,
                    |mut output| async move {
                        let keep_running = AtomicBool::new(true);
                        let (s, mut r): (UnboundedSender<$event>, UnboundedReceiver<$event>) =
                            unbounded_channel();

                        let mut web_socket_connection = $connect_expr();

                        loop {
                            web_socket_connection.connect(book_ticker).await.unwrap();

                            loop {
                                futures::select! {
                                    _recv = web_socket.event_loop(&keep_running).fuse() => continue,
                                    recv2 = r.recv().fuse() => {
                                        if let Some(event) = recv2 {
                                            output
                                                .send($process_msg(event))
                                                .await
                                                .unwrap(); // Handle error properly in production code
                                        };
                                    }
                                };
                            }
                        }
                    },
                )
            }
        };
    }
}
