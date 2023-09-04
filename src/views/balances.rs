use binance::rest_model::Balance;
use iced::{Element, widget::{text, column, Column}};

use crate::Message;

pub fn balances_view(bs: &[Balance]) -> Element<'_, Message> {
    column![
        text("BALANCES").size(24.0),
        Column::with_children(bs.iter().map(|b| text(format!("{} {}", b.asset, b.free))).map(Element::from).collect())
    ].into()
}
