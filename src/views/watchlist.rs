use crate::Message;
use iced::{
    widget::{button, container, row, scrollable, text, Column, Space},
    Element, Length,
};
use std::collections::HashMap;

use super::components::{
    list::{RowA, RowB},
    unstyled_btn::UnstyledBtn,
};

pub fn watchlist_view<'a>(
    ps: &'a HashMap<String, f32>,
    whitelist: &'a [String],
) -> Element<'a, Message> {
    let mut sorted_assets: Vec<_> = ps
        .iter()
        .filter_map(|(n, p)| {
            if whitelist.contains(n) {
                Some((n, p))
            } else {
                None
            }
        })
        .collect();
    sorted_assets.sort_by(|(_, p1), (_, p2)| p2.partial_cmp(p1).unwrap());

    scrollable(
        Column::with_children(
            sorted_assets
                .iter()
                .enumerate()
                .map(|(i, (n, p))| {
                    container(row![
                        button(text(n))
                            .on_press(Message::AssetSelected(n.to_string()))
                            .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {}))),
                        Space::new(Length::Fill, 1.0),
                        button(text(p))
                            .on_press(Message::AssetSelected(n.to_string()))
                            .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {}))),
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
        )
        .spacing(5),
    )
    .into()
}
