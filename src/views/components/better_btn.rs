use iced::{widget::button, Color};

pub struct BetterBtn;

impl button::StyleSheet for BetterBtn {
    type Style = iced::Theme;
    fn active(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.3, 0.3, 0.3, 0.3,
            ))),
            text_color: Color::WHITE,
            shadow_offset: iced::Vector { x: 1.0, y: 1.0 },
            border_radius: 0.0.into(),
            border_width: 1.0,
            border_color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
        }
    }

    fn pressed(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.3, 0.3, 0.3, 0.3,
            ))),
            text_color: Color::WHITE,
            shadow_offset: iced::Vector { x: 1.0, y: 1.0 },
            border_radius: 0.0.into(),
            border_width: 1.0,
            border_color: Color::from_rgb(1.0, 0.0, 0.0),
        }
    }
}
