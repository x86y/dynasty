use crate::{app::AppData, message::Message, theme::h2c, views::components::better_btn::GreenBtn};

use iced::{
    widget::{
        button, column, container, text,
        text_editor::{self, Content},
        Column, Space,
    },
    Command, Element, Font, Length,
};
use meval::Context;

#[cfg(feature = "k")]
use ngnk::{iK, K0};

pub(crate) struct CalculatorPane {
    calculator: Calculator,
    content: iced::widget::text_editor::Content,
    is_editing: bool,
    eval_results: Vec<String>,
    ctx: Context<'static>,
}

#[derive(Debug, Clone)]
pub(crate) enum CalculatorMessage {
    Toggle,
    Action(text_editor::Action),
    Tick,
}

impl CalculatorPane {
    pub(crate) fn new() -> Self {
        Self {
            calculator: Calculator {},
            content: Content::new(),
            is_editing: true,
            ctx: Context::empty(),
            eval_results: Vec::new(),
        }
    }

    pub(crate) fn run(&mut self) {
        self.eval_results = self
            .content
            .text()
            .lines()
            .map(|l| self.calculator.eval(l, &self.ctx))
            .collect();
    }

    pub(crate) fn update(
        &mut self,
        data: &AppData,
        message: CalculatorMessage,
    ) -> Command<Message> {
        match message {
            CalculatorMessage::Tick => {
                let mut r: String = String::new();

                for key in data.prices.keys().take(250) {
                    let name = key.strip_suffix("USDT").unwrap_or(key);
                    if name.is_empty() {
                        continue;
                    }

                    r.push_str(&format!("`\"{name}\""));
                }
                r.push('!');
                for (key, value) in data.prices.iter().take(250) {
                    let name = key.strip_suffix("USDT").unwrap_or(key);
                    if name.is_empty() {
                        continue;
                    }

                    self.ctx.var(name, *value as f64);
                    let f: String = name.chars().filter(|c| c.is_alphabetic()).collect();

                    if !f.is_empty() {
                        r.push_str(&format!("{value} "));
                    }
                }
                #[cfg(feature = "k")]
                K0(format!("b:{r}"), Vec::new());

                let mut out = String::new();
                for (i, _) in data.orders.iter().enumerate() {
                    out.push_str(&format!("`t{i}",));
                }
                out.push('!');
                for (i, trade) in data.orders.iter().enumerate() {
                    let price_now = *data.prices.get(&trade.symbol).unwrap_or(&0.0) as f64;
                    let price = trade.price;
                    let qty = trade.executed_qty;
                    let pnl_value = if trade.side == binance::rest_model::OrderSide::Buy {
                        qty * (price_now - price)
                    } else {
                        qty * (price - price_now)
                    };
                    self.ctx.var(format!("t{i}"), pnl_value);
                    out.push_str(&format!("{pnl_value} "));
                }
                #[cfg(feature = "k")]
                K0(format!("d:{out}"), Vec::new());
                if !self.is_editing {
                    self.run();
                }
                Command::none()
            }
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
                            .on_press(CalculatorMessage::Toggle.into())
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
                    .on_press(CalculatorMessage::Toggle.into())
            ])
        }
        .padding(10)
        .into()
    }
}

struct Calculator {}

impl Calculator {
    #[cfg(not(feature = "k"))]
    pub(crate) fn eval(&self, line: &str, ctx: &Context) -> String {
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
    pub(crate) fn eval(&self, line: &str, _ctx: &Context) -> String {
        //format!("{:?}", CK(K0(line.to_string(), vec![])))
        format!("{:?}", iK(K0(line.to_string(), vec![])))
    }
}
