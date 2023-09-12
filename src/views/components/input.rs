use iced::{widget::text_input, Background, Color};

pub struct Inp;

impl text_input::StyleSheet for Inp {
    type Style = iced::Theme;

    fn active(&self, _: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::from_rgb(0.11, 0.11, 0.11)),
            border_radius: 0.0.into(),
            border_width: 1.0,
            border_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.3,
            },
            icon_color: Default::default(),
        }
    }

    fn focused(&self, _: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::from_rgb(0.5, 0.5, 0.5)),
            border_radius: 0.0.into(),
            border_width: 1.0,
            border_color: Color {
                a: 0.5,
                r: 0.5,
                g: 0.5,
                b: 0.5,
            },
            icon_color: Default::default(),
        }
    }

    fn placeholder_color(&self, _: &Self::Style) -> Color {
        Color::from_rgb(0.3, 0.3, 0.3)
    }

    fn value_color(&self, _: &Self::Style) -> Color {
        Color::WHITE
    }

    fn disabled_color(&self, _: &Self::Style) -> Color {
        Color::from_rgb(0.7, 0.7, 0.7)
    }

    fn selection_color(&self, _: &Self::Style) -> Color {
        Color::from_rgb(0.3, 0.3, 0.3)
    }

    fn disabled(&self, _: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::from_rgb(0.5, 0.5, 0.5)),
            border_radius: 0.0.into(),
            border_width: 1.0,
            border_color: Color {
                a: 0.5,
                r: 0.5,
                g: 0.5,
                b: 0.5,
            },
            icon_color: Default::default(),
        }
    }
}
