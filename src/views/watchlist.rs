use std::collections::HashMap;
use iced::{Element, widget::{text, column, scrollable, Column}, Length};
use crate::{Message, MESSAGE_LOG};

pub fn watchlist_view<'a>(ps: &'a HashMap<String, f32>, whitelist: &'a [String]) -> Element<'a, Message> {
    column![
        text("PRICES").size(24.0),
        scrollable(
                Column::with_children(
                    ps
                        .iter()
                        .filter_map(|(n, p)| {
                            if whitelist.contains(n) {
                                Some(text(format!("{n} {p}")))
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
    ].into()
}

