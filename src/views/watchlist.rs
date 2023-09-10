use crate::{Message, MESSAGE_LOG};
use iced::{
    widget::{button, column, scrollable, text, Column},
    Element, Length,
};
use std::collections::HashMap;

pub fn watchlist_view<'a>(
    ps: &'a HashMap<String, f32>,
    whitelist: &'a [String],
) -> Element<'a, Message> {
    column![
        text("PRICES").size(24.0),
        scrollable(
            Column::with_children(
                ps.iter()
                    .filter_map(|(n, p)| {
                        if whitelist.contains(n) {
                            Some(
                                button(text(format!("{n} {p}")))
                                    .on_press(Message::AssetSelected(n.clone())),
                            )
                        } else {
                            None
                        }
                    })
                    .map(Element::from)
                    .collect(),
            )
            .width(Length::Fill)
            .spacing(10),
        )
        .id(MESSAGE_LOG.clone())
    ]
    .into()
}
