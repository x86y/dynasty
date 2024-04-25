use super::orders::{t, tb};
use crate::{data::AppData, theme::h2c, views::dashboard::DashboardMessage};

use iced::{
    widget::{column, container, row, scrollable, Column},
    Color, Element, Length,
};
use ringbuf::{ring_buffer::RbBase, Rb};

pub fn trades_view<'a>(
    data: &'a AppData,
    loader: &'a crate::views::components::loading::Loader,
) -> Element<'a, DashboardMessage> {
    if data.trades.is_empty() {
        return loader.view();
    }

    column![
        row![
            tb("Price").width(Length::Fill),
            tb("Amount").width(Length::Fill),
            tb("Time").width(Length::Fill)
        ],
        scrollable(Column::with_children(
            data.trades
                .iter()
                .rev()
                .map(|b| {
                    let c = if b.is_buyer_maker {
                        Color::from_rgb(1.0, 0.0, 0.0)
                    } else {
                        Color::from_rgb(0.0, 1.0, 0.0)
                    };

                    container(row![
                        t(&b.price).style(c).width(Length::Fill),
                        t(&b.qty).width(Length::Fill).style(h2c("B7BDB7").unwrap()),
                        t(&b.trade_order_time_formatted)
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
