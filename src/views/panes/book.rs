use super::orders::{t, tb};

use crate::{theme::h2c, views::panes::Message};

use std::collections::BTreeMap;

use iced::{
    widget::{column, row, scrollable, Column, Container},
    Element, Length,
};

pub fn book_view(
    book: &(String, BTreeMap<String, f64>, BTreeMap<String, f64>),
) -> Element<'_, Message> {
    let header = row![
        tb("Price").width(Length::FillPortion(1)),
        tb("Amount").width(Length::FillPortion(1)),
        tb("Total").width(Length::FillPortion(1)),
    ]
    .spacing(10);

    let ask_rows = Column::with_children(
        book.2
            .iter()
            .rev()
            .take(9)
            .map(|(price, quantity)| {
                row![
                    t(format!("{:.2}", price.parse::<f64>().unwrap()))
                        .width(Length::FillPortion(1))
                        .style(iced::Color::from_rgb(1.0, 0.0, 0.0)),
                    t(format!("{:.4}", quantity))
                        .width(Length::FillPortion(1))
                        .style(h2c("B7BDB7").unwrap()),
                    t(format!("{:.2}", price.parse::<f64>().unwrap() * quantity))
                        .style(h2c("B7BDB7").unwrap())
                        .width(Length::FillPortion(1)),
                ]
                .spacing(10)
            })
            .map(Element::from),
    );

    let bid_rows = Column::with_children(
        book.1
            .iter()
            .rev()
            .take(9)
            .map(|(price, quantity)| {
                row![
                    t(format!("{:.2}", price.parse::<f64>().unwrap()))
                        .width(Length::FillPortion(1))
                        .style(iced::Color::from_rgb(0.0, 1.0, 0.0)),
                    t(format!("{:.2}", quantity))
                        .width(Length::FillPortion(1))
                        .style(h2c("B7BDB7").unwrap()),
                    t(format!("{:.2}", price.parse::<f64>().unwrap() * quantity))
                        .width(Length::FillPortion(1))
                        .style(h2c("B7BDB7").unwrap())
                ]
                .spacing(10)
            })
            .map(Element::from),
    );

    let content = column![
        header,
        ask_rows,
        t(format!(
            "${}",
            book.1
                .iter()
                .next_back()
                .unwrap_or((&String::new(), &0.0))
                .0
        ))
        .font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
        bid_rows
    ]
    .padding([2, 12])
    .spacing(10)
    .max_width(500);

    Container::new(scrollable(content)).into()
}
