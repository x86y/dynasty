#![feature(async_closure)]
mod api;
mod views;
mod ws;

use iced::font;
use iced::widget::responsive;
use iced::Font;
use pane_grid::Configuration;
use std::collections::HashMap;
use views::panes::style;
use views::panes::view_controls;
use views::panes::Pane;
use views::panes::PANE_ID_COLOR_FOCUSED;
use views::panes::PANE_ID_COLOR_UNFOCUSED;
use ws::book;
use ws::market;
use ws::prices;
use ws::user;

use api::trade_spot;
use binance::rest_model::{Balance, Order};
use iced::executor;
use iced::widget::{column, container, row, text};
use iced::{Application, Color, Command, Element, Length, Settings, Subscription, Theme};
use views::balances::balances_view;
use views::book::book_view;
use views::market::market_view;
use views::orders::orders_view;
use views::watchlist::watchlist_view;

use iced::widget::pane_grid::{self, PaneGrid};

pub fn main() -> iced::Result {
    App::run(Settings {
        window: iced::window::Settings {
            size: (800, 800),
            ..Default::default()
        },
        default_font: Font::with_name("Monospace"),
        ..Default::default()
    })
}

struct App {
    panes: pane_grid::State<Pane>,
    panes_created: usize,
    focus: Option<pane_grid::Pane>,
    symbols_whitelist: Vec<String>,
    new_price: String,
    new_amt: String,
    new_pair: String,
    pair_submitted: bool,
    data: AppData,
}

#[derive(Default)]
struct AppData {
    prices: HashMap<String, f32>,
    book: Vec<f64>,
    balances: Vec<Balance>,
    orders: Vec<Order>,
    quote: String,
}

#[derive(Debug, Clone)]
pub enum Message {
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
    MarketPairSet,
    MarketPairSet2(()),
    MarketPairUnset(()),
    MarketPrice(prices::Event),
    BuyPressed,
    SellPressed,
    PriceInc(f64),
    QtySet(f64),
    PriceEcho(prices::Event),
    MarketEcho(market::MarketEvent),
    BookEcho(book::BookEvent),
    UserEcho(user::WsUpdate),
    OrdersRecieved(Vec<Order>),
    MarketChanged(String),
    AssetSelected(String),
    BalancesRecieved(Vec<Balance>),
    FontsLoaded(Result<(), iced::font::Error>),
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    type Executor = executor::Default;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let panes = pane_grid::State::with_configuration(Configuration::Split {
            axis: pane_grid::Axis::Horizontal,
            ratio: 0.7,
            a: Box::new(Configuration::Split {
                axis: pane_grid::Axis::Vertical,
                ratio: 0.25,
                a: Box::new(Configuration::Pane(Pane::new(0))),
                b: Box::new(Configuration::Split {
                    axis: pane_grid::Axis::Vertical,
                    ratio: 0.25,
                    a: Box::new(Configuration::Pane(Pane::new(1))),
                    b: Box::new(Configuration::Split {
                        axis: pane_grid::Axis::Vertical,
                        ratio: 0.75,
                        a: Box::new(Configuration::Pane(Pane::new(2))),
                        b: Box::new(Configuration::Pane(Pane::new(3))),
                    }),
                }),
            }),
            b: Box::new(Configuration::Pane(Pane::new(4))),
        });

        (
            App {
                panes,
                panes_created: 1,
                focus: None,
                symbols_whitelist: [
                    "BTCUSDT", "ETHUSDT", "LINKUSDT", "UNIUSDT", "ARBUSDT", "SYNUSDT", "OPUSDT",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                new_price: Default::default(),
                new_amt: Default::default(),
                new_pair: Default::default(),
                pair_submitted: Default::default(),
                data: Default::default(),
            },
            Command::batch(vec![
                Command::perform(api::orders_history(), Message::OrdersRecieved),
                Command::perform(api::balances(), Message::BalancesRecieved),
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
            Message::Split(axis, pane) => {
                let result = self.panes.split(axis, &pane, Pane::new(self.panes_created));

                if let Some((pane, _)) = result {
                    self.focus = Some(pane);
                }

                self.panes_created += 1;
                Command::none()
            }
            Message::SplitFocused(axis) => {
                if let Some(pane) = self.focus {
                    let result = self.panes.split(axis, &pane, Pane::new(self.panes_created));

                    if let Some((pane, _)) = result {
                        self.focus = Some(pane);
                    }

                    self.panes_created += 1;
                }
                Command::none()
            }
            Message::FocusAdjacent(direction) => {
                if let Some(pane) = self.focus {
                    if let Some(adjacent) = self.panes.adjacent(&pane, direction) {
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
                self.panes.resize(&split, ratio);
                Command::none()
            }
            Message::Dragged(pane_grid::DragEvent::Dropped { pane, target }) => {
                self.panes.drop(&pane, target);
                Command::none()
            }
            Message::Dragged(_) => Command::none(),
            Message::Maximize(pane) => {
                self.panes.maximize(&pane);
                Command::none()
            }
            Message::Restore => {
                self.panes.restore();
                Command::none()
            }
            Message::Close(pane) => {
                if let Some((_, sibling)) = self.panes.close(&pane) {
                    self.focus = Some(sibling);
                }
                Command::none()
            }
            Message::CloseFocused => {
                if let Some(pane) = self.focus {
                    if let Some(Pane { is_pinned, .. }) = self.panes.get(&pane) {
                        if !is_pinned {
                            if let Some((_, sibling)) = self.panes.close(&pane) {
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
                trade_spot(
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
                trade_spot(
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
                    prices::Event::MessageReceived(m) => match m {
                        prices::Message::Asset(a) => {
                            self.data.prices.insert(a.name.clone(), a.price);
                            Some(())
                        }
                        _ => None,
                    },
                };
                Command::none()
            }
            Message::MarketEcho(msg) => {
                match msg {
                    market::MarketEvent::MessageReceived(m) => match m {
                        market::Message::BookTicker(bt) => {
                            self.data.book = vec![bt.bid, bt.ask, bt.bid_qty, bt.ask_qty];
                            Some(())
                        }
                        _ => None,
                    },
                };
                Command::none()
            }
            Message::BookEcho(msg) => {
                match msg {
                    book::BookEvent::MessageReceived(m) => match m {
                        book::Message::BookTicker(bt) => {
                            self.data.book = vec![bt.bid, bt.ask, bt.bid_qty, bt.ask_qty];
                            Some(())
                        }
                        _ => None,
                    },
                };
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
                if !a.ends_with("USDT") {
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
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            prices::connect().map(Message::PriceEcho),
            user::connect().map(Message::UserEcho),
            if self.pair_submitted {
                market::connect(self.new_pair.to_lowercase()).map(Message::MarketEcho)
            } else {
                Subscription::none()
            },
            if self.pair_submitted {
                book::connect(self.new_pair.to_lowercase()).map(Message::BookEcho)
            } else {
                Subscription::none()
            },
            /* keyboard::on_key_press(|key_code, modifiers| {
            if !modifiers.command() {
            return None;
            }

            handle_hotkey(key_code)
            }), */
        ])
    }

    fn view(&self) -> Element<Message> {
        let focus = self.focus;
        let total_panes = self.panes.len();

        let pane_grid = PaneGrid::new(&self.panes, |id, pane, is_maximized| {
            let is_focused = focus == Some(id);

            let title = row![text(if pane.id == 0 {
                "Prices"
            } else if pane.id == 1 {
                "Book"
            } else if pane.id == 2 {
                "Market"
            } else if pane.id == 3 {
                "Balances"
            } else {
                "Orders"
            })
            .style(if is_focused {
                PANE_ID_COLOR_FOCUSED
            } else {
                PANE_ID_COLOR_UNFOCUSED
            })]
            .spacing(5);

            let title_bar = pane_grid::TitleBar::new(title)
                .controls(view_controls(id, total_panes, pane.is_pinned, is_maximized))
                .padding(2)
                .style(if is_focused {
                    style::title_bar_focused
                } else {
                    style::title_bar_active
                });

            pane_grid::Content::new(responsive(move |_size| {
                if pane.id == 0 {
                    watchlist_view(&self.data.prices, &self.symbols_whitelist)
                } else if pane.id == 1 {
                    book_view(&self.data.book)
                } else if pane.id == 2 {
                    market_view(&self.new_price, &self.new_amt, &self.new_pair)
                } else if pane.id == 3 {
                    balances_view(&self.data.balances)
                } else {
                    orders_view(&self.data.orders)
                }
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

        let message_log: Element<_> = if self.data.prices.is_empty() {
            container(text("Loading...").style(Color::from_rgb8(0x88, 0x88, 0x88)))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into()
        } else {
            column![container(pane_grid)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10),]
            .into()
        };

        column![message_log]
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .spacing(10)
            .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
