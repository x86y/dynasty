pub mod balances;
pub mod book;
pub mod market;
pub mod orders;
pub mod trades;
pub mod watchlist;

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
    pub id: PaneType,
    pub is_pinned: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PaneType {
    Prices,
    Book,
    Trades,
    Market,
    Balances,
    Orders,
}

impl From<usize> for PaneType {
    fn from(_value: usize) -> Self {
        Self::Balances
    }
}

impl ToString for PaneType {
    fn to_string(&self) -> String {
        match self {
            PaneType::Prices => "Prices",
            PaneType::Book => "Book",
            PaneType::Trades => "Trades",
            PaneType::Market => "Market",
            PaneType::Balances => "Balances",
            PaneType::Orders => "Orders",
        }
        .to_string()
    }
}

impl Pane {
    pub fn new(ty: PaneType) -> Self {
        Self {
            id: ty,
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
            background: Some(iced::Background::Color(Color::from_rgb(0.07, 0.07, 0.07))),
            border: iced::Border {
                radius: 16.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn title_bar_focused(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            text_color: Some(palette.primary.strong.text),
            background: Some(iced::Background::Color(Color::from_rgb(0.07, 0.07, 0.07))),
            border: iced::Border {
                radius: 16.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

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
