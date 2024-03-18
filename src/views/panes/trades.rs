use std::collections::VecDeque;

use binance::ws_model::TradesEvent;
use iced::{
    widget::{column, container, row, scrollable, Column, Space},
    Color, Element, Length,
};

use crate::{theme::h2c, Message};

use super::orders::{t, tb};

pub fn trades_view(bs: &VecDeque<TradesEvent>) -> Element<'_, Message> {
    column![
        row![
            tb("Price"),
            Space::new(Length::Fill, 1.0),
            tb("Amount"),
            Space::new(Length::Fill, 1.0),
            tb("Time")
        ],
        scrollable(Column::with_children(
            bs.iter()
                .map(|b| {
                    let c = if b.is_buyer_maker {
                        Color::from_rgb(1.0, 0.0, 0.0)
                    } else {
                        Color::from_rgb(0.0, 1.0, 0.0)
                    };
                    container(row![
                        t(format!("{:.2}", b.price.parse::<f32>().unwrap())).style(c),
                        Space::new(Length::Fill, 1.0),
                        t(format!("{:.2} ", b.qty.parse::<f32>().unwrap()))
                            .style(h2c("B7BDB7").unwrap()),
                        Space::new(Length::Fill, 1.0),
                        t(b.trade_order_time).style(h2c("B7BDB7").unwrap()),
                    ])
                    .width(Length::Fill)
                })
                .map(Element::from),
        )) //    .style(ScrollbarStyle::theme())
    ]
    .padding([2, 12])
    .into()
}
