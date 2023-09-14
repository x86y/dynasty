use iced::{
    theme,
    widget::{button, pane_grid, row, text},
    Color, Element, Font,
};
use iced_futures::core::text::LineHeight;

use crate::Message;

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

#[derive(Clone, Copy)]
pub struct Pane {
    pub id: usize,
    pub is_pinned: bool,
}

impl Pane {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            is_pinned: false,
        }
    }
}

pub fn view_controls<'a>(
    pane: pane_grid::Pane,
    total_panes: usize,
    is_pinned: bool,
    is_maximized: bool,
) -> Element<'a, Message> {
    let mut row = row![].spacing(5);

    if total_panes > 1 {
        let toggle = {
            let (content, message) = if is_maximized {
                (
                    text('\u{f149}'.to_string())
                        .font(Font::with_name("bootstrap-icons"))
                        .line_height(LineHeight::Relative(1.1)),
                    Message::Restore,
                )
            } else {
                (
                    text('\u{f14a}'.to_string())
                        .font(Font::with_name("bootstrap-icons"))
                        .line_height(LineHeight::Relative(1.1)),
                    Message::Maximize(pane),
                )
            };
            button(content.size(14))
                .style(theme::Button::Secondary)
                .padding(3)
                .on_press(message)
        };

        row = row.push(toggle);
    }

    let mut close = button(
        text('\u{f659}'.to_string())
            .font(Font::with_name("bootstrap-icons"))
            .line_height(LineHeight::Relative(1.1))
            .size(14),
    )
    .style(theme::Button::Destructive)
    .padding(3);

    if total_panes > 1 && !is_pinned {
        close = close.on_press(Message::Close(pane));
    }

    row.push(close).height(20).into()
}

pub const PANE_ID_COLOR_UNFOCUSED: Color = Color::from_rgb(
    0xFF as f32 / 255.0,
    0xC7 as f32 / 255.0,
    0xC7 as f32 / 255.0,
);
pub const PANE_ID_COLOR_FOCUSED: Color = Color::from_rgb(
    0xFF as f32 / 255.0,
    0x47 as f32 / 255.0,
    0x47 as f32 / 255.0,
);

pub mod style {
    use iced::widget::container;
    use iced::{Color, Theme};

    pub fn title_bar_active(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            text_color: Some(palette.background.strong.text),
            background: Some(iced::Background::Color(Color::from_rgba(
                0.3, 0.3, 0.3, 0.9,
            ))),
            ..Default::default()
        }
    }

    pub fn title_bar_focused(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            text_color: Some(palette.primary.strong.text),
            background: Some(iced::Background::Color(Color::from_rgba(
                0.3, 0.3, 0.3, 0.9,
            ))),
            ..Default::default()
        }
    }

    pub fn pane_active(_: &Theme) -> container::Appearance {
        container::Appearance {
            border_width: 1.0,
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            ..Default::default()
        }
    }

    pub fn pane_focused(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            border_width: 1.0,
            border_color: palette.primary.strong.color,
            ..Default::default()
        }
    }
}
