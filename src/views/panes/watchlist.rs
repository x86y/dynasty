use crate::theme::h2c;
use crate::views::components::loading::Loader;
use crate::views::components::{better_btn::BetterBtn, input::Inp, unstyled_btn::UnstyledBtn};
use crate::views::dashboard::DashboardMessage;
use ahash::AHashMap;
use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Column, Space},
    Element, Font, Length,
};

use super::orders::tb;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WatchlistFilter {
    Favorites,
    Eth,
    Btc,
    Alts,
}

macro_rules! filter_button {
    ($label:expr, $filter:expr, $current_filter:expr, $message:expr) => {
        button($label)
            .padding(8)
            .style(if $filter == $current_filter {
                iced::theme::Button::Custom(Box::new(BetterBtn {}))
            } else {
                iced::theme::Button::Text
            })
            .on_press($message)
    };
}

fn asset_button<'a>(n: &str, p: f32) -> Element<'a, DashboardMessage> {
    container(row![
        button(tb(n).size(14).style(h2c("EFE1D1").unwrap()))
            .on_press(DashboardMessage::AssetSelected(n.to_string()))
            .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {}))),
        Space::new(Length::Fill, 1.0),
        button(
            text(format!("{p} "))
                .size(14)
                .style(h2c("B7BDB76").unwrap())
        )
        .on_press(DashboardMessage::AssetSelected(n.to_string()))
        .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {}))),
    ])
    .width(Length::Fill)
    .into()
}

pub fn watchlist_view<'a>(
    ps: &'a AHashMap<String, f32>,
    favorites: &'a [String],
    filter: WatchlistFilter,
    search: &'a str,
    loader: &'a Loader,
) -> Element<'a, DashboardMessage> {
    if ps.is_empty() {
        return loader.view();
    };

    let mut sorted_assets: Vec<_> = ps.iter().collect();
    sorted_assets.sort_by(|(_, p1), (_, p2)| p2.partial_cmp(p1).unwrap());

    column![
        row![
            filter_button!(
                text("\u{F588}").font(Font::with_name("bootstrap-icons")),
                WatchlistFilter::Favorites,
                filter,
                DashboardMessage::ApplyWatchlistFilter(WatchlistFilter::Favorites)
            ),
            filter_button!(
                "BTC",
                WatchlistFilter::Btc,
                filter,
                DashboardMessage::ApplyWatchlistFilter(WatchlistFilter::Btc)
            ),
            filter_button!(
                "ETH",
                WatchlistFilter::Eth,
                filter,
                DashboardMessage::ApplyWatchlistFilter(WatchlistFilter::Eth)
            ),
            filter_button!(
                "ALTS",
                WatchlistFilter::Alts,
                filter,
                DashboardMessage::ApplyWatchlistFilter(WatchlistFilter::Alts)
            ),
            text_input("type to filter", search)
                .on_input(DashboardMessage::WatchlistFilterInput)
                .style(iced::theme::TextInput::Custom(Box::new(Inp {})))
        ]
        .spacing(2.0),
        scrollable(
            Column::with_children(
                sorted_assets
                    .iter()
                    .filter_map(|i| {
                        if search.is_empty() {
                            match filter {
                                WatchlistFilter::Favorites => favorites.contains(i.0),
                                WatchlistFilter::Eth => i.0.contains("ETH"),
                                WatchlistFilter::Btc => i.0.contains("BTC"),
                                WatchlistFilter::Alts => true,
                            }
                            .then_some((i.0, i.1))
                        } else {
                            i.0.contains(&search.to_uppercase()).then_some((i.0, i.1))
                        }
                    })
                    .map(|(n, p)| asset_button(n, *p))
                    .map(Element::from)
            )
            .padding(8)
        )
    ]
    .align_items(iced::Alignment::Start)
    .into()
}
