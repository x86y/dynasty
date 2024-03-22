mod api;
mod config;
mod theme;
mod views;
mod ws;

use binance::rest_model::OrderStatus;
use binance::ws_model::TradesEvent;
use config::Config;
use iced::font;
use iced::widget::button;
use iced::widget::responsive;
use iced::widget::svg;
use iced::widget::text_editor;
use iced::widget::text_input;
use iced::widget::Row;
use iced::widget::Space;
use iced::Font;
use pane_grid::Configuration;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::env;
use views::panes::calculator::calculator_view;
use views::panes::orders::tb;
use views::panes::style;
use views::panes::view_controls;
use views::panes::Pane;
use views::panes::PaneType;
use views::panes::PANE_ID_COLOR_FOCUSED;
use views::panes::PANE_ID_COLOR_UNFOCUSED;
use ws::book;
use ws::prices;
use ws::trades;
use ws::user;

use binance::rest_model::{Balance, Order};
use iced::executor;
use iced::widget::{column, container, row, text};
use iced::{Application, Color, Command, Element, Length, Settings, Subscription, Theme};
use views::panes::balances::balances_view;
use views::panes::book::book_view;
use views::panes::market::market_view;
use views::panes::orders::orders_view;
use views::panes::trades::trades_view;
use views::panes::watchlist::watchlist_view;
use views::panes::watchlist::WatchlistFilter;

use iced::widget::pane_grid::{self, PaneGrid};

pub fn main() -> iced::Result {
    App::run(Settings {
        window: iced::window::Settings {
            size: iced::Size {
                width: 800.0,
                height: 800.0,
            },
            ..Default::default()
        },
        default_font: Font::with_name("SF Mono"),
        antialiasing: true,
        ..Default::default()
    })
}

struct App {
    panes: pane_grid::State<Pane>,
    panes_created: usize,
    focus: Option<pane_grid::Pane>,
    watchlist_favorites: Vec<String>,
    new_price: String,
    new_amt: String,
    new_pair: String,
    pair_submitted: bool,
    filter: WatchlistFilter,
    filter_string: String,
    data: AppData,
    calculator_content: iced::widget::text_editor::Content,
    calculator_editing: bool,
    current_view: ViewState,
    config: Config,
}

#[derive(PartialEq)]
enum ViewState {
    Dashboard,
    Settings,
}

#[derive(Default)]
struct AppData {
    prices: HashMap<String, f32>,
    book: (String, BTreeMap<String, f64>, BTreeMap<String, f64>),
    trades: VecDeque<TradesEvent>,
    balances: Vec<Balance>,
    orders: Vec<Order>,
    quote: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    CalcToggle,
    CalcAction(text_editor::Action),
    SaveConfig(String, String),
    ConfigUpdated(Result<Config, ()>),
    SettingsApiKeyChanged(String),
    SettingsApiSecretChanged(String),
    SetDashboardView,
    SetSettingsView,
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
    type Flags = ();
    type Executor = executor::Default;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let config = Config::load().unwrap_or_default();

        let panes = pane_grid::State::with_configuration(h![
            0.7,
            v![
                0.25,
                pane![Prices],
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
                current_view: ViewState::Dashboard,
                config: Default::default(),
                calculator_content: iced::widget::text_editor::Content::new(),
                calculator_editing: true,
            },
            Command::batch(vec![
                Command::perform(async { Ok(config) }, Message::ConfigUpdated),
                font::load(include_bytes!("../fonts/icons.ttf").as_slice())
                    .map(Message::FontsLoaded),
            ]),
        )
    }

    fn title(&self) -> String {
        String::from("Dynasty")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::CalcToggle => {
                self.calculator_editing = !self.calculator_editing;
                Command::none()
            }
            Message::CalcAction(action) => {
                self.calculator_content.perform(action);
                Command::none()
            }
            Message::SaveConfig(pub_k, sec_k) => Command::perform(
                async {
                    let config = Config {
                        api_key: pub_k,
                        api_secret_key: sec_k,
                    };
                    config.save().map_err(|_| ())?;

                    Ok(config)
                },
                Message::ConfigUpdated,
            ),
            Message::ConfigUpdated(c) => {
                self.config = c.expect("TODO: bad config popup/message");

                self.current_view = ViewState::Dashboard;
                Command::batch(vec![
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
                ])
            }
            Message::SettingsApiKeyChanged(value) => {
                self.config.api_key = value;
                Command::none()
            }
            Message::SettingsApiSecretChanged(value) => {
                self.config.api_secret_key = value;
                Command::none()
            }
            Message::SetSettingsView => {
                if self.current_view == ViewState::Settings {
                    self.current_view = ViewState::Dashboard;
                } else {
                    self.current_view = ViewState::Settings;
                }
                Command::none()
            }
            Message::SetDashboardView => {
                self.current_view = ViewState::Dashboard;
                Command::none()
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
                Command::none()
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
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
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

            pane_grid::Content::new(responsive(move |_size| match pane.id {
                PaneType::Prices => watchlist_view(
                    &self.data.prices,
                    &self.watchlist_favorites,
                    self.filter,
                    &self.filter_string,
                ),
                PaneType::Book => book_view(&self.data.book),
                PaneType::Trades => trades_view(&self.data.trades),
                PaneType::Market => market_view(&self.new_price, &self.new_amt, &self.new_pair),
                PaneType::Balances => balances_view(&self.data.balances),
                PaneType::Orders => orders_view(&self.data.orders, &self.data.prices),
                PaneType::Calculator => calculator_view(
                    &self.calculator_content,
                    self.calculator_editing,
                    &self.data.prices,
                ),
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
                    .on_press(Message::SetSettingsView)
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

        let api_key_input = text_input("API Key", &self.config.api_key)
            .secure(true)
            .width(Length::Fill)
            .on_input(Message::SettingsApiKeyChanged);
        let api_secret_key_input = text_input("API Secret Key", &self.config.api_secret_key)
            .secure(true)
            .width(Length::Fill)
            .on_input(Message::SettingsApiSecretChanged);

        let settings = container(
            column![
                row![text("API Key:").width(Length::Fixed(100.0)), api_key_input].spacing(10),
                row![
                    text("API Secret Key:").width(Length::Fixed(100.0)),
                    api_secret_key_input,
                ]
                .spacing(10),
                button(tb("Save")).on_press(Message::SaveConfig(
                    self.config.api_key.clone(),
                    self.config.api_secret_key.clone()
                )),
            ]
            .spacing(10)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(iced::Alignment::Center),
        )
        .style(container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.07, 0.07, 0.07))),
            border: iced::Border {
                radius: 16.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .center_x()
        .center_y();

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
                    header,
                    if self.current_view == ViewState::Dashboard
                        && !self.config.api_key.is_empty()
                        && !self.config.api_secret_key.is_empty()
                    {
                        container(dashboard_grid)
                    } else {
                        container(settings)
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
