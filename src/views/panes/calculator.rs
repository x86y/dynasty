use crate::{theme::h2c, views::components::better_btn::GreenBtn, Message};
use iced::{
    widget::{
        button, column, container, text,
        text_editor::{self, Content},
        Column, Space,
    },
    Element, Font, Length,
};
use meval::Context;

#[cfg(not(feature = "k"))]
pub fn e(line: &str, ctx: &Context) -> String {
    use meval::Expr;
    format!(
        "{}",
        line.parse::<Expr>()
            .unwrap()
            .eval_with_context(ctx)
            .unwrap_or_default()
    )
}
#[cfg(feature = "k")]
pub fn e(line: &str, _ctx: &Context) -> String {
    use ngnk::{iK, CK, K0};
    //format!("{:?}", CK(K0(line.to_string(), vec![])))
    format!("{:?}", iK(K0(line.to_string(), vec![])))
}

pub fn calculator_view<'a>(
    content: &'a Content,
    results: &[String],
    is_editing: bool,
) -> Element<'a, Message> {
    if is_editing {
        container(
            column![
                text_editor::TextEditor::new(content)
                    .height(Length::Fill)
                    .on_action(Message::CalcAction),
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
                    .zip(results)
                    .map(|(s, e)| column![
                        text(s)
                            .font(iced::Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            })
                            .size(14)
                            .style(h2c("EFE1D1").unwrap()),
                        text(e)
                            .size(12)
                            .style(h2c("EEEEEE").unwrap()),
                    ])
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
