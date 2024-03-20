use std::collections::HashMap;

use iced::{
    widget::{
        button, column, container, text,
        text_editor::{self, Content},
        Space,
    },
    Element, Length,
};
use meval::{Context, Expr};

use crate::Message;

pub fn calculator_view<'a>(
    content: &'a Content,
    is_editing: bool,
    ps: &'a HashMap<String, f32>,
) -> Element<'a, Message> {
    let mut ctx = Context::new();
    ctx.var("btc", *ps.get("BTCUSDT").unwrap_or(&0.0) as f64);
    ctx.var("uni", *ps.get("UNIUSDT").unwrap_or(&0.0) as f64);
    if is_editing {
        container(
            column![
                text_editor::TextEditor::new(content)
                    .height(Length::Fill)
                    .on_action(Message::CalcAction),
                container(button("run").on_press(Message::CalcToggle)).padding(2)
            ]
            .spacing(10),
        )
    } else {
        container(
            column![
                text(
                    content
                        .text()
                        .parse::<Expr>()
                        .unwrap()
                        .eval_with_context(ctx)
                        .unwrap_or_default()
                ),
                Space::new(Length::Fill, 1.0),
                button("edit").on_press(Message::CalcToggle)
            ]
            .spacing(10),
        )
    }
    .padding(10)
    .into()
}
