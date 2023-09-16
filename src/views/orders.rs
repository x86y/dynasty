use binance::rest_model::Order;
use iced::{
    widget::{column, container, row, scrollable, text, Column, Space},
    Element, Font, Length,
};

use crate::Message;

use super::components::list::{RowA, RowB};

macro_rules! fill {
    () => {
        Space::new(Length::Fill, 0.0)
    };
}
pub fn t<'a>(s: impl ToString) -> iced::widget::Text<'a> {
    text(s).font(Font::with_name("SF Mono"))
}

pub fn orders_view(os: &[Order]) -> Element<'_, Message> {
    let header = row![
        t("Time").width(Length::Fixed(150.0)),
        fill![],
        t("Id").width(Length::Fixed(150.0)),
        fill![],
        t("Symbol").width(Length::Fixed(100.0)),
        fill![],
        t("Price").width(Length::Fixed(100.0)),
        fill![],
        t("Executed").width(Length::Fixed(100.0)),
        fill![],
        t("Side").width(Length::Fixed(100.0)),
        fill![],
        t("Status").width(Length::Fixed(100.0)),
        fill![],
    ]
    .width(Length::Fill);

    let rows: Vec<Element<_>> = os
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let time_t = t(format!("{:?}", &b.time)).width(Length::Fixed(150.0));
            let id_t = t(format!("{:?}", &b.order_id)).width(Length::Fixed(150.0));
            let symbol_t = t(&b.symbol).width(Length::Fixed(100.0));
            let price_t = t(b.price).width(Length::Fixed(100.0));
            let executed_t = t(b.executed_qty).width(Length::Fixed(100.0));
            let side_t = t(format!("{:?}", &b.side)).width(Length::Fixed(100.0));
            let status_t = t(format!("{:?}", &b.status)).width(Length::Fixed(100.0));

            container(
                row![
                    time_t,
                    fill![],
                    id_t,
                    fill![],
                    symbol_t,
                    fill![],
                    price_t,
                    fill![],
                    executed_t,
                    fill![],
                    side_t,
                    fill![],
                    status_t,
                    fill![],
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

    column![header, scrollable(Column::with_children(rows))].into()
}
