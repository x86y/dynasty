use std::collections::VecDeque;

use binance::ws_model::TradesEvent;
use iced::{
    widget::{container, row, scrollable, Column, Space},
    Color, Element, Length,
};

use crate::{theme::h2c, Message};

use super::{components::list::{RowA, RowB}, orders::t};

pub fn trades_view(bs: &VecDeque<TradesEvent>) -> Element<'_, Message> {
    scrollable(Column::with_children(
        bs.iter()
            .enumerate()
            .map(|(i, b)| {
                let c = if b.is_buyer_maker {
                    Color::from_rgb(1.0, 0.0, 0.0)
                } else {
                    Color::from_rgb(0.0, 1.0, 0.0)
                };
                container(row![
                    t(&b.symbol).style(h2c("EFE1D1").unwrap()).style(c),
                    Space::new(Length::Fill, 1.0),
                    t(format!("{:.2}", b.price.parse::<f32>().unwrap())).style(c),
                    Space::new(Length::Fill, 1.0),
                    t(format!("{:.2}", b.qty.parse::<f32>().unwrap())).style(c),
                ])
                .style(iced::theme::Container::Custom(if i % 2 == 0 {
                    Box::new(RowA {})
                } else {
                    Box::new(RowB {})
                }))
                .width(Length::Fill)
            })
            .map(Element::from)
            .collect(),
    ))
    .into()
}
