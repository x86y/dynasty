use super::orders::t;

use crate::views::{
    components::{
        better_btn::{GreenBtn, RedBtn},
        input::Inp,
    },
    panes::Message,
};

use iced::Font;
use iced::{
    widget::{button, column, container, row, text, text_input, Space},
    Alignment, Element, Length,
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

pub fn market_view<'a>(quote: &str, amt: &str, pair: &str) -> Element<'a, Message> {
    container(
        column![
            Space::new(Length::Fill, 1.0),
            tin!("type a pair", pair)
                .on_input(Message::MarketPairChanged)
                .width(300.0)
                .on_submit(Message::MarketPairSet),
            row![
                column![
                    tin!("price", quote)
                        .on_input(Message::MarketQuoteChanged)
                        .width(150.0),
                    row![
                        bbtn!(text("-0.1%").size(12)).on_press(Message::PriceInc(-0.1)),
                        bbtn!(text("+0.1%").size(12)).on_press(Message::PriceInc(0.1)),
                    ]
                    .spacing(2.0)
                    .width(150.0),
                ],
                column![
                    tin!("amount", amt)
                        .on_input(Message::MarketAmtChanged)
                        .width(150.0),
                    row![
                        bbtn!(text("10%").size(12)).on_press(Message::QtySet(0.1)),
                        Space::new(Length::Fill, 1.0),
                        bbtn!(text("25%").size(12)).on_press(Message::QtySet(0.25)),
                        Space::new(Length::Fill, 1.0),
                        bbtn!(text("50%").size(12)).on_press(Message::QtySet(0.5)),
                        Space::new(Length::Fill, 1.0),
                        bbtn!(text("100%").size(12)).on_press(Message::QtySet(1.0)),
                    ]
                    .width(150.0),
                ]
            ]
            .spacing(4.0)
            .width(300.0),
            row![
                button(
                    t("Buy")
                        .font(Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .size(12)
                )
                .style(iced::theme::Button::Custom(Box::new(GreenBtn {})))
                .padding(8)
                .on_press(Message::BuyPressed),
                Space::new(5.0, 0.0),
                button(
                    t("Sell")
                        .font(Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .size(12)
                )
                .style(iced::theme::Button::Custom(Box::new(RedBtn {})))
                .padding(8)
                .on_press(Message::SellPressed)
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
