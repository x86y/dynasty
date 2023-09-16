/* macro_rules! list {
    ($v: ident, $i: ident, $n: ident) => {
        Column::with_children(
            $v.into_iter()
                .map(|b| column![button(text(format!(
            "{:<8} {:<8}",
            b.$i,
            (b.$n * 10.0).round() / 10.0
        )))
        .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {})))
        .on_press(Message::AssetSelected(b.asset.clone())), horizontal_rule(1.0)].width(150.0))
                .map(Element::from)
                .collect(),
        )
    };
}
pub(crate) use list; */

use iced::{widget::container, Background, Color};

pub struct RowA;
pub struct RowB;

impl container::StyleSheet for RowA {
    type Style = iced::Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: Some(Color::WHITE),
            background: Some(Background::Color(Color::from_rgb(0.15, 0.09, 0.15))),
            border_radius: 0.0.into(),
            border_width: 1.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

impl container::StyleSheet for RowB {
    type Style = iced::Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: Some(Color::WHITE),
            background: Some(Background::Color(Color::from_rgba(0.15, 0.09, 0.15, 0.5))),
            border_radius: 0.0.into(),
            border_width: 1.0,
            border_color: Color::TRANSPARENT,
        }
    }
}
