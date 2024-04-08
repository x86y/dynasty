pub(crate) mod balances;
pub(crate) mod book;
pub(crate) mod calculator;
pub(crate) mod chart;
pub(crate) mod market;
pub(crate) mod orders;
pub(crate) mod trades;
pub(crate) mod watchlist;

/* pub fn handle_hotkey(key_code: keyboard::KeyCode) -> Option<Message> {
    use keyboard::KeyCode;
    use pane_grid::{Axis, Direction};

    let direction = match key_code {
        KeyCode::Up => Some(Direction::Up),
        KeyCode::Down => Some(Direction::Down),
        KeyCode::Left => Some(Direction::Left),
        KeyCode::Right => Some(Direction::Right),
        _ => None,
    };

    match key_code {
        KeyCode::V => Some(Message::SplitFocused(Axis::Vertical)),
        KeyCode::H => Some(Message::SplitFocused(Axis::Horizontal)),
        KeyCode::W => Some(Message::CloseFocused),
        _ => direction.map(Message::FocusAdjacent),
    }
} */

pub mod style {
    use iced::widget::container;
    use iced::{Color, Theme};

    pub fn pane_active(_: &Theme) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.07, 0.07, 0.07))),
            border: iced::Border {
                width: 0.0,
                radius: 16.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn pane_focused(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.07, 0.07, 0.07))),
            border: iced::Border {
                width: 0.0,
                radius: 16.0.into(),
                color: palette.primary.strong.color,
            },
            ..Default::default()
        }
    }
}
