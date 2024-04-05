use crate::{
    message::Message,
    svg_logos,
    theme::h2c,
    views::{components::unstyled_btn::UnstyledBtn, dashboard::DashboardMessage},
};

use binance::rest_model::Balance;
use iced::{
    widget::{button, container, row, svg, text, Column, Space},
    Element, Length,
};

use super::orders::tb;

pub fn balances_view<'a>(bs: &[Balance]) -> Element<'a, Message> {
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
                            .on_press(DashboardMessage::AssetSelected(b.asset.clone()).into()),
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
                    .on_press(DashboardMessage::AssetSelected(b.asset.clone()).into()),
                ])
                .width(Length::Fill)
            })
            .map(Element::from),
    )
    .padding(8)
    .into()
}
