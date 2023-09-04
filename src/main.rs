#![feature(async_closure)]
mod api;
mod views;
mod ws;

use ws::prices;
use std::collections::HashMap;

use api::trade_spot;
use binance::rest_model::{Balance, Order};
use iced::executor;
use iced::widget::{column, container, scrollable, text, row, Space};
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
    new_message: String,
    new_pair: String,
    data: AppData,
}

#[derive(Default)]
struct AppData {
    prices: HashMap<String, f32>,
    balances: Vec<Balance>,
    orders: Vec<Order>,
    quote: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    MarketQuoteChanged(String),
    MarketPairChanged(String),
    MarketPairSet,
    MarketPrice(prices::Event),
    BuyPressed,
    SellPressed,
    Echo(prices::Event),
    OrdersRecieved(Vec<Order>),
    MarketChanged(String),
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
                Command::none()
            }
            Message::BuyPressed => Command::perform(
                trade_spot(
                    "LINKUSDT".to_string(),
                    0.001,
                    binance::rest_model::OrderSide::Buy,
                ),
                |m| {
                    println!("{m:?}");
                    Message::MarketChanged("REEEEE".to_string())
                },
            ),
            Message::SellPressed => Command::perform(
                trade_spot(
                    "LINKUSDT".to_string(),
                    0.001,
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
                self.new_message = nm;
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
            Message::OrdersRecieved(orders) => {
                self.data.orders = orders;
                Command::none()
            }
            Message::BalancesRecieved(bals) => {
                self.data.balances = bals;
                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        prices::connect().map(Message::Echo)
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
                market_view(&self.new_message, &self.new_pair),
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
