// use iced::Color;
//
// pub struct ScrollbarStyle;
//
// impl iced::widget::scrollable::StyleSheet for ScrollbarStyle {
//     type Style = iced::Theme;
//
//     fn active(&self, _style: &Self::Style) -> iced::widget::scrollable::Appearance {
//         iced::widget::scrollable::Appearance {
//             container: iced::widget::container::Appearance {
//                 background: Some(iced::Background::Color(Color::TRANSPARENT)),
//                 border: iced::Border {
//                     radius: [2.0, 2.0, 2.0, 2.0].into(),
//                     width: 0.5,
//                     color: Color::TRANSPARENT,
//                 },
//                 ..Default::default()
//             },
//             scrollbar: iced::widget::scrollable::Scrollbar {
//                 background: Some(iced::Background::Color(Color::WHITE)),
//                 scroller: iced::widget::scrollable::Scroller {
//                     color: Color::WHITE,
//                     border: iced::Border {
//                         radius: [0.0, 0.0, 0.0, 0.0].into(),
//                         width: 4.0,
//                         color: Color::TRANSPARENT,
//                     },
//                 },
//                 border: iced::Border {
//                     radius: [0.0, 0.0, 0.0, 0.0].into(),
//                     width: 4.0,
//                     color: Color::TRANSPARENT,
//                 },
//             },
//             gap: Some(iced::Background::Color(Color::TRANSPARENT)),
//         }
//     }
//
//     fn hovered(
//         &self,
//         _style: &Self::Style,
//         _is_mouse_over_scrollbar: bool,
//     ) -> iced::widget::scrollable::Appearance {
//         iced::widget::scrollable::Appearance {
//             container: iced::widget::container::Appearance {
//                 background: Some(iced::Background::Color(Color::TRANSPARENT)),
//                 border: iced::Border {
//                     radius: [2.0, 2.0, 2.0, 2.0].into(),
//                     width: 0.5,
//                     color: Color::TRANSPARENT,
//                 },
//                 ..Default::default()
//             },
//             scrollbar: iced::widget::scrollable::Scrollbar {
//                 background: Some(iced::Background::Color(Color::WHITE)),
//                 scroller: iced::widget::scrollable::Scroller {
//                     color: Color::WHITE,
//                     border: iced::Border {
//                         radius: [0.0, 0.0, 0.0, 0.0].into(),
//                         width: 4.0,
//                         color: Color::TRANSPARENT,
//                     },
//                 },
//                 border: iced::Border {
//                     radius: [0.0, 0.0, 0.0, 0.0].into(),
//                     width: 4.0,
//                     color: Color::TRANSPARENT,
//                 },
//             },
//             gap: Some(iced::Background::Color(Color::TRANSPARENT)),
//         }
//     }
// }
//
// impl ScrollbarStyle {
//     pub fn theme() -> iced::theme::Scrollable {
//         iced::theme::Scrollable::Custom(Box::from(ScrollbarStyle))
//     }
// }
