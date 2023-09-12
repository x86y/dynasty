use binance::rest_model::Order;
use iced::{
    widget::{column, container, scrollable, text, Column},
    Element,
};

use crate::Message;

use super::components::list::{RowA, RowB};

pub fn orders_view(os: &[Order]) -> Element<'_, Message> {
    let header = text(format!(
        "{:<8} {:<6} {:<8} {:<4} {:<8}",
        "Symbol", "Price", "Executed", "Side", "Status"
    ));
    let rows = os
        .iter()
        .enumerate()
        .map(|(i, b)| {
            container(text(format!(
                " {:<8} {:<6} {:<8} {:<4?} {:<8?}",
                b.symbol, b.price, b.executed_qty, b.side, b.status
            )))
            .style(iced::theme::Container::Custom(if i % 2 == 0 {
                Box::new(RowA {})
            } else {
                Box::new(RowB {})
            }))
            .width(320.0)
        })
        .map(Element::from);
    column![
        text("ORDERS").size(24.0),
        header,
        scrollable(Column::with_children(rows.collect()).spacing(4.0)).height(500.0)
    ]
    .into()
}
