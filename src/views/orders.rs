use binance::rest_model::Order;
use iced::{
    widget::{column, container, row, scrollable, text, Column, Space},
    Element, Length,
};

use crate::Message;

use super::components::list::{RowA, RowB};

pub fn orders_view(os: &[Order]) -> Element<'_, Message> {
    let symbol_header = text("Symbol");
    let id_header = text("Id");
    let time_header = text("Time");
    let price_header = text("Price");
    let executed_header = text("Executed");
    let side_header = text("Side");
    let status_header = text("Status");

    let header = row![
        time_header,
        Space::new(Length::Fill, 0.0),
        id_header,
        Space::new(Length::Fill, 0.0),
        symbol_header,
        Space::new(Length::Fill, 0.0),
        price_header,
        Space::new(Length::Fill, 0.0),
        executed_header,
        Space::new(Length::Fill, 0.0),
        side_header,
        Space::new(Length::Fill, 0.0),
        status_header,
        Space::new(Length::Fill, 0.0),
    ]
    .width(Length::Fill);

    let rows: Vec<Element<_>> = os
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let time_text = text(format!("{:?}", &b.time)).width(Length::Fixed(40.0));
            let id_text = text(format!("{:?}", &b.order_id)).width(Length::Fixed(40.0));
            let symbol_text = text(&b.symbol).width(Length::Fixed(100.0));
            let price_text = text(b.price).width(Length::Fixed(50.0));
            let executed_text = text(b.executed_qty).width(Length::Fixed(50.0));
            let side_text = text(format!("{:?}", &b.side)).width(Length::Fixed(40.0));
            let status_text = text(format!("{:?}", &b.status)).width(Length::Fixed(40.0));

            container(
                row![
                    time_text.width(Length::Shrink),
                    Space::new(Length::Fill, 0.0),
                    id_text.width(Length::Shrink),
                    Space::new(Length::Fill, 0.0),
                    symbol_text,
                    Space::new(Length::Fill, 0.0),
                    price_text,
                    Space::new(Length::Fill, 0.0),
                    executed_text,
                    Space::new(Length::Fill, 0.0),
                    side_text,
                    Space::new(Length::Fill, 0.0),
                    status_text,
                    Space::new(Length::Fill, 0.0),
                ]
                .width(Length::Fill),
            )
            .style(iced::theme::Container::Custom(if i % 2 == 0 {
                Box::new(RowA {})
            } else {
                Box::new(RowB {})
            }))
            .into()
        })
        .collect();

    let orders_column = column![
        header,
        scrollable(Column::with_children(rows).spacing(4.0)).height(500.0),
    ];

    orders_column.into()
}
