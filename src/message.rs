use std::fmt::Debug;
use std::fmt::Display;
use std::time::Instant;

use crate::{
    config::Config,
    views::{dashboard::DashboardMessage, settings::SettingsMessage},
    ws::WsMessage,
};

use binance::rest_model::KlineSummaries;
use binance::rest_model::{Balance, Order};

/// Converts Result Err variant into string, stores error source
///
/// Part of event system. Used to handle throwaway results
#[derive(Debug, Clone)]
pub(crate) enum MaybeError {
    NoError { source: String },
    Error { source: String, message: String },
}

impl MaybeError {
    pub(crate) fn new(source: String) -> Self {
        Self::NoError { source }
    }

    fn into_source(self) -> String {
        match self {
            MaybeError::NoError { source } => source,
            MaybeError::Error { source, .. } => source,
        }
    }

    /// Throws away Ok and stores Err as `Display` string
    pub(crate) fn maybe<T, E>(self, value: &Result<T, E>) -> Self
    where
        E: Display,
    {
        if let Err(err) = value {
            Self::Error {
                source: self.into_source(),
                message: err.to_string(),
            }
        } else {
            self
        }
    }
}

impl Display for MaybeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaybeError::NoError { source } => write!(f, "{source}"),
            MaybeError::Error { source, message } => write!(f, "{source}: {message}"),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    /// Go to or from settings depending on where you are.
    ///
    /// You cannot close settings while config is invalid
    SettingsToggled,

    /// Manually triggered at interval
    Tick,

    /// Error source and message
    DispatchErr((String, String)),

    /// Config update happened
    ConfigUpdated(Result<Config, String>),

    // API keys update happened
    CredentialsUpdated,

    /// API responses
    OrdersRecieved(Vec<Order>),
    BalancesRecieved(Vec<Balance>),
    KlinesRecieved(KlineSummaries),
    MarketChanged(String),

    /// Settings view events
    Settings(SettingsMessage),

    /// Dashboard view events
    Dashboard(DashboardMessage),

    /// Event from one of websockets
    Ws(WsMessage),

    /// Does nothing
    NoOp,

    // Loader widget tick
    LoaderTick(Instant),
}

impl From<MaybeError> for Message {
    fn from(value: MaybeError) -> Self {
        match value {
            MaybeError::Error { source, message } => Self::DispatchErr((source, message)),
            MaybeError::NoError { .. } => Self::NoOp,
        }
    }
}

impl From<WsMessage> for Message {
    fn from(value: WsMessage) -> Self {
        Self::Ws(value)
    }
}

impl From<SettingsMessage> for Message {
    fn from(value: SettingsMessage) -> Self {
        Self::Settings(value)
    }
}

impl From<DashboardMessage> for Message {
    fn from(value: DashboardMessage) -> Self {
        Self::Dashboard(value)
    }
}
