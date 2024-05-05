use std::future::Future;

use iced::Executor;
use iced_futures::futures;

pub(crate) struct DynastyExecutor(tokio::runtime::Runtime);

/// Customized iced_futures tokio executor
impl Executor for DynastyExecutor {
    fn new() -> Result<Self, futures::io::Error> {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("dynasty-tokio")
            .build()
            .map(Self)
    }

    #[allow(clippy::let_underscore_future)]
    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        let _ = tokio::runtime::Runtime::spawn(&self.0, future);
    }

    fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        let _guard = tokio::runtime::Runtime::enter(&self.0);
        f()
    }
}
