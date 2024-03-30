#[cfg(not(any(feature = "calculator_meval", feature = "calculator_k")))]
compile_error!("no calculator backend selected");

use std::collections::HashMap;

use crate::{app::AppData, message::Message, theme::h2c, views::components::better_btn::GreenBtn};

use binance::rest_model::Order;
use iced::{
    widget::{
        button, column, container, text,
        text_editor::{self, Content},
        Column, Space,
    },
    Alignment, Command, Element, Font, Length,
};

#[cfg(feature = "calculator_k")]
use calc_k::Calculator;

#[cfg(all(feature = "calculator_meval", not(feature = "calculator_k")))]
use calc_meval::Calculator;

pub(crate) struct CalculatorPane {
    calculator: Calculator,
    content: iced::widget::text_editor::Content,
    is_editing: bool,
    eval_results: Vec<String>,
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
            calculator: Calculator::new(),
            content: Content::new(),
            is_editing: true,
            eval_results: Vec::new(),
        }
    }

    pub(crate) fn run(&mut self) {
        self.eval_results = self
            .content
            .text()
            .lines()
            .map(|l| self.calculator.eval(l))
            .collect();
    }

    pub(crate) fn update(
        &mut self,
        data: &AppData,
        message: CalculatorMessage,
    ) -> Command<Message> {
        match message {
            CalculatorMessage::Tick => {
                self.calculator.update_context(&data.prices, &data.orders);

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
                .align_items(Alignment::Center)
                .spacing(10),
            )
        } else {
            container(
                column![
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
                ]
                .align_items(Alignment::Center),
            )
        }
        .padding(10)
        .into()
    }
}

pub(crate) fn order_value(order: &Order, prices: &HashMap<String, f32>) -> f64 {
    let price_now = *prices.get(&order.symbol).unwrap_or(&0.0) as f64;
    let price = order.price;
    let qty = order.executed_qty;
    if order.side == binance::rest_model::OrderSide::Buy {
        qty * (price_now - price)
    } else {
        qty * (price - price_now)
    }
}

#[cfg(feature = "calculator_k")]
mod calc_k {
    use crate::views::panes::calculator::order_value;

    use std::collections::HashMap;

    use binance::rest_model::Order;
    use ngnk::{iK, kinit, K0};

    pub(crate) struct Calculator {}

    impl Calculator {
        pub(crate) fn new() -> Self {
            kinit();

            Self {}
        }

        pub(crate) fn update_context(&mut self, prices: &HashMap<String, f32>, orders: &[Order]) {
            let mut r: String = String::new();

            for key in prices.keys().take(250) {
                let name = key.strip_suffix("USDT").unwrap_or(key);
                if name.is_empty() {
                    continue;
                }

                r.push_str(&format!("`\"{name}\""));
            }
            r.push('!');
            for (key, value) in prices.iter().take(250) {
                let name = key.strip_suffix("USDT").unwrap_or(key);
                if name.is_empty() {
                    continue;
                }

                let f: String = name.chars().filter(|c| c.is_alphabetic()).collect();

                if !f.is_empty() {
                    r.push_str(&format!("{value} "));
                }
            }
            K0(format!("b:{r}"), Vec::new());

            let mut out = String::new();
            for i in 0..orders.len() {
                out.push_str(&format!("`t{i}",));
            }
            out.push('!');
            for trade in orders.iter() {
                out.push_str(&format!("{} ", order_value(trade, prices)));
            }
            K0(format!("d:{out}"), Vec::new());
        }

        pub(crate) fn eval(&self, line: &str) -> String {
            //format!("{:?}", CK(K0(line.to_string(), vec![])))
            format!("{:?}", iK(K0(line.to_string(), vec![])))
        }
    }
}

#[cfg(feature = "calculator_meval")]
mod calc_meval {
    use crate::views::panes::calculator::order_value;

    use std::collections::HashMap;

    use binance::rest_model::Order;
    use meval::{Context, Expr};

    pub(crate) struct Calculator {
        ctx: Context<'static>,
    }

    impl Calculator {
        pub(crate) fn new() -> Self {
            Self {
                ctx: Context::empty(),
            }
        }

        pub(crate) fn update_context(&mut self, prices: &HashMap<String, f32>, orders: &[Order]) {
            for (key, value) in prices.iter() {
                let name = key.strip_suffix("USDT").unwrap_or(key);
                if name.is_empty() {
                    continue;
                }

                self.ctx.var(name, *value as f64);
            }

            for (i, trade) in orders.iter().enumerate() {
                self.ctx.var(format!("t{i}"), order_value(trade, prices));
            }
        }

        pub(crate) fn eval(&self, line: &str) -> String {
            line.parse::<Expr>()
                .unwrap()
                .eval_with_context(&self.ctx)
                .unwrap_or_default()
                .to_string()
        }
    }
}
