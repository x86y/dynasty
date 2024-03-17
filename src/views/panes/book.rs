use std::collections::BTreeMap;

use iced::{
    widget::{column, row, scrollable, Column, Container, Rule},
    Element, Length,
};

use crate::Message;

use super::orders::t;

pub fn book_view(book: &(String, BTreeMap<String, f64>, BTreeMap<String, f64>)) -> Element<'_, Message> {
    let header = row![
        t("Price").width(Length::FillPortion(1)),
        t("Amount").width(Length::FillPortion(1)),
        t("Total").width(Length::FillPortion(1)),
    ]
    .spacing(10)
    .padding(10);

    let ask_rows = Column::with_children(
        book.1
            .iter()
            .take(8)
            .map(|(price, quantity)| {
                row![
                    t(format!("{:.2}", price.parse::<f64>().unwrap()))
                        .width(Length::FillPortion(1))
                        .style(iced::Color::from_rgb(1.0, 0.0, 0.0)),
                    t(format!("{:.4}", quantity))
                        .width(Length::FillPortion(1))
                        .style(iced::Color::from_rgb(1.0, 0.0, 0.0)),
                    t(format!("{:.2}", price.parse::<f64>().unwrap() * quantity))
                        .style(iced::Color::from_rgb(1.0, 0.0, 0.0))
                        .width(Length::FillPortion(1)),
                ]
                .spacing(10)
            })
            .map(Element::from)
    );

    let bid_rows = Column::with_children(
        book.2
            .iter()
            .rev()
            .take(8)
            .map(|(price, quantity)| {
                row![
                    t(format!("{:.2}", price.parse::<f64>().unwrap())).width(Length::FillPortion(1))
                        .style(iced::Color::from_rgb(0.0, 1.0, 0.0)),
                    t(format!("{:.2}", quantity)).width(Length::FillPortion(1))
                        .style(iced::Color::from_rgb(0.0, 1.0, 0.0)),
                    t(format!("{:.2}", price.parse::<f64>().unwrap() * quantity))
                        .style(iced::Color::from_rgb(0.0, 1.0, 0.0))
                        .width(Length::FillPortion(1))
                ]
                .spacing(10)
            })
            .map(Element::from)
    );

    let content = column![
        header,
        ask_rows,
        Rule::horizontal(1),
        t(format!("${}", book.1.iter().next_back().unwrap().0)),
        Rule::horizontal(1),
        bid_rows,
    ]
    .spacing(10)
    .max_width(500);

    Container::new(scrollable(content)).into()
}

