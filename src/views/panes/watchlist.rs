use crate::data::AppData;
use crate::theme::h2c;
use crate::views::components::loading::Loader;
use crate::views::components::{better_btn::BetterBtn, input::Inp, unstyled_btn::UnstyledBtn};
use crate::views::dashboard::DashboardMessage;
use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Column, Space},
    Element, Font, Length,
};

use super::orders::tb;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum WatchlistFilter {
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
    data: &'a AppData,
    filter: WatchlistFilter,
    search: &'a str,
    loader: &'a Loader,
) -> Element<'a, DashboardMessage> {
    if data.prices.is_empty() {
        return loader.view();
    };

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
                data.prices
                    .descending_filtered()
                    .map(|(n, p)| asset_button(n, *p))
                    .map(Element::from)
            )
            .padding(8)
        )
    ]
    .align_items(iced::Alignment::Start)
    .into()
}
