use crate::{message::Message, theme::h2c, views::components::better_btn::GreenBtn};

use iced::{
    widget::{
        button, column, container, text,
        text_editor::{self, Content},
        Column, Space,
    },
    Command, Element, Font, Length,
};
use meval::Context;

pub(crate) struct CalculatorPane {
    content: iced::widget::text_editor::Content,
    pub(crate) is_editing: bool,
    eval_results: Vec<String>,
    pub(crate) ctx: Context<'static>,
}

#[derive(Debug, Clone)]
pub(crate) enum CalculatorMessage {
    Toggle,
    Action(text_editor::Action),
}

impl CalculatorPane {
    pub(crate) fn new() -> Self {
        Self {
            content: Content::new(),
            is_editing: true,
            ctx: Context::empty(),
            eval_results: vec![],
        }
    }

    pub(crate) fn run(&mut self) {
        self.eval_results = self.content.text().lines().map(|l| self.e(l)).collect();
    }

    #[cfg(not(feature = "k"))]
    pub(crate) fn e(&self, line: &str) -> String {
        use meval::Expr;
        format!(
            "{}",
            line.parse::<Expr>()
                .unwrap()
                .eval_with_context(&self.ctx)
                .unwrap_or_default()
        )
    }
    #[cfg(feature = "k")]
    pub(crate) fn e(&self, line: &str) -> String {
        use ngnk::{iK, CK, K0};
        //format!("{:?}", CK(K0(line.to_string(), vec![])))
        format!("{:?}", iK(K0(line.to_string(), vec![])))
    }

    pub(crate) fn update(&mut self, message: CalculatorMessage) -> Command<Message> {
        match message {
            CalculatorMessage::Toggle => {
                self.run();
                self.is_editing = !self.is_editing;

                Command::none()
            }
            CalculatorMessage::Action(action) => {
                self.content.perform(action);
                Command::none()
            }
        }
    }

    pub(crate) fn view(&self) -> Element<'_, Message> {
        if self.is_editing {
            container(
                column![
                    text_editor::TextEditor::new(&self.content)
                        .height(Length::Fill)
                        .on_action(|a| Message::Calculator(CalculatorMessage::Action(a))),
                    container(
                        button(text("\u{F4F5}").font(Font::with_name("bootstrap-icons")))
                            .style(iced::theme::Button::Custom(Box::new(GreenBtn {})))
                            .on_press(Message::Calculator(CalculatorMessage::Toggle))
                    )
                    .padding(2)
                ]
                .spacing(10),
            )
        } else {
            container(column![
                Column::with_children(
                    self.content
                        .text()
                        .lines()
                        .zip(&self.eval_results)
                        .map(|(s, e)| column![
                            text(s)
                                .font(iced::Font {
                                    weight: iced::font::Weight::Bold,
                                    ..Default::default()
                                })
                                .size(14)
                                .style(h2c("EFE1D1").unwrap()),
                            text(e).size(12).style(h2c("EEEEEE").unwrap()),
                        ])
                        .map(Element::from)
                ),
                Space::new(Length::Fill, Length::Fill),
                button(text('\u{F4CA}').font(Font::with_name("bootstrap-icons")))
                    .style(iced::theme::Button::Custom(Box::new(GreenBtn {})))
                    .on_press(Message::Calculator(CalculatorMessage::Toggle))
            ])
        }
        .padding(10)
        .into()
    }
}
