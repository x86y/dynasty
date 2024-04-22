use std::collections::HashMap;

use crate::{api::Client, message::Message, theme::h2c};

use binance::rest_model::{Order, OrderSide, OrderType};
use iced::{
    widget::{column, container, row, text, Column, Space},
    Element, Font, Length,
};

macro_rules! fill {
    () => {
        Space::new(Length::Fill, 0.0)
    };
}
macro_rules! filled { ($($rs:expr),+) => { row![$($rs, fill![]),+] }; }

pub fn t<'a>(s: impl ToString) -> iced::widget::Text<'a> {
    text(s).size(14).style(h2c("EEEEEE").unwrap())
}
pub fn tb<'a>(s: impl ToString) -> iced::widget::Text<'a> {
    t(s).font(Font {
        family: iced::font::Family::Name("Iosevka"),
        weight: iced::font::Weight::Bold,
        ..Default::default()
    })
    .size(14)
    .style(h2c("808080").unwrap())
}

pub fn orders_view<'a>(os: &[Order], ps: &'a HashMap<String, f32>) -> Element<'a, Message> {
    let header = filled![
        tb("Symbol").width(Length::Fixed(100.0)),
        tb("Price").width(Length::Fixed(100.0)),
        tb("Size").width(Length::Fixed(100.0)),
        tb("X-Size").width(Length::Fixed(100.0)),
        tb("Side").width(Length::Fixed(100.0)),
        tb("Status").width(Length::Fixed(100.0)),
        tb("PNL").width(Length::Fixed(100.0)),
        tb("Time").width(Length::Fixed(150.0))
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

            let symbol_t = {
                let s = &b.symbol;
                tb(s).style(h2c("11EE11").unwrap())
            }
            .width(Length::Fixed(100.0));
            let [base, quote] = Client::split_symbol(&b.symbol).unwrap();
            let norm_price = if b.order_type != OrderType::Market {
                b.price
            } else {
                b.cummulative_quote_qty / b.executed_qty
            };
            let price_t = t(format!("{norm_price:.3}")).width(Length::Fixed(100.0));
            let executed_t = t(format!("{} {base}", b.executed_qty)).width(Length::Fixed(100.0));
            let executed_base = t(format!("{:.0} {quote}", b.executed_qty * norm_price))
                .width(Length::Fixed(100.0));
            let side_t = t(format!("{:?}", &b.side))
                .width(Length::Fixed(100.0))
                .style(
                    if b.side == OrderSide::Buy {
                        h2c("11EE11")
                    } else {
                        h2c("EE1111")
                    }
                    .unwrap(),
                );
            let status_t = t(format!("{:?}", &b.status)).width(Length::Fixed(100.0));
            let price_now = ps.get(&b.symbol).unwrap_or(&0.0);

            let pnl = {
                let pnl_value = match b.side {
                    binance::rest_model::OrderSide::Buy => {
                        b.executed_qty * (*price_now as f64 - norm_price)
                    }
                    binance::rest_model::OrderSide::Sell => {
                        b.executed_qty * (norm_price - *price_now as f64)
                    }
                };
                t(format!("{pnl_value:.0}$"))
                    .width(Length::Fixed(100.0))
                    .style(
                        if pnl_value >= 0.0 {
                            h2c("11EE11")
                        } else {
                            h2c("EE1111")
                        }
                        .unwrap(),
                    )
            };

            container(
                filled![
                    symbol_t,
                    price_t,
                    executed_t,
                    executed_base,
                    side_t,
                    status_t,
                    pnl,
                    time_t
                ]
                .width(Length::Fill),
            )
            .padding([2, 4])
            .into()
        })
        .collect();

    column![header, Column::with_children(rows).padding(8)].into()
}
