use iced::{
    widget::{
        button::{danger, secondary},
        pane_grid::{self, Configuration},
        responsive, row, text, PaneGrid,
    },
    Command, Element, Font, Length,
};

use crate::{
    api::Client, config::Config, data::AppData, message::Message, theme::h2c, ws::Websockets,
};

use super::panes::{
    balances::BalancesPane,
    book::BookPane,
    calculator::{CalculatorPane, CalculatorPaneMessage},
    chart::ChartPane,
    market::{Market, MarketPanelMessage},
    orders::OrdersPane,
    style,
    trades::TradesPane,
    watchlist::{WatchlistMessage, WatchlistPane},
};

#[derive(PartialEq)]
pub(crate) enum PaneType {
    Prices,
    Book,
    Trades,
    Market,
    Balances,
    Orders,
    Calculator,
    Chart,
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

pub(crate) struct Pane {
    id: PaneType,
    is_pinned: bool,
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
            iced::widget::button(content.size(12).color(h2c("FFFFFF").unwrap()))
                .height(14)
                .width(14)
                .style(secondary)
                .on_press(message)
        };

        row = row.push(toggle);
    }

    let mut close = iced::widget::button(
        text('\u{F62A}')
            .size(12)
            .font(Font::with_name("bootstrap-icons")),
    )
    .height(14)
    .width(14)
    .style(danger);

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

    Watchlist(WatchlistMessage),
    Market(MarketPanelMessage),
    Calculator(CalculatorPaneMessage),

    CurrencyPairSelected(String),

    // TODO: move to chart
    TimeframeChanged(String),
}

impl From<WatchlistMessage> for DashboardMessage {
    fn from(value: WatchlistMessage) -> Self {
        Self::Watchlist(value)
    }
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
    watchlist: WatchlistPane,
    chart: ChartPane,
    calculator: CalculatorPane,
    market: Market,
    book: BookPane,
    orders: OrdersPane,
    balances: BalancesPane,
    trades: TradesPane,
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
            watchlist: WatchlistPane::new(),
            chart: ChartPane::new(),
            calculator: CalculatorPane::new(),
            market: Market::new(),
            book: BookPane::new(),
            orders: OrdersPane::new(),
            balances: BalancesPane::new(),
            trades: TradesPane::new(),
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
            DashboardMessage::CurrencyPairSelected(pair) => {
                ws.track_new_currency_pair(&pair);
                self.market.set_currency_pair(pair);

                Command::none()
            }
            DashboardMessage::Watchlist(msg) => self
                .watchlist
                .update(msg, data, config)
                .map(DashboardMessage::from)
                .map(Message::from),
            DashboardMessage::Calculator(msg) => self
                .calculator
                .update(msg)
                .map(DashboardMessage::from)
                .map(Message::from),
            DashboardMessage::TimeframeChanged(tf) => api.klines(self.pair().to_owned(), tf),
            DashboardMessage::Market(msg) => self.market.update(msg, api, data, ws),
        }
    }

    pub(crate) fn tick(&mut self, data: &AppData) {
        self.calculator.tick(data);
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
                PaneType::Prices => self.watchlist.view(data).map(DashboardMessage::from),
                PaneType::Chart => self.chart.view(data),
                PaneType::Book => self.book.view(data),
                PaneType::Trades => self.trades.view(data),
                PaneType::Market => self.market.view().map(DashboardMessage::from),
                PaneType::Balances => self.balances.view(data),
                PaneType::Orders => self.orders.view(data),
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
}
