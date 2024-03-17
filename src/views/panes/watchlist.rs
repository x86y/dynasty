use crate::{theme::h2c, Message};
use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Column, Space},
    Element, Font, Length,
};
use std::collections::HashMap;

use crate::views::components::{better_btn::BetterBtn, input::Inp, unstyled_btn::UnstyledBtn};

#[derive(Debug, Clone, Copy)]
pub enum WatchlistFilter {
    Favorites,
    Eth,
    Btc,
    Alts,
}

pub fn watchlist_view<'a>(
    ps: &'a HashMap<String, f32>,
    favorites: &'a [String],
    filter: WatchlistFilter,
    search: &'a str,
) -> Element<'a, Message> {
    let mut sorted_assets: Vec<_> = ps.iter().collect();
    sorted_assets.sort_by(|(_, p1), (_, p2)| p2.partial_cmp(p1).unwrap());

    column![
        row![
            button(text("\u{F588}").font(Font::with_name("bootstrap-icons")))
                .padding(8)
                .style(iced::theme::Button::Custom(Box::new(BetterBtn {})))
                .on_press(Message::ApplyWatchlistFilter(WatchlistFilter::Favorites)),
            button("BTC")
                .padding(8)
                .style(iced::theme::Button::Text)
                .on_press(Message::ApplyWatchlistFilter(WatchlistFilter::Btc)),
            button("ETH")
                .padding(8)
                .style(iced::theme::Button::Text)
                .on_press(Message::ApplyWatchlistFilter(WatchlistFilter::Eth)),
            button("ALTS")
                .padding(8)
                .style(iced::theme::Button::Text)
                .on_press(Message::ApplyWatchlistFilter(WatchlistFilter::Alts)),
            text_input("type to filter", search)
                .on_input(Message::WatchlistFilterInput)
                .style(iced::theme::TextInput::Custom(Box::new(Inp {})))
        ]
        .spacing(2.0),
        scrollable(
            Column::with_children(
                sorted_assets
                    .iter()
                    .filter_map(|i| {
                        if !search.is_empty() {
                            i.0.contains(&search.to_uppercase()).then_some((i.0, i.1))
                        } else {
                            match filter {
                                WatchlistFilter::Favorites => favorites.contains(i.0),
                                WatchlistFilter::Eth => i.0.contains("ETH"),
                                WatchlistFilter::Btc => i.0.contains("BTC"),
                                WatchlistFilter::Alts => true,
                            }
                            .then_some((i.0, i.1))
                        }
                    })
                    .map(|(n, p)| {
                        container(row![
                            button(text(n).style(h2c("EFE1D1").unwrap()))
                                .on_press(Message::AssetSelected(n.to_string()))
                                .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {}))),
                            Space::new(Length::Fill, 1.0),
                            button(text(format!["{p} "]).style(h2c("B7BDB76").unwrap()))
                                .on_press(Message::AssetSelected(n.to_string()))
                                .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {}))),
                        ])
                        .width(Length::Fill)
                    })
                    .map(Element::from)
            )
            .padding(8)
        ) //.style(ScrollbarStyle::theme())
    ]
    .align_items(iced::Alignment::Start)
    .into()
}
