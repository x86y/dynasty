use crate::config::Config;
use crate::data::{AppData, PriceFilter};
use crate::theme::h2c;
use crate::views::components::loading::loader;
use crate::views::components::{better_btn::BetterBtn, input::Inp, unstyled_btn::UnstyledBtn};
use crate::views::dashboard::DashboardMessage;
use iced::Command;
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
    ($label:expr, $filter:expr, $current_filter:expr) => {
        button($label)
            .padding(8)
            .style(if $filter == $current_filter {
                iced::theme::Button::Custom(Box::new(BetterBtn {}))
            } else {
                iced::theme::Button::Text
            })
            .on_press(DashboardMessage::Watchlist(WatchlistMessage::ApplyFilter(
                $filter,
                $filter == $current_filter,
            )))
    };
}

fn asset_button<'a>(n: &str, p: f32) -> Element<'a, DashboardMessage> {
    container(row![
        button(tb(n).size(14).style(h2c("EFE1D1").unwrap()))
            .on_press(DashboardMessage::CurrencyPairSelected(n.to_string()))
            .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {}))),
        Space::new(Length::Fill, 1.0),
        button(
            text(format!("{p} "))
                .size(14)
                .style(h2c("B7BDB76").unwrap())
        )
        .on_press(DashboardMessage::CurrencyPairSelected(n.to_string()))
        .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {}))),
    ])
    .width(Length::Fill)
    .into()
}

#[derive(Debug, Clone)]
pub(crate) enum WatchlistMessage {
    FilterInput(String),
    ApplyFilter(WatchlistFilter, bool),
}

pub(crate) struct WatchlistPane {
    filter: WatchlistFilter,
    filter_string: String,
}

impl WatchlistPane {
    pub(crate) fn new() -> Self {
        Self {
            filter: WatchlistFilter::Favorites,
            filter_string: String::new(),
        }
    }

    pub(crate) fn view<'a>(&'a self, data: &'a AppData) -> Element<'a, DashboardMessage> {
        if data.prices.is_empty() {
            return loader!().into();
        };

        column![
            row![
                filter_button!(
                    text("\u{F588}").font(Font::with_name("bootstrap-icons")),
                    WatchlistFilter::Favorites,
                    self.filter
                ),
                filter_button!("BTC", WatchlistFilter::Btc, self.filter),
                filter_button!("ETH", WatchlistFilter::Eth, self.filter),
                filter_button!("ALTS", WatchlistFilter::Alts, self.filter),
                text_input("type to filter", &self.filter_string)
                    .on_input(|i| WatchlistMessage::FilterInput(i).into())
                    .style(iced::theme::TextInput::Custom(Box::new(Inp {})))
            ]
            .spacing(2.0),
            scrollable(
                Column::with_children(
                    data.prices
                        .sorted_and_filtered()
                        .map(|(n, p)| asset_button(n, *p))
                        .map(Element::from)
                )
                .padding(8)
            )
        ]
        .align_items(iced::Alignment::Start)
        .into()
    }

    pub(crate) fn update(
        &mut self,
        msg: WatchlistMessage,
        data: &mut AppData,
        config: &Config,
    ) -> Command<WatchlistMessage> {
        match msg {
            WatchlistMessage::ApplyFilter(f, clicked_again) => {
                if clicked_again {
                    data.prices.flip_sort();
                } else {
                    data.prices.reset_sort();
                    let filter = match f {
                        WatchlistFilter::Favorites => {
                            PriceFilter::Matches(config.watchlist_favorites.clone())
                        }
                        WatchlistFilter::Eth => PriceFilter::Contains("ETH".to_owned()),
                        WatchlistFilter::Btc => PriceFilter::Contains("BTC".to_owned()),
                        WatchlistFilter::Alts => PriceFilter::All,
                    };
                    data.prices.set_filter(filter);
                    self.filter = f;
                }

                Command::none()
            }
            WatchlistMessage::FilterInput(s) => {
                self.filter_string = s.to_uppercase();
                data.prices
                    .set_filter(PriceFilter::Contains(self.filter_string.clone()));

                Command::none()
            }
        }
    }
}
