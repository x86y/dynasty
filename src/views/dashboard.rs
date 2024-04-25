use std::time::Instant;

use iced::{
    theme,
    widget::{
        button,
        pane_grid::{self, Configuration},
        responsive, row, text, PaneGrid,
    },
    window, Command, Element, Font, Length,
};
use iced_futures::Subscription;
use ringbuf::Rb;

use crate::{
    api::Client,
    config::Config,
    data::{AppData, PriceFilter},
    message::Message,
    theme::h2c,
    ws::{prices::AssetDetails, Websockets},
};

use super::{
    components::loading::Loader,
    panes::{
        balances::balances_view,
        book::book_view,
        calculator::{CalculatorPane, CalculatorPaneMessage},
        chart::ChartPane,
        market::{Market, MarketPanelMessage},
        orders::orders_view,
        style,
        trades::trades_view,
        watchlist::{watchlist_view, WatchlistFilter},
    },
};

#[derive(PartialEq)]
pub enum PaneType {
    Prices,
    Book,
    Trades,
    Market,
    Balances,
    Orders,
    Calculator,
    Chart,
}

impl From<usize> for PaneType {
    fn from(_value: usize) -> Self {
        Self::Balances
    }
}

impl ToString for PaneType {
    fn to_string(&self) -> String {
        match self {
            PaneType::Prices => "Watchlist",
            PaneType::Book => "Book",
            PaneType::Trades => "Trades",
            PaneType::Market => "Market",
            PaneType::Balances => "Balances",
            PaneType::Orders => "Orders",
            PaneType::Calculator => "Calculator",
            PaneType::Chart => "Chart",
        }
        .to_string()
    }
}

pub struct Pane {
    pub id: PaneType,
    pub is_pinned: bool,
}

impl Pane {
    pub fn new(ty: PaneType) -> Self {
        Self {
            id: ty,
            is_pinned: false,
        }
    }
}

pub fn view_controls<'a>(
    pane: pane_grid::Pane,
    total_panes: usize,
    is_pinned: bool,
    is_maximized: bool,
) -> Element<'a, DashboardMessage> {
    let mut row = row![].spacing(5);

    if total_panes > 1 {
        let toggle = {
            let (content, message) = if is_maximized {
                (
                    text('\u{F3DE}').font(Font::with_name("bootstrap-icons")),
                    DashboardMessage::Restore,
                )
            } else {
                (
                    text('\u{F3DF}').font(Font::with_name("bootstrap-icons")),
                    DashboardMessage::Maximize(pane),
                )
            };
            button(content.size(12).style(h2c("FFFFFF").unwrap()))
                .height(14)
                .width(14)
                .style(theme::Button::Secondary)
                .on_press(message)
        };

        row = row.push(toggle);
    }

    let mut close = button(
        text('\u{F62A}')
            .size(12)
            .font(Font::with_name("bootstrap-icons")),
    )
    .height(14)
    .width(14)
    .style(theme::Button::Destructive);

    if total_panes > 1 && !is_pinned {
        close = close.on_press(DashboardMessage::Close(pane));
    }

    row.push(close).into()
}

#[derive(Debug, Clone)]
pub(crate) enum DashboardMessage {
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    Maximize(pane_grid::Pane),
    Restore,
    Close(pane_grid::Pane),
    WatchlistFilterInput(String),
    ApplyWatchlistFilter((WatchlistFilter, bool)),
    AssetSelected(String),
    Market(MarketPanelMessage),
    Calculator(CalculatorPaneMessage),
    TimeframeChanged(String),

    // Loader widget tick
    LoaderTick(Instant),
}

impl From<CalculatorPaneMessage> for DashboardMessage {
    fn from(value: CalculatorPaneMessage) -> Self {
        Self::Calculator(value)
    }
}

impl From<MarketPanelMessage> for DashboardMessage {
    fn from(value: MarketPanelMessage) -> Self {
        Self::Market(value)
    }
}

pub(crate) struct DashboardView {
    focus: Option<pane_grid::Pane>,
    panes: pane_grid::State<Pane>,
    // TODO: filter
    filter: WatchlistFilter,
    filter_string: String,
    // panes
    chart: ChartPane,
    calculator: CalculatorPane,
    market: Market,
    loader: Loader,
}

macro_rules! v {
    ($r: expr, $a: expr, $b: expr) => {
        b![Vertical, $r, $a, $b]
    };
}
macro_rules! h {
    ($r: expr, $a: expr, $b: expr) => {
        b![Horizontal, $r, $a, $b]
    };
}
macro_rules! b {
    ($d: ident, $r: expr, $a: expr, $b: expr) => {
        Configuration::Split {
            axis: pane_grid::Axis::$d,
            ratio: $r,
            a: Box::new($a),
            b: Box::new($b),
        }
    };
}
macro_rules! pane {
    ($p: ident) => {
        Configuration::Pane(Pane::new(PaneType::$p))
    };
}

impl DashboardView {
    pub(crate) fn new() -> Self {
        let panes = pane_grid::State::with_configuration(h![
            0.65,
            v![
                0.15,
                h![0.6, pane![Prices], pane![Balances]],
                v![
                    0.5,
                    pane![Chart],
                    v![0.6, h![0.33, pane![Market], pane![Trades]], pane![Book]]
                ]
            ],
            v![0.7, pane![Orders], pane![Calculator]]
        ]);

        Self {
            focus: None,
            panes,
            chart: ChartPane::new(),
            calculator: CalculatorPane::new(),
            filter: WatchlistFilter::Favorites,
            filter_string: String::new(),
            market: Market::new(),
            loader: Loader::new(),
        }
    }

    /// currently entered pair of currencies
    pub(crate) fn pair(&self) -> &str {
        self.market.pair()
    }

    pub(crate) fn update(
        &mut self,
        message: DashboardMessage,
        api: &Client,
        data: &mut AppData,
        ws: &Websockets,
        config: &Config,
    ) -> Command<Message> {
        match message {
            DashboardMessage::Clicked(pane) => {
                self.focus = Some(pane);
                Command::none()
            }
            DashboardMessage::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);
                Command::none()
            }
            DashboardMessage::Dragged(pane_grid::DragEvent::Dropped { pane, target }) => {
                self.panes.drop(pane, target);
                Command::none()
            }
            DashboardMessage::Dragged(_) => Command::none(),
            DashboardMessage::Maximize(pane) => {
                self.panes.maximize(pane);
                Command::none()
            }
            DashboardMessage::Restore => {
                self.panes.restore();
                Command::none()
            }
            DashboardMessage::Close(pane) => {
                if let Some((_, sibling)) = self.panes.close(pane) {
                    self.focus = Some(sibling);
                }
                Command::none()
            }
            DashboardMessage::ApplyWatchlistFilter((f, clicked_again)) => {
                if clicked_again {
                    data.prices.flip_sort();
                } else {
                    let filter = match f {
                        WatchlistFilter::Favorites => {
                            PriceFilter::Matches(config.watchlist_favorites.clone())
                        }
                        WatchlistFilter::Eth => PriceFilter::Contains("ETH".to_owned()),
                        WatchlistFilter::Btc => PriceFilter::Contains("BTC".to_owned()),
                        WatchlistFilter::Alts => PriceFilter::All,
                    };

                    data.prices.set_filter(filter);
                    self.filter = f;
                }

                Command::none()
            }
            DashboardMessage::WatchlistFilterInput(s) => {
                self.filter_string = s.to_uppercase();
                data.prices
                    .set_filter(PriceFilter::Contains(self.filter_string.clone()));

                Command::none()
            }
            DashboardMessage::AssetSelected(pair) => {
                ws.track_new_currency_pair(&pair);
                self.market.pair_selected(pair);

                Command::none()
            }
            DashboardMessage::Calculator(msg) => self
                .calculator
                .update(msg)
                .map(DashboardMessage::from)
                .map(Message::from),
            DashboardMessage::TimeframeChanged(tf) => api.klines(self.pair().to_owned(), tf),
            DashboardMessage::Market(msg) => self.market.update(msg, api, data, ws),
            DashboardMessage::LoaderTick(instant) => {
                self.loader.update(instant);
                Command::none()
            }
        }
    }

    pub(crate) fn tick(&mut self, data: &AppData) {
        self.calculator.tick(data);
    }

    pub(crate) fn prepend_chart_data<T>(&mut self, slc: T) -> Command<DashboardMessage>
    where
        T: Iterator<Item = f64>,
    {
        self.chart.data.clear();
        self.chart.data.push_iter_overwrite(slc);
        Command::none()
    }

    pub(crate) fn chart_pair_price(&mut self, asset: &AssetDetails) {
        if asset.name == self.market.pair() {
            self.chart.update_data(f64::from(asset.price));
        }
    }

    pub(crate) fn view<'a>(&'a self, data: &'a AppData) -> Element<'a, DashboardMessage> {
        let focus = self.focus;
        let total_panes = self.panes.len();

        PaneGrid::new(&self.panes, |id, pane, is_maximized| {
            let is_focused = focus == Some(id);

            let title = row![text(pane.id.to_string())].spacing(5);
            let title_bar = pane_grid::TitleBar::new(title)
                .controls(view_controls(id, total_panes, pane.is_pinned, is_maximized))
                .padding([8, 12]);

            pane_grid::Content::new(responsive(|_size| match pane.id {
                PaneType::Prices => {
                    watchlist_view(data, self.filter, &self.filter_string, &self.loader)
                }
                PaneType::Chart => self.chart.view(&self.loader),
                PaneType::Book => book_view(data, &self.loader),
                PaneType::Trades => trades_view(data, &self.loader),
                PaneType::Market => self.market.view().map(DashboardMessage::from),
                PaneType::Balances => balances_view(data, &self.loader),
                PaneType::Orders => orders_view(data, &self.loader),
                PaneType::Calculator => self.calculator.view().map(DashboardMessage::from),
            }))
            .title_bar(title_bar)
            .style(if is_focused {
                style::pane_focused
            } else {
                style::pane_active
            })
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(10)
        .on_click(DashboardMessage::Clicked)
        .on_drag(DashboardMessage::Dragged)
        .on_resize(10, DashboardMessage::Resized)
        .into()
    }

    pub(crate) fn subscription(&self, data: &AppData) -> Subscription<Message> {
        if data.is_loading() {
            window::frames()
                .map(DashboardMessage::LoaderTick)
                .map(Message::from)
        } else {
            Subscription::none()
        }
    }
}
