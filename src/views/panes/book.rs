use super::orders::{t, tb};

use crate::{
    data::AppData,
    theme::h2c,
    views::{components::loading::loader, dashboard::DashboardMessage},
};

use iced::{
    widget::{column, row, Column, Container},
    Element, Length,
};

pub(crate) struct BookPane {}

impl BookPane {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn view<'a>(&'a self, data: &'a AppData) -> Element<'a, DashboardMessage> {
        let book = &data.book;

        if book.1.is_empty() {
            return loader!().into();
        }

        let header = row![
            tb("Price").width(Length::Fill),
            tb("Amount").width(Length::Fill),
            tb("Total").width(Length::Fill)
        ]
        .spacing(10);

        let ask_rows = Column::with_children(
            book.2
                .iter()
                .rev()
                .take(12)
                .map(|(price, quantity)| {
                    row![
                        t(format!("{:.2}", price.parse::<f64>().unwrap()))
                            .width(Length::FillPortion(1))
                            .color(iced::Color::from_rgb(1.0, 0.0, 0.0)),
                        t(format!("{quantity:.4}"))
                            .width(Length::FillPortion(1))
                            .color(h2c("B7BDB7").unwrap()),
                        t(format!("{:.2}", price.parse::<f64>().unwrap() * quantity))
                            .color(h2c("B7BDB7").unwrap())
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
                            .color(iced::Color::from_rgb(0.0, 1.0, 0.0)),
                        t(format!("{quantity:.2}"))
                            .width(Length::FillPortion(1))
                            .color(h2c("B7BDB7").unwrap()),
                        t(format!("{:.2}", price.parse::<f64>().unwrap() * quantity))
                            .width(Length::FillPortion(1))
                            .color(h2c("B7BDB7").unwrap())
                    ]
                    .spacing(10)
                })
                .map(Element::from),
        );

        let content = column![
            header,
            ask_rows,
            tb(format!(
                "${}",
                book.1
                    .iter()
                    .next_back()
                    .unwrap_or((&String::new(), &0.0))
                    .0
            ))
            .color(iced::Color::WHITE),
            bid_rows
        ]
        .padding([2, 12])
        .spacing(10)
        .max_width(500);

        Container::new(content).into()
    }
}
