use super::orders::{t, tb};

use crate::{message::Message, theme::h2c, ws::trades::TradesEvent};

use std::collections::VecDeque;

use iced::{
    widget::{column, container, row, scrollable, Column},
    Color, Element, Length,
};

pub fn trades_view(bs: &VecDeque<TradesEvent>) -> Element<'_, Message> {
    column![
        row![
            tb("Price").width(Length::Fill),
            tb("Amount").width(Length::Fill),
            tb("Time").width(Length::Fill)
        ],
        scrollable(Column::with_children(
            bs.iter()
                .map(|b| {
                    let c = if b.is_buyer_maker {
                        Color::from_rgb(1.0, 0.0, 0.0)
                    } else {
                        Color::from_rgb(0.0, 1.0, 0.0)
                    };

                    let timestamp = b.trade_order_time as i64;
                    let dt = chrono::DateTime::from_timestamp(timestamp / 1000, 0).unwrap();
                    let time_str = dt.format("%H:%M:%S").to_string();

                    container(row![
                        t(format!("{:.2}", b.price)).style(c).width(Length::Fill),
                        t(format!("{:.2} ", b.qty))
                            .width(Length::Fill)
                            .style(h2c("B7BDB7").unwrap()),
                        t(time_str)
                            .style(h2c("B7BDB7").unwrap())
                            .width(Length::Fill),
                    ])
                    .width(Length::Fill)
                })
                .map(Element::from),
        )) // .style(ScrollbarStyle::theme())
    ]
    .padding([2, 12])
    .into()
}
