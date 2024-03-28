use crate::{views::components::better_btn::GreenBtn, Message};
use iced::{
    widget::{
        button, column, container, text,
        text_editor::{self, Content},
        Column, Space,
    },
    Element, Font, Length,
};
use meval::{Context, Expr};

pub fn calculator_view<'a>(
    content: &'a Content,
    is_editing: bool,
    ctx: &Context,
) -> Element<'a, Message> {
    if is_editing {
        container(
            column![
                text_editor::TextEditor::new(content)
                    .height(Length::Fill)
                    .on_action(Message::CalcAction), // .style(|_theme, _status| text_editor::Appearance {
                //     border: iced::Border {
                //         width: 0.0,
                //         radius: 0.0.into(),
                //         color: Default::default()
                //     },
                //     background: iced::Background::Color(Color::from_rgb(0.07, 0.07, 0.07)),
                // })
                container(
                    button(text("\u{F4F5}").font(Font::with_name("bootstrap-icons")))
                        .style(iced::theme::Button::Custom(Box::new(GreenBtn {})))
                        .on_press(Message::CalcToggle)
                )
                .padding(2)
            ]
            .spacing(10),
        )
    } else {
        container(column![
            Column::with_children(
                content
                    .text()
                    .lines()
                    .map(|line| {
                        text(
                            line.parse::<Expr>()
                                .unwrap()
                                .eval_with_context(ctx)
                                .unwrap_or_default(),
                        )
                    })
                    .map(Element::from)
            ),
            Space::new(Length::Fill, Length::Fill),
            button(text('\u{F4CA}').font(Font::with_name("bootstrap-icons")))
                .style(iced::theme::Button::Custom(Box::new(GreenBtn {})))
                .on_press(Message::CalcToggle)
        ])
    }
    .padding(10)
    .into()
}
