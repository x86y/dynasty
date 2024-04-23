use super::orders::tb;

use crate::{
    api::Client,
    data::AppData,
    message::Message,
    views::components::{
        better_btn::{GreenBtn, RedBtn},
        input::Inp,
    },
    ws::Websockets,
};

use iced::{
    widget::{button, column, container, row, text, text_input, Space},
    Alignment, Command, Element, Length,
};

macro_rules! bbtn {
    ($e: expr) => {
        button($e).style(iced::theme::Button::Text).padding(8)
    };
}

macro_rules! tin {
    ($e: expr, $b: expr) => {
        text_input($e, $b).style(iced::theme::TextInput::Custom(Box::new(Inp {})))
    };
}

#[derive(Debug, Clone)]
pub(crate) enum MarketPanelMessage {
    BuyPressed,
    SellPressed,
    PriceMultiplied(f64),
    PriceInput(String),
    AmountMultiplied(f64),
    AmountInput(String),
    PairSet,
    PairInput(String),
}

pub(crate) struct Market {
    price: String,
    amount: String,
    pair: String,
}

impl Market {
    pub(crate) fn new() -> Self {
        Self {
            price: String::default(),
            amount: String::default(),
            pair: "BTCUSDT".to_owned(),
        }
    }

    /// currently entered pair of currencies
    pub(crate) fn pair(&self) -> &str {
        &self.pair
    }

    pub(crate) fn view(&self) -> Element<'_, MarketPanelMessage> {
        container(
            column![
                Space::new(Length::Fill, 1.0),
                tin!("type a pair", &self.pair)
                    .on_input(MarketPanelMessage::PairInput)
                    .width(300.0)
                    .on_submit(MarketPanelMessage::PairSet),
                row![
                    column![
                        tin!("price", &self.price)
                            .on_input(MarketPanelMessage::PriceInput)
                            .width(150.0),
                        row![
                            bbtn!(text("-0.1%").size(12))
                                .on_press(MarketPanelMessage::PriceMultiplied(-0.1)),
                            bbtn!(text("+0.1%").size(12))
                                .on_press(MarketPanelMessage::PriceMultiplied(0.1)),
                        ]
                        .spacing(2.0)
                        .width(150.0),
                    ],
                    column![
                        tin!("amount", &self.amount)
                            .on_input(MarketPanelMessage::AmountInput)
                            .width(150.0),
                        row![
                            bbtn!(text("10%").size(12))
                                .on_press(MarketPanelMessage::AmountMultiplied(0.1)),
                            Space::new(Length::Fill, 1.0),
                            bbtn!(text("25%").size(12))
                                .on_press(MarketPanelMessage::AmountMultiplied(0.25)),
                            Space::new(Length::Fill, 1.0),
                            bbtn!(text("50%").size(12))
                                .on_press(MarketPanelMessage::AmountMultiplied(0.5)),
                            Space::new(Length::Fill, 1.0),
                            bbtn!(text("100%").size(12))
                                .on_press(MarketPanelMessage::AmountMultiplied(1.0)),
                        ]
                        .width(150.0),
                    ]
                ]
                .spacing(4.0)
                .width(300.0),
                row![
                    button(tb("Buy").style(iced::Color::WHITE).size(12))
                        .style(iced::theme::Button::Custom(Box::new(GreenBtn {})))
                        .padding(8)
                        .on_press(MarketPanelMessage::BuyPressed),
                    Space::new(5.0, 0.0),
                    button(tb("Sell").style(iced::Color::WHITE).size(12))
                        .style(iced::theme::Button::Custom(Box::new(RedBtn {})))
                        .padding(8)
                        .on_press(MarketPanelMessage::SellPressed)
                ],
                Space::new(Length::Fill, 1.0)
            ]
            .spacing(4.0)
            .align_items(Alignment::Center),
        )
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into()
    }

    pub(crate) fn update(
        &mut self,
        msg: MarketPanelMessage,
        api: &Client,
        data: &AppData,
        ws: &Websockets,
    ) -> Command<Message> {
        match msg {
            MarketPanelMessage::BuyPressed => self.buy(api),
            MarketPanelMessage::SellPressed => self.sell(api),
            MarketPanelMessage::AmountMultiplied(f) => {
                let usdt_b = data
                    .balances
                    .iter()
                    .find(|b| b.asset == "USDT")
                    .unwrap()
                    .free;
                self.amount = (usdt_b * f).to_string();
                Command::none()
            }
            MarketPanelMessage::PriceInput(new) => {
                self.price = new;
                Command::none()
            }
            MarketPanelMessage::AmountInput(new) => {
                self.amount = new;
                Command::none()
            }
            MarketPanelMessage::PriceMultiplied(inc) => {
                let price = data
                    .prices
                    .get(&self.pair)
                    .expect("price exists for some reason");
                self.price =
                    (((*price as f64 * (1.0 + (inc / 100.0))) * 100.0).round() / 100.0).to_string();
                Command::none()
            }
            MarketPanelMessage::PairSet => {
                ws.track_new_currency_pair(&self.pair);
                Command::none()
            }
            MarketPanelMessage::PairInput(new) => {
                self.pair = new;
                Command::none()
            }
        }
    }

    fn buy(&mut self, api: &Client) -> Command<Message> {
        api.trade_spot(
            self.pair.clone(),
            self.price.parse().unwrap(),
            self.amount.parse().unwrap(),
            binance::rest_model::OrderSide::Buy,
        )
    }

    fn sell(&mut self, api: &Client) -> Command<Message> {
        api.trade_spot(
            self.pair.clone(),
            self.price.parse().unwrap(),
            self.amount.parse().unwrap(),
            binance::rest_model::OrderSide::Sell,
        )
    }

    // FIXME: this is totally wrong and broken
    /// Set new pair from selected currency
    pub(crate) fn pair_selected(&mut self, mut new: String) {
        // FIXME: this breaks with any other currency
        if !(new.ends_with("USDT") || new.ends_with("BTC") || new.ends_with("ETH")) {
            new = format!("{new}USDT");
        }
        self.pair = new;
    }
}
