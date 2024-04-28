use crate::{
    data::AppData,
    svg_logos,
    theme::h2c,
    views::{
        components::{loading::loader, unstyled_btn::UnstyledBtn},
        dashboard::DashboardMessage,
    },
};

use iced::{
    widget::{button, container, row, svg, text, Column, Space},
    Element, Length,
};

use super::orders::tb;

pub(crate) struct BalancesPane {}

impl BalancesPane {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn view<'a>(&'a self, data: &'a AppData) -> Element<'a, DashboardMessage> {
        let bs = &data.balances;

        if bs.is_empty() {
            return loader!().into();
        }

        Column::with_children(
            bs.iter()
                .map(|b| {
                    let asset = &b.asset;
                    let ticker = asset.strip_suffix("USDT").unwrap_or(asset);
                    let handle = match svg_logos::LOGOS.get(ticker) {
                        Some(bytes) => svg::Handle::from_memory(*bytes),
                        // this silently fails
                        None => svg::Handle::from_path("NONEXISTENT"),
                    };

                    let svg = svg(handle)
                        .width(Length::Fixed(16.0))
                        .height(Length::Fixed(16.0));
                    container(row![
                        row![
                            svg,
                            button(tb(&b.asset).size(14).style(h2c("B7BDB7").unwrap()))
                                .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {})))
                                .on_press(DashboardMessage::CurrencyPairSelected(b.asset.clone())),
                        ]
                        .spacing(4)
                        .align_items(iced::Alignment::Center),
                        Space::new(Length::Fill, 1.0),
                        button(
                            text(format!("{}", (b.free * 10.0).round() / 10.0))
                                .size(14)
                                .style(h2c("B7BDB7").unwrap())
                        )
                        .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {})))
                        .on_press(DashboardMessage::CurrencyPairSelected(b.asset.clone())),
                    ])
                    .width(Length::Fill)
                })
                .map(Element::from),
        )
        .padding(8)
        .into()
    }
}
