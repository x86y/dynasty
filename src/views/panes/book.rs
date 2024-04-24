use super::orders::{t, tb};

use crate::{data::AppData, theme::h2c, views::dashboard::DashboardMessage};

use iced::{
    widget::{column, row, Column, Container},
    Element, Length,
};

pub fn book_view<'a>(
    data: &'a AppData,
    loader: &'a crate::views::components::loading::Loader,
) -> Element<'a, DashboardMessage> {
    let book = &data.book;

    if book.1.is_empty() {
        return loader.view();
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
                        .style(iced::Color::from_rgb(1.0, 0.0, 0.0)),
                    t(format!("{quantity:.4}"))
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
                    t(format!("{quantity:.2}"))
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
        tb(format!(
            "${}",
            book.1
                .iter()
                .next_back()
                .unwrap_or((&String::new(), &0.0))
                .0
        ))
        .style(iced::Color::WHITE),
        bid_rows
    ]
    .padding([2, 12])
    .spacing(10)
    .max_width(500);

    Container::new(content).into()
}
