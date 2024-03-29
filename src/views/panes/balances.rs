use crate::theme::h2c;
use crate::views::components::unstyled_btn::UnstyledBtn;
use crate::views::panes::Message;

use binance::rest_model::Balance;
use iced::{
    widget::{button, container, row, scrollable, svg, text, Column, Space},
    Element, Length,
};

pub fn balances_view<'a>(bs: &[Balance]) -> Element<'a, Message> {
    scrollable(
        Column::with_children(
            bs.iter()
                .map(|b| {
                    let ticker = &b.asset.split("USDT").next().unwrap();
                    let handle = svg::Handle::from_path(format!(
                        "{}/assets/logos/{}.svg",
                        env!("CARGO_MANIFEST_DIR"),
                        ticker
                    ));

                    let svg = svg(handle)
                        .width(Length::Fixed(16.0))
                        .height(Length::Fixed(16.0));
                    container(row![
                        row![
                            svg,
                            button(
                                text(&b.asset)
                                    .font(iced::Font {
                                        weight: iced::font::Weight::Bold,
                                        ..Default::default()
                                    })
                                    .size(14)
                                    .style(h2c("B7BDB7").unwrap())
                            )
                            .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {})))
                            .on_press(Message::AssetSelected(b.asset.clone())),
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
                        .on_press(Message::AssetSelected(b.asset.clone())),
                    ])
                    .width(Length::Fill)
                })
                .map(Element::from),
        )
        .padding(8),
    )
    .into()
}
