use std::collections::HashMap;

use binance::rest_model::{Order, OrderType};

use crate::{theme::h2c, Message};
use iced::{
    widget::{column, container, row, scrollable, text, Column, Space},
    Element, Font, Length,
};

macro_rules! fill {
    () => {
        Space::new(Length::Fill, 0.0)
    };
}
macro_rules! filled { ($($rs:expr),+) => { row![$($rs, fill![]),+] }; }

pub fn t<'a>(s: impl ToString) -> iced::widget::Text<'a> {
    text(s)
        .font(Font::with_name("SF Mono"))
        .size(14)
        .style(h2c("EEEEEE").unwrap())
}
pub fn tb<'a>(s: impl ToString) -> iced::widget::Text<'a> {
    t(s).font(Font {
        weight: iced::font::Weight::Bold,
        ..Default::default()
    })
    .size(12)
    .style(h2c("808080").unwrap())
}

pub fn orders_view<'a>(os: &[Order], ps: &'a HashMap<String, f32>) -> Element<'a, Message> {
    let header = filled![
        tb("Time").width(Length::Fixed(150.0)),
        tb("Symbol").width(Length::Fixed(100.0)),
        tb("Price").width(Length::Fixed(100.0)),
        tb("Ex.Qty").width(Length::Fixed(100.0)),
        tb("Ex.Base").width(Length::Fixed(100.0)),
        tb("Side").width(Length::Fixed(100.0)),
        tb("Status").width(Length::Fixed(100.0)),
        tb("PNL").width(Length::Fixed(100.0))
    ]
    .padding([0, 12])
    .width(Length::Fill);

    let rows: Vec<Element<_>> = os
        .iter()
        .map(|b| {
            let time_t = {
                let dt: chrono::DateTime<chrono::Utc> =
                    chrono::TimeZone::timestamp_opt(&chrono::Utc, (b.time / 1000) as i64, 0)
                        .unwrap();
                let formatted_time = dt.format("%m-%d %H:%M").to_string();
                t(formatted_time).width(Length::Fixed(150.0))
            };
            let symbol_t = t(&b.symbol).width(Length::Fixed(100.0));
            let price_t = t(b.price).width(Length::Fixed(100.0));
            let executed_t = t(b.executed_qty).width(Length::Fixed(100.0));
            let executed_base =
                t(format!("{:.0}", b.executed_qty * b.price)).width(Length::Fixed(100.0));
            let side_t = t(format!("{:?}", &b.side)).width(Length::Fixed(100.0));
            let status_t = t(format!("{:?}", &b.status)).width(Length::Fixed(100.0));
            let price_now = ps.get(&b.symbol).unwrap_or(&0.0);

            let pnl = {
                let price = if b.order_type != OrderType::Market {
                    b.price
                } else {
                    b.cummulative_quote_qty / b.executed_qty
                };
                let pnl_value = match b.side {
                    binance::rest_model::OrderSide::Buy => {
                        b.executed_qty * (*price_now as f64 - price)
                    }
                    binance::rest_model::OrderSide::Sell => {
                        b.executed_qty * (price - *price_now as f64)
                    }
                };
                t(format!("{:.0}$", pnl_value)).width(Length::Fixed(100.0))
            };

            container(
                filled![
                    time_t,
                    symbol_t,
                    price_t,
                    executed_t,
                    executed_base,
                    side_t,
                    status_t,
                    pnl
                ]
                .width(Length::Fill),
            )
            .padding([2, 4])
            .into()
        })
        .collect();

    column![
        header,
        scrollable(Column::with_children(rows).padding(8)) //.style(ScrollbarStyle::theme())
    ]
    .into()
}
