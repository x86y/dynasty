use super::orders::tb;

use crate::{
    api::Client,
    message::Message,
    views::{
        components::{
            better_btn::{GreenBtn, RedBtn},
            input::Inp,
        },
        dashboard::DashboardMessage,
    },
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

pub(crate) struct Market {
    price: String,
    amount: String,
    pair: String,
    ws_pair: String,
}

impl Market {
    pub(crate) fn new() -> Self {
        let mut m = Self {
            price: String::default(),
            amount: String::default(),
            pair: String::new(),
            ws_pair: String::new(),
        };

        m.set_pair("BTCUSDT".to_owned(), false);

        m
    }

    /// currently entered pair of currencies
    pub(crate) fn pair(&self) -> &str {
        &self.pair
    }

    /// currently entered pair of currencies, in binance websocket compatible format
    pub(crate) fn ws_pair(&self) -> &str {
        &self.ws_pair
    }

    pub(crate) fn view(&self) -> Element<'_, Message> {
        container(
            column![
                Space::new(Length::Fill, 1.0),
                tin!("type a pair", &self.pair)
                    .on_input(|s| DashboardMessage::MarketPairInput(s).into())
                    .width(300.0)
                    .on_submit(DashboardMessage::MarketPairSet.into()),
                row![
                    column![
                        tin!("price", &self.price)
                            .on_input(|s| DashboardMessage::MarketPriceInput(s).into())
                            .width(150.0),
                        row![
                            bbtn!(text("-0.1%").size(12))
                                .on_press(DashboardMessage::PriceInc(-0.1).into()),
                            bbtn!(text("+0.1%").size(12))
                                .on_press(DashboardMessage::PriceInc(0.1).into()),
                        ]
                        .spacing(2.0)
                        .width(150.0),
                    ],
                    column![
                        tin!("amount", &self.amount)
                            .on_input(|s| DashboardMessage::MarketAmountInput(s).into())
                            .width(150.0),
                        row![
                            bbtn!(text("10%").size(12))
                                .on_press(DashboardMessage::QtySet(0.1).into()),
                            Space::new(Length::Fill, 1.0),
                            bbtn!(text("25%").size(12))
                                .on_press(DashboardMessage::QtySet(0.25).into()),
                            Space::new(Length::Fill, 1.0),
                            bbtn!(text("50%").size(12))
                                .on_press(DashboardMessage::QtySet(0.5).into()),
                            Space::new(Length::Fill, 1.0),
                            bbtn!(text("100%").size(12))
                                .on_press(DashboardMessage::QtySet(1.0).into()),
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
                        .on_press(DashboardMessage::BuyPressed.into()),
                    Space::new(5.0, 0.0),
                    button(tb("Sell").style(iced::Color::WHITE).size(12))
                        .style(iced::theme::Button::Custom(Box::new(RedBtn {})))
                        .padding(8)
                        .on_press(DashboardMessage::SellPressed.into())
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

    pub(crate) fn buy_pressed(&mut self, api: &Client) -> Command<Message> {
        api.trade_spot(
            self.pair.clone(),
            self.price.parse().unwrap(),
            self.amount.parse().unwrap(),
            binance::rest_model::OrderSide::Buy,
        )
    }

    pub(crate) fn sell_pressed(&mut self, api: &Client) -> Command<Message> {
        api.trade_spot(
            self.pair.clone(),
            self.price.parse().unwrap(),
            self.amount.parse().unwrap(),
            binance::rest_model::OrderSide::Sell,
        )
    }

    pub(crate) fn set_price(&mut self, new: String) {
        self.price = new;
    }

    pub(crate) fn set_amount(&mut self, new: String) {
        self.amount = new;
    }

    // FIXME: this is totally wrong and broken
    /// Set new pair
    ///
    /// from_selection means value came from other widget and needs to be normalized
    pub(crate) fn set_pair(&mut self, mut new: String, from_selection: bool) {
        // FIXME: this breaks with any other currency
        if from_selection
            && !(new.ends_with("USDT") || new.ends_with("BTC") || new.ends_with("ETH"))
        {
            new = format!("{new}USDT");
        }

        self.ws_pair = new.to_lowercase();
        self.pair = new;
    }
}
