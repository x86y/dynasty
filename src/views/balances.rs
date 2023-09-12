use binance::rest_model::Balance;
use iced::{
    widget::{button, container, scrollable, text, Column},
    Element, Length,
};

use crate::Message;

use super::components::{
    list::{RowA, RowB},
    unstyled_btn::UnstyledBtn,
};

pub fn balances_view(bs: &[Balance]) -> Element<'_, Message> {
    scrollable(
        Column::with_children(
            bs.iter()
                .enumerate()
                .map(|(i, b)| {
                    container(
                        button(text(format!(
                            "{:<8} {:<8}",
                            b.asset,
                            (b.free * 10.0).round() / 10.0
                        )))
                        .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {})))
                        .on_press(Message::AssetSelected(b.asset.clone())),
                    )
                    .style(iced::theme::Container::Custom(if i % 2 == 0 {
                        Box::new(RowA {})
                    } else {
                        Box::new(RowB {})
                    }))
                    .width(Length::Fill)
                })
                .map(Element::from)
                .collect(),
        )
        .spacing(5),
    )
    .into()
}
