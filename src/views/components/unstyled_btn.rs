use iced::{widget::button, Color};

/* pub fn unstyled_btn<'a, Message>(content: &'a str, message: Message) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    Element::new(
        iced::widget::Button::new(content)
            .on_press(message)
            .style(iced::theme::Button::Custom(Box::new(UnstyledBtn {}))),
    )
} */

pub struct UnstyledBtn;

impl button::StyleSheet for UnstyledBtn {
    type Style = iced::Theme;
    fn active(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: None,
            text_color: Color::WHITE,
            ..Default::default()
        }
    }
}
