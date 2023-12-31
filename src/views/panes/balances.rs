use binance::rest_model::Balance;
use iced::{
    widget::{button, container, row, scrollable, text, Column, Space},
    Element, Length,
};

use crate::{theme::h2c, Message};

use crate::views::components::{
    list::{RowA, RowB},
    unstyled_btn::UnstyledBtn,
};

pub fn balances_view(bs: &[Balance]) -> Element<'_, Message> {
    scrollable(Column::with_children(
        bs.iter()
            .enumerate()
            .map(|(i, b)| {
                container(row![
                    button(text(&b.asset).style(h2c("EFE1D1").unwrap()))
                        .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {})))
                        .on_press(Message::AssetSelected(b.asset.clone())),
                    Space::new(Length::Fill, 1.0),
                    // button(text("***"))
                    button(
                        text(format!("{}", (b.free * 10.0).round() / 10.0))
                            .style(h2c("03DAC6").unwrap())
                    )
                    .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {})))
                    .on_press(Message::AssetSelected(b.asset.clone())),
                ])
                .style(iced::theme::Container::Custom(if i % 2 == 0 {
                    Box::new(RowA {})
                } else {
                    Box::new(RowB {})
                }))
                .width(Length::Fill)
            })
            .map(Element::from)
            .collect(),
    ))
    .into()
}
