#![feature(async_closure)]
mod api;
mod views;
mod ws;

use std::collections::HashMap;
use ws::market;
use ws::user;
use ws::prices;

use api::trade_spot;
use binance::rest_model::{Balance, Order};
use iced::executor;
use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Application, Color, Command, Element, Length, Settings, Subscription, Theme};
use once_cell::sync::Lazy;
use views::balances::balances_view;
use views::market::market_view;
use views::orders::orders_view;
use views::watchlist::watchlist_view;

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

#[derive(Default)]
struct App {
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
    MarketQuoteChanged(String),
    MarketAmtChanged(String),
    MarketPairChanged(String),
    MarketPairSet,
    MarketPrice(prices::Event),
    BuyPressed,
    SellPressed,
    Echo(prices::Event),
    MarketEcho(market::MarketEvent),
    UserEcho(user::MarketEvent),
    OrdersRecieved(Vec<Order>),
    MarketChanged(String),
    AssetSelected(String),
    BalancesRecieved(Vec<Balance>),
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    type Executor = executor::Default;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            App {
                symbols_whitelist: [
                    "BTCUSDT", "ETHUSDT", "LINKUSDT", "UNIUSDT", "ARBUSDT", "SYNUSDT", "OPUSDT",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                ..Self::default()
            },
            Command::batch(vec![
                Command::perform(api::orders_history(), Message::OrdersRecieved),
                Command::perform(api::balances(), Message::BalancesRecieved),
            ]),
        )
    }

    fn title(&self) -> String {
        String::from("Dynasty")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::MarketPrice(p) => {
                println!("incame price {p:?}");
                Command::none()
            }
            Message::MarketPairSet => {
                self.pair_submitted = true;
                Command::none()
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
            Message::Echo(msg) => {
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
                        _ => None
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
                dbg!(f);
                Command::none()
            }
            Message::AssetSelected(a) => {
                if !a.ends_with("USDT") {
                    self.new_pair = format!("{a}USDT");
                } else {
                    self.new_pair = a;
                }
                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            prices::connect().map(Message::Echo),
            user::connect().map(Message::UserEcho),
            if self.pair_submitted {
                market::connect(self.new_pair.clone()).map(Message::MarketEcho)
            } else {
                Subscription::none()
            }
        ])
    }

    fn view(&self) -> Element<Message> {
        let message_log: Element<_> = if self.data.prices.is_empty() {
            container(text("Loading...").style(Color::from_rgb8(0x88, 0x88, 0x88)))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into()
        } else {
            column![
                watchlist_view(&self.data.prices, &self.symbols_whitelist),
                market_view(&self.new_price, &self.new_amt, &self.new_pair, &self.data.book),
                row![
                    balances_view(&self.data.balances),
                    Space::new(Length::Fill, 1.0),
                    orders_view(&self.data.orders),
                ]
            ]
            .into()
        };

        column![message_log]
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .spacing(10)
            .into()
    }
}

static MESSAGE_LOG: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
