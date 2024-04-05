use super::orders::tb;

use crate::{
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
                .on_input(|s| DashboardMessage::MarketPairChanged(s).into())
                .width(300.0)
                .on_submit(DashboardMessage::MarketPairSet.into()),
            row![
                column![
                    tin!("price", quote)
                        .on_input(|s| DashboardMessage::MarketQuoteChanged(s).into())
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
                    tin!("amount", amt)
                        .on_input(|s| DashboardMessage::MarketAmtChanged(s).into())
                        .width(150.0),
                    row![
                        bbtn!(text("10%").size(12)).on_press(DashboardMessage::QtySet(0.1).into()),
                        Space::new(Length::Fill, 1.0),
                        bbtn!(text("25%").size(12)).on_press(DashboardMessage::QtySet(0.25).into()),
                        Space::new(Length::Fill, 1.0),
                        bbtn!(text("50%").size(12)).on_press(DashboardMessage::QtySet(0.5).into()),
                        Space::new(Length::Fill, 1.0),
                        bbtn!(text("100%").size(12)).on_press(DashboardMessage::QtySet(1.0).into()),
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
