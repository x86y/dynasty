use iced::{
    widget::{button, column, row, text, text_input, Space},
    Alignment, Element, Length,
};

use crate::Message;

pub fn market_view<'a>(quote: &str, pair: &str) -> Element<'a, Message> {
    column![
        Space::new(Length::Fill, 1.0),
        text("Buy/Sell").size(24.0),
        text_input("", pair)
            .on_input(Message::MarketPairChanged)
            .width(400.0)
            .on_submit(Message::MarketPairSet),
        text_input("", quote)
            .on_input(Message::MarketQuoteChanged)
            .width(400.0),
        row![
            button("BUY").on_press(Message::BuyPressed),
            button("SELL").on_press(Message::SellPressed),
        ],
        Space::new(Length::Fill, 1.0)
    ]
    .align_items(Alignment::Center)
    .into()
}
