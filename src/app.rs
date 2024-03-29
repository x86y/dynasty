use crate::api;
use crate::config::Config;
use crate::message::Message;
use crate::message::Screen;
use crate::views::panes::balances::balances_view;
use crate::views::panes::book::book_view;
use crate::views::panes::calculator::CalculatorMessage;
use crate::views::panes::calculator::CalculatorPane;
use crate::views::panes::chart::ChartPane;
use crate::views::panes::market::market_view;
use crate::views::panes::orders::orders_view;
use crate::views::panes::settings::SettingsPane;
use crate::views::panes::style;
use crate::views::panes::trades::trades_view;
use crate::views::panes::view_controls;
use crate::views::panes::watchlist::watchlist_view;
use crate::views::panes::watchlist::WatchlistFilter;
use crate::views::panes::Pane;
use crate::views::panes::PaneType;
use crate::views::panes::PANE_ID_COLOR_FOCUSED;
use crate::views::panes::PANE_ID_COLOR_UNFOCUSED;
use crate::ws::book;
use crate::ws::prices;
use crate::ws::trades;
use crate::ws::user;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::env;
use std::time::Duration;

use binance::rest_model::OrderStatus;
use binance::rest_model::{Balance, Order};
use binance::ws_model::TradesEvent;
use iced::executor;
use iced::font;
use iced::widget::button;
use iced::widget::pane_grid::{self, PaneGrid};
use iced::widget::responsive;
use iced::widget::svg;
use iced::widget::Row;
use iced::widget::Space;
use iced::widget::{column, container, row, text};
use iced::{Application, Color, Command, Element, Length, Subscription, Theme};
use pane_grid::Configuration;

pub(crate) struct App {
    panes: pane_grid::State<Pane>,
    panes_created: usize,
    focus: Option<pane_grid::Pane>,
    watchlist_favorites: Vec<String>,
    filter: WatchlistFilter,
    filter_string: String,
    new_price: String,
    new_amt: String,
    new_pair: String,
    pair_submitted: bool,
    pub(crate) data: AppData,
    current_screen: Screen,
    config: Config,
    errors: Vec<String>,
    chart: ChartPane,
    calculator: CalculatorPane,
    settings: SettingsPane,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct AppData {
    pub(crate) prices: HashMap<String, f32>,
    book: (String, BTreeMap<String, f64>, BTreeMap<String, f64>),
    trades: VecDeque<TradesEvent>,
    balances: Vec<Balance>,
    pub(crate) orders: Vec<Order>,
    pub(crate) chart_data: Vec<f64>,
    quote: String,
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

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Flags = Config;
    type Executor = executor::Default;

    fn new(config: Self::Flags) -> (Self, Command<Message>) {
        let panes = pane_grid::State::with_configuration(h![
            0.7,
            v![
                0.25,
                h![0.6, pane![Prices], pane![Chart]],
                v![
                    0.5,
                    h![
                        0.5,
                        pane![Market],
                        v![0.5, pane![Trades], pane![Calculator]]
                    ],
                    v![0.6, pane![Book], pane![Balances]]
                ]
            ],
            pane![Orders]
        ]);

        (
            App {
                panes,
                panes_created: 1,
                focus: None,
                filter: WatchlistFilter::Favorites,
                filter_string: "".to_string(),
                watchlist_favorites: [
                    "BTCUSDT", "ETHUSDT", "LINKUSDT", "UNIUSDT", "ARBUSDT", "SYNUSDT", "OPUSDT",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                new_price: Default::default(),
                new_amt: Default::default(),
                new_pair: "BTCUSDT".into(),
                pair_submitted: true,
                data: Default::default(),
                current_screen: Screen::Dashboard,
                config: config.clone(),
                errors: vec![],
                calculator: CalculatorPane::new(),
                settings: SettingsPane::new(config),
                chart: ChartPane::new(),
            },
            Command::batch([
                Command::perform(
                    async {
                        #[cfg(feature = "k")]
                        use ngnk::kinit;
                        #[cfg(feature = "k")]
                        kinit();
                    },
                    |_| Message::FetchData,
                ),
                font::load(
                    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/icons.ttf"))
                        .as_slice(),
                )
                .map(Message::FontsLoaded),
            ]),
        )
    }

    fn title(&self) -> String {
        String::from("Dynasty")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick => Command::perform(async {}, |_| CalculatorMessage::Tick.into()),
            Message::FetchData => Command::batch([
                Command::perform(
                    api::orders_history(
                        self.config.api_key.clone(),
                        self.config.api_secret_key.clone(),
                    ),
                    Message::OrdersRecieved,
                ),
                Command::perform(
                    api::balances(
                        self.config.api_key.clone(),
                        self.config.api_secret_key.clone(),
                    ),
                    Message::BalancesRecieved,
                ),
            ]),
            Message::ConfigUpdated(c) => {
                self.config = c.expect("TODO: bad config popup/message");

                Command::perform(async {}, |_| Message::FetchData)
            }
            Message::Split(axis, pane) => {
                let result = self
                    .panes
                    .split(axis, pane, Pane::new(self.panes_created.into()));

                if let Some((pane, _)) = result {
                    self.focus = Some(pane);
                }

                self.panes_created += 1;
                Command::none()
            }
            Message::SplitFocused(axis) => {
                if let Some(pane) = self.focus {
                    let result = self
                        .panes
                        .split(axis, pane, Pane::new(self.panes_created.into()));

                    if let Some((pane, _)) = result {
                        self.focus = Some(pane);
                    }

                    self.panes_created += 1;
                }
                Command::none()
            }
            Message::FocusAdjacent(direction) => {
                if let Some(pane) = self.focus {
                    if let Some(adjacent) = self.panes.adjacent(pane, direction) {
                        self.focus = Some(adjacent);
                    }
                }
                Command::none()
            }
            Message::Clicked(pane) => {
                self.focus = Some(pane);
                Command::none()
            }
            Message::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);
                Command::none()
            }
            Message::Dragged(pane_grid::DragEvent::Dropped { pane, target }) => {
                self.panes.drop(pane, target);
                Command::none()
            }
            Message::Dragged(_) => Command::none(),
            Message::Maximize(pane) => {
                self.panes.maximize(pane);
                Command::none()
            }
            Message::Restore => {
                self.panes.restore();
                Command::none()
            }
            Message::Close(pane) => {
                if let Some((_, sibling)) = self.panes.close(pane) {
                    self.focus = Some(sibling);
                }
                Command::none()
            }
            Message::CloseFocused => {
                if let Some(pane) = self.focus {
                    if let Some(Pane { is_pinned, .. }) = self.panes.get(pane) {
                        if !is_pinned {
                            if let Some((_, sibling)) = self.panes.close(pane) {
                                self.focus = Some(sibling);
                            }
                        }
                    }
                }
                Command::none()
            }
            Message::FontsLoaded(_) => Command::none(),
            Message::MarketPrice(p) => {
                println!("incame price {p:?}");
                Command::none()
            }
            Message::MarketPairSet => {
                self.pair_submitted = true;
                Command::none()
            }
            Message::MarketPairSet2(()) => {
                self.pair_submitted = true;
                Command::none()
            }
            Message::MarketPairUnset(_) => {
                self.pair_submitted = false;
                Command::perform(async {}, Message::MarketPairSet2)
            }
            Message::BuyPressed => Command::perform(
                api::trade_spot(
                    self.config.api_key.clone(),
                    self.config.api_secret_key.clone(),
                    self.new_pair.clone(),
                    self.new_price.clone().parse().unwrap(),
                    self.new_amt.parse().unwrap(),
                    binance::rest_model::OrderSide::Buy,
                ),
                |m| {
                    println!("{m:?}");
                    Message::MarketChanged("REEEEE".to_string())
                },
            ),
            Message::SellPressed => Command::perform(
                api::trade_spot(
                    self.config.api_key.clone(),
                    self.config.api_secret_key.clone(),
                    self.new_pair.clone(),
                    self.new_price.clone().parse().unwrap(),
                    self.new_amt.parse().unwrap(),
                    binance::rest_model::OrderSide::Sell,
                ),
                |m| {
                    println!("{m:?}");
                    Message::MarketChanged("REEEEE".to_string())
                },
            ),
            Message::MarketChanged(new_market) => {
                self.data.quote = new_market;
                Command::none()
            }
            Message::MarketPairChanged(np) => {
                self.new_pair = np;
                Command::none()
            }
            Message::MarketQuoteChanged(nm) => {
                self.new_price = nm;
                Command::none()
            }
            Message::MarketAmtChanged(nm) => {
                self.new_amt = nm;
                Command::none()
            }
            Message::PriceEcho(msg) => {
                match msg {
                    prices::Event::MessageReceived(m) => {
                        self.data.prices.insert(m.name.clone(), m.price);
                    }
                };
                if m.name == self.new_pair.into() {
                    Command::perform(async {}, self.chart.update(m.price))
                } else {
                    Command::none()
                }
            }
            Message::BookEcho(msg) => {
                match msg {
                    book::BookEvent::MessageReceived(bt) => {
                        self.data.book = (bt.sym, bt.bids, bt.asks);
                    }
                };
                Command::none()
            }
            Message::TradeEcho(t) => {
                match t {
                    trades::Event::MessageReceived(te) => {
                        if self.data.trades.len() >= 1000 {
                            self.data.trades.pop_back();
                        }
                        self.data.trades.push_front(te);
                    }
                }
                Command::none()
            }
            Message::OrdersRecieved(orders) => {
                self.data.orders = orders;
                Command::none()
            }
            Message::BalancesRecieved(bals) => {
                self.data.balances = bals;
                Command::none()
            }
            Message::UserEcho(f) => {
                let user::WsUpdate::UpdateReceived(u) = f;
                match u {
                    binance::ws_model::WebsocketEvent::AccountPositionUpdate(p) => {
                        for b in p.balances.iter() {
                            let ib = self.data.balances.iter_mut().find(|a| a.asset == b.asset);
                            if let Some(uib) = ib {
                                *uib = unsafe { std::mem::transmute(b.clone()) }
                            }
                        }
                    }
                    binance::ws_model::WebsocketEvent::OrderUpdate(o) => {
                        let existing_order = self.data.orders.iter_mut().find(|order| {
                            // order.client_order_id == o.order_id&&
                            order.symbol == o.symbol
                                && order.side == o.side
                                && order.status == OrderStatus::PartiallyFilled
                        });

                        if let Some(order) = existing_order {
                            // Update the existing order with the new values
                            order.executed_qty += o.qty_last_executed;
                            order.cummulative_quote_qty += o.qty;
                            order.update_time = o.trade_order_time;
                        } else {
                            self.data.orders.insert(
                                0,
                                Order {
                                    symbol: o.symbol,
                                    order_id: o.order_id,
                                    order_list_id: o.order_list_id as i32,
                                    client_order_id: o.client_order_id.unwrap(),
                                    price: o.price,
                                    orig_qty: o.qty,
                                    executed_qty: o.qty_last_executed,
                                    cummulative_quote_qty: o.qty,
                                    status: o.current_order_status,
                                    time_in_force: o.time_in_force,
                                    order_type: o.order_type,
                                    side: o.side,
                                    stop_price: o.stop_price,
                                    iceberg_qty: o.iceberg_qty,
                                    time: o.event_time,
                                    update_time: o.trade_order_time,
                                    is_working: false,
                                    orig_quote_order_qty: o.qty,
                                },
                            );
                        }
                    }
                    binance::ws_model::WebsocketEvent::BalanceUpdate(_p) => {
                        // not needed imo?
                    }
                    binance::ws_model::WebsocketEvent::ListOrderUpdate(_lo) => {
                        // not needed imo?
                    }
                    _ => unreachable!(),
                };
                Command::none()
            }
            Message::AssetSelected(a) => {
                if !a.ends_with("USDT") && !a.ends_with("BTC") && !a.ends_with("ETH") {
                    self.new_pair = format!("{a}USDT");
                } else {
                    self.new_pair = a;
                }
                Command::perform(async {}, Message::MarketPairUnset)
            }
            Message::QtySet(f) => {
                let usdt_b = self
                    .data
                    .balances
                    .iter()
                    .find(|b| b.asset == "USDT")
                    .unwrap()
                    .free;
                self.new_amt = (usdt_b * f).to_string();
                Command::none()
            }
            Message::PriceInc(inc) => {
                let price = self.data.prices.get(&self.new_pair).unwrap();
                self.new_price =
                    (((*price as f64 * (1.0 + (inc / 100.0))) * 100.0).round() / 100.0).to_string();
                Command::none()
            }
            Message::ApplyWatchlistFilter(f) => {
                self.filter = f;
                Command::none()
            }
            Message::WatchlistFilterInput(wfi) => {
                self.filter_string = wfi;
                Command::none()
            }
            Message::DispatchErr(s) => {
                self.errors.push(s);
                Command::none()
            }
            Message::UI(ui) => {
                match ui {
                    crate::message::UI::GoToSettings => self.current_screen = Screen::Settings,
                    crate::message::UI::GoToDashboard => self.current_screen = Screen::Dashboard,
                    crate::message::UI::ToggleSettings => {
                        if self.current_screen == Screen::Settings {
                            self.current_screen = Screen::Dashboard;
                        } else {
                            self.current_screen = Screen::Settings;
                        }
                    }
                };
                Command::none()
            }
            Message::Calculator(msg) => self.calculator.update(&self.data, msg),
            Message::Settings(msg) => self.settings.update(msg),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(Duration::from_millis(1000)).map(|_| Message::Tick),
            prices::connect().map(Message::PriceEcho),
            user::connect(self.config.api_key.clone()).map(Message::UserEcho),
            if self.pair_submitted {
                trades::connect(self.new_pair.to_lowercase()).map(Message::TradeEcho)
            } else {
                Subscription::none()
            },
            if self.pair_submitted {
                book::connect(self.new_pair.to_lowercase()).map(Message::BookEcho)
            } else {
                Subscription::none()
            },
            /*
            keyboard::on_key_press(|key_code, modifiers| {
                if !modifiers.command() {
                    return None;
                }
                handle_hotkey(key_code)
            })
            */
        ])
    }

    fn view(&self) -> Element<Message> {
        let focus = self.focus;
        let total_panes = self.panes.len();

        let dashboard_grid = PaneGrid::new(&self.panes, |id, pane, is_maximized| {
            let is_focused = focus == Some(id);

            let title = row![text(pane.id.to_string()).style(if is_focused {
                PANE_ID_COLOR_FOCUSED
            } else {
                PANE_ID_COLOR_UNFOCUSED
            })]
            .spacing(5);

            let title_bar = pane_grid::TitleBar::new(title)
                .controls(view_controls(id, total_panes, pane.is_pinned, is_maximized))
                .padding(16)
                .style(if is_focused {
                    style::title_bar_focused
                } else {
                    style::title_bar_active
                });

            pane_grid::Content::new(responsive(|_size| match pane.id {
                PaneType::Prices => watchlist_view(
                    &self.data.prices,
                    &self.watchlist_favorites,
                    self.filter,
                    &self.filter_string,
                ),
                PaneType::Chart => self.chart.view().into(),
                PaneType::Book => book_view(&self.data.book),
                PaneType::Trades => trades_view(&self.data.trades),
                PaneType::Market => market_view(&self.new_price, &self.new_amt, &self.new_pair),
                PaneType::Balances => balances_view(&self.data.balances),
                PaneType::Orders => orders_view(&self.data.orders, &self.data.prices),
                PaneType::Calculator => self.calculator.view(),
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
        .on_click(Message::Clicked)
        .on_drag(Message::Dragged)
        .on_resize(10, Message::Resized);

        let header = container(
            row![
                Row::with_children(
                    self.watchlist_favorites
                        .iter()
                        .map(|t| {
                            let price_now = self.data.prices.get(t).unwrap_or(&0.0);
                            let ticker = t.split("USDT").next().unwrap();
                            let handle = svg::Handle::from_path(format!(
                                "{}/assets/logos/{}.svg",
                                env!("CARGO_MANIFEST_DIR"),
                                ticker
                            ));

                            let svg = svg(handle)
                                .width(Length::Fixed(16.0))
                                .height(Length::Fixed(16.0));
                            return row![svg, text(format!("{:.2}", price_now)).size(14)]
                                .spacing(4)
                                .align_items(iced::Alignment::Center);
                        })
                        .map(Element::from)
                )
                .spacing(12),
                Space::new(Length::Fill, 1),
                button(text("Settings").size(14))
                    .padding(8)
                    .style(iced::theme::Button::Text)
                    .on_press(Message::UI(crate::message::UI::ToggleSettings))
            ]
            .align_items(iced::Alignment::Center),
        )
        .padding([0, 16])
        .style(container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.07, 0.07, 0.07))),
            border: iced::Border {
                radius: 16.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let err_header = container(
            row![
                Row::with_children(self.errors.last().map(text).map(Element::from)).spacing(12),
                Space::new(Length::Fill, 1),
                button(text("X").size(14))
                    .padding(8)
                    .style(iced::theme::Button::Text)
                    .on_press(Message::UI(crate::message::UI::ToggleSettings))
            ]
            .align_items(iced::Alignment::Center),
        )
        .padding([0, 16])
        .style(container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.99, 0.03, 0.03))),
            border: iced::Border {
                radius: 16.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let message_log: Element<_> = if self.data.prices.is_empty() {
            container(text("Loading...").style(Color::from_rgb8(0x88, 0x88, 0x88)))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into()
        } else {
            column![container(
                column![
                    if self.errors.is_empty() {
                        header
                    } else {
                        err_header
                    },
                    if self.current_screen == Screen::Dashboard
                        && !self.config.api_key.is_empty()
                        && !self.config.api_secret_key.is_empty()
                    {
                        container(dashboard_grid)
                    } else {
                        container(self.settings.view())
                    }
                ]
                .spacing(8)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10),]
            .into()
        };

        container(message_log)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .style(|_: &_| container::Appearance {
                background: Some(iced::Background::Color(Color::BLACK)),
                ..Default::default()
            })
            .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
