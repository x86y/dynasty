use iced::Color;

pub struct ScrollbarStyle;

impl iced::widget::scrollable::StyleSheet for ScrollbarStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::scrollable::Scrollbar {
        iced::widget::scrollable::Scrollbar {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            border_radius: [2.0, 2.0, 2.0, 2.0].into(),
            border_width: 0.5,
            border_color: Color::TRANSPARENT,
            scroller: iced::widget::scrollable::Scroller {
                color: Color::WHITE,
                border_radius: [0.0, 0.0, 0.0, 0.0].into(),
                border_width: 4.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(
        &self,
        _style: &Self::Style,
        _is_mouse_over_scrollbar: bool,
    ) -> iced::widget::scrollable::Scrollbar {
        iced::widget::scrollable::Scrollbar {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            border_radius: [2.0, 2.0, 2.0, 2.0].into(),
            border_width: 0.5,
            border_color: Color::TRANSPARENT,
            scroller: iced::widget::scrollable::Scroller {
                color: Color::WHITE,
                border_radius: [0.0, 0.0, 0.0, 0.0].into(),
                border_width: 4.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }
}

impl ScrollbarStyle {
    pub fn theme() -> iced::theme::Scrollable {
        iced::theme::Scrollable::Custom(Box::from(ScrollbarStyle))
    }
}

