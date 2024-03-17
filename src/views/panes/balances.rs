use std::collections::HashMap;

use binance::rest_model::Balance;
use iced::{
    widget::{button, column, container, row, scrollable, text, Column, Space},
    Element, Length,
};

use crate::{theme::h2c, Message};

use crate::views::components::unstyled_btn::UnstyledBtn;

pub fn balances_view<'a>(bs: &[Balance], ps: &'a HashMap<String, f32>) -> Element<'a, Message> {
    scrollable(Column::with_children(
        bs.iter()
            .map(|b| {
                let price_now = ps.get(&format!("{}USDT", b.asset)).unwrap_or(&0.0);
                container(row![
                    button(
                        text(&b.asset)
                            .font(iced::Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            })
                            .size(14)
                            .style(h2c("EFE1D1").unwrap())
                    )
                    .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {})))
                    .on_press(Message::AssetSelected(b.asset.clone())),
                    Space::new(Length::Fill, 1.0),
                    // button(text("***"))
                    column![
                        button(
                            text(format!("{}", (b.free * 10.0).round() / 10.0))
                                .size(14)
                                .style(h2c("03DAC6").unwrap())
                        )
                        .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {})))
                        .on_press(Message::AssetSelected(b.asset.clone())),
                        text(price_now).size(14)
                    ]
                    .align_items(iced::Alignment::End)
                ])
                .width(Length::Fill)
            })
            .map(Element::from),
    ))
    .into()
}
