use iced::{
    widget::{button, column, row, text, text_input, Space},
    Alignment, Element, Length,
};

use crate::Message;

pub fn market_view<'a>(quote: &str, amt: &str, pair: &str, book: &[f64]) -> Element<'a, Message> {
    column![
        Space::new(Length::Fill, 1.0),
        if !book.is_empty() {
            row![
                column![
                    text(format!("ask {} ", book[0])),
                    text(format!("bid {} ", book[1])),
                ],
                column![
                    text(format!("qty {} ", book[2])),
                    text(format!("qty {}", book[3]))
                ]
            ]
        } else {
            row![]
        },
        text_input("Select Pair", pair)
            .on_input(Message::MarketPairChanged)
            .width(400.0)
            .on_submit(Message::MarketPairSet),
        row![
            text_input("price", quote)
                .on_input(Message::MarketQuoteChanged)
                .width(150.0),
            Space::new(Length::Fill, 1.0),
            text_input("amount", amt)
                .on_input(Message::MarketAmtChanged)
                .width(150.0),
        ].width(400.0),
        row![
            button("BUY").on_press(Message::BuyPressed),
            Space::new(5.0, 0.0),
            button("SELL").on_press(Message::SellPressed),
        ],
        Space::new(Length::Fill, 1.0)
    ]
    .align_items(Alignment::Center)
    .into()
}
