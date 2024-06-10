use iced::{widget::button, Color};

pub fn unstyled_btn() -> button::Style {
    button::Style {
        background: None,
        text_color: Color::WHITE,
        ..Default::default()
    }
}

pub fn better_btn() -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.3, 0.3, 0.3, 0.3,
        ))),
        text_color: Color::WHITE,
        border: iced::Border {
            radius: 4.0.into(),
            width: 1.0,
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
        },
        ..Default::default()
    }
}

/* fn pressed(&self, _: &Self::Style) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.3, 0.3, 0.3, 0.3,
        ))),
        text_color: Color::WHITE,
        border: iced::Border {
            radius: 0.0.into(),
            width: 1.0,
            color: Color::from_rgb(1.0, 0.0, 0.0),
        },
        ..Default::default()
    }
} */

pub fn green_btn() -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            50.0 / 255.0,
            217.0 / 255.0,
            147.0 / 255.0,
            1.0,
        ))),
        text_color: Color::WHITE,
        border: iced::Border {
            radius: 4.0.into(),
            width: 1.0,
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
        },
        ..Default::default()
    }
}

pub fn red_btn() -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            1.0,
            112.0 / 255.0,
            126.0 / 255.0,
            1.0,
        ))),
        text_color: Color::WHITE,
        border: iced::Border {
            radius: 4.0.into(),
            width: 1.0,
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
        },
        ..Default::default()
    }
}

/* fn pressed(&self, _: &Self::Style) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.3, 0.3, 0.3, 0.3,
        ))),
        text_color: Color::WHITE,
        border: iced::Border {
            radius: 0.0.into(),
            width: 1.0,
            color: Color::from_rgb(1.0, 0.0, 0.0),
        },
        ..Default::default()
    }
} */

/* fn pressed(&self, _: &Self::Style) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.3, 0.3, 0.3, 0.3,
        ))),
        text_color: Color::WHITE,
        border: iced::Border {
            radius: 0.0.into(),
            width: 1.0,
            color: Color::from_rgb(1.0, 0.0, 0.0),
        },
        ..Default::default()
    }
} */
