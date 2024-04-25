#[cfg(not(any(feature = "calculator_meval", feature = "calculator_k")))]
compile_error!("no calculator backend selected");

use crate::{data::AppData, theme::h2c, views::components::better_btn::GreenBtn};

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

use super::orders::tb;

pub(crate) struct CalculatorPane {
    calculator: Calculator,
    content: iced::widget::text_editor::Content,
    is_editing: bool,
    eval_results: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum CalculatorPaneMessage {
    Toggle,
    Action(text_editor::Action),
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
        message: CalculatorPaneMessage,
    ) -> Command<CalculatorPaneMessage> {
        match message {
            CalculatorPaneMessage::Toggle => {
                self.run();
                self.is_editing = !self.is_editing;

                Command::none()
            }
            CalculatorPaneMessage::Action(action) => {
                self.content.perform(action);
                Command::none()
            }
        }
    }

    pub(crate) fn tick(&mut self, data: &AppData) {
        self.calculator.update_context(data);

        if !self.is_editing {
            self.run();
        }
    }

    pub(crate) fn view(&self) -> Element<'_, CalculatorPaneMessage> {
        if self.is_editing {
            container(
                column![
                    text_editor::TextEditor::new(&self.content)
                        .height(Length::Fill)
                        .on_action(CalculatorPaneMessage::Action),
                    container(
                        button(text("\u{F4F5}").font(Font::with_name("bootstrap-icons")))
                            .style(iced::theme::Button::Custom(Box::new(GreenBtn {})))
                            .on_press(CalculatorPaneMessage::Toggle)
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
                                tb(s).size(18).style(h2c("EFE1D1").unwrap()),
                                text(e).size(18).style(h2c("EEEEEE").unwrap()),
                            ])
                            .map(Element::from)
                    ),
                    Space::new(Length::Fill, Length::Fill),
                    button(text('\u{F4CA}').font(Font::with_name("bootstrap-icons")))
                        .style(iced::theme::Button::Custom(Box::new(GreenBtn {})))
                        .on_press(CalculatorPaneMessage::Toggle)
                ]
                .align_items(Alignment::Center),
            )
        }
        .padding(10)
        .into()
    }
}

pub(crate) fn order_value(order: &Order, price_now: f64) -> f64 {
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
    use crate::{api::Client, data::AppData, views::panes::calculator::order_value};

    use ngnk::{kinit, CK, K0};

    pub(crate) struct Calculator {}

    impl Calculator {
        pub(crate) fn new() -> Self {
            kinit();

            Self {}
        }

        pub(crate) fn update_context(&mut self, data: &AppData) {
            let mut keys = String::new();
            let mut values = String::new();
            for (key, val) in data.prices.descending().take(250) {
                if let Some([base, _]) = Client::split_symbol(key) {
                    let filtered: String = base.chars().filter(|c| c.is_alphabetic()).collect();
                    if !filtered.is_empty() {
                        keys.push_str(&format!("`\"{filtered}\""));
                        values.push_str(&format!("{val} "));
                    }
                }
            }
            K0(format!("PRICES:({keys}! {values})"), Vec::new());
            keys.clear();
            values.clear();

            for (i, trade) in data.orders.iter().enumerate() {
                keys.push_str(&format!("`t{} ", i));
                values.push_str(&format!(
                    "{} ",
                    order_value(trade, data.prices.price(&trade.symbol) as f64)
                ));
            }
            K0(format!("ORDERS:({keys}!{values})"), Vec::new());
        }

        pub(crate) fn eval(&self, line: &str) -> String {
            //format!("{:?}", CK(K0(line.to_string(), vec![])))
            let payload = format!(".[{{`k@{line}}};[];{{(\"Error in K code\")}}]");
            CK(K0(payload, vec![]))
        }
    }
}

#[cfg(feature = "calculator_meval")]
mod calc_meval {
    use crate::{data::AppData, views::panes::calculator::order_value};

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

        pub(crate) fn update_context(&mut self, data: &AppData) {
            for (key, value) in data.prices.descending() {
                let name = key.strip_suffix("USDT").unwrap_or(key);
                if name.is_empty() {
                    continue;
                }

                self.ctx.var(name, *value as f64);
            }

            for (i, trade) in data.orders.iter().enumerate() {
                self.ctx.var(
                    format!("t{i}"),
                    order_value(trade, data.prices.price(&trade.symbol) as f64),
                );
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
