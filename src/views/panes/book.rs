use iced::{
    widget::{column, container, row, text, Space},
    Color, Element, Length,
};

use crate::{
    views::components::list::{RowA, RowB},
    Message,
};

pub fn book_view(book: &[f64]) -> Element<'_, Message> {
    if !book.is_empty() {
        column![
            container(row![
                text(format!(" {}", book[1])).style(Color::from_rgb(1.0, 0.0, 0.0)),
                Space::new(Length::Fill, 1.0),
                text(format!(" {}", book[3])).style(Color::from_rgb(1.0, 0.0, 0.0))
            ])
            .style(iced::theme::Container::Custom(Box::new(RowA {})))
            .width(Length::Fill),
            container(row![
                text(format!(" {}", book[0])).style(Color::from_rgb(0.0, 1.0, 0.0)),
                Space::new(Length::Fill, 1.0),
                text(format!(" {}", book[2])).style(Color::from_rgb(0.0, 1.0, 0.0))
            ])
            .style(iced::theme::Container::Custom(Box::new(RowB {})))
            .width(Length::Fill),
        ]
    } else {
        column![]
    }
    .into()
}
