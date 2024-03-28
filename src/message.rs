use std::time;

use crate::{
    config::Config,
    views::panes::{
        calculator::CalculatorMessage, settings::SettingsMessage, watchlist::WatchlistFilter,
    },
    ws::{book, prices, trades, user},
};

use binance::rest_model::{Balance, Order};
use iced::widget::pane_grid;

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Screen {
    Dashboard,
    Settings,
}

#[derive(Debug, Clone)]
pub(crate) enum UI {
    GoToDashboard,
    GoToSettings,
    // there is a button that is shared between dashboard and settings
    ToggleSettings,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    UI(UI),
    DispatchErr(String),
    // fetch personal data with api key
    FetchData,
    // config update happened
    ConfigUpdated(Result<Config, ()>),
    Split(pane_grid::Axis, pane_grid::Pane),
    SplitFocused(pane_grid::Axis),
    FocusAdjacent(pane_grid::Direction),
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    Maximize(pane_grid::Pane),
    Restore,
    Close(pane_grid::Pane),
    CloseFocused,
    MarketQuoteChanged(String),
    MarketAmtChanged(String),
    MarketPairChanged(String),
    WatchlistFilterInput(String),
    MarketPairSet,
    MarketPairSet2(()),
    MarketPairUnset(()),
    MarketPrice(prices::Event),
    BuyPressed,
    SellPressed,
    PriceInc(f64),
    QtySet(f64),
    CustomPoller(time::Instant),
    PriceEcho(prices::Event),
    TradeEcho(trades::Event),
    BookEcho(book::BookEvent),
    UserEcho(user::WsUpdate),
    OrdersRecieved(Vec<Order>),
    MarketChanged(String),
    AssetSelected(String),
    BalancesRecieved(Vec<Balance>),
    ApplyWatchlistFilter(WatchlistFilter),
    FontsLoaded(Result<(), iced::font::Error>),
    Calculator(CalculatorMessage),
    Settings(SettingsMessage),
}
