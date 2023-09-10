use binance::rest_model::Balance;
use iced::{
    widget::{button, column, text, Column},
    Element,
};

use crate::Message;

pub fn balances_view(bs: &[Balance]) -> Element<'_, Message> {
    column![
        text("BALANCES").size(24.0),
        Column::with_children(
            bs.iter()
                .map(|b| button(text(format!(
                    "{} {}",
                    b.asset,
                    (b.free * 10.0).round() / 10.0
                )))
                .on_press(Message::AssetSelected(b.asset.clone())))
                .map(Element::from)
                .collect()
        )
    ]
    .into()
}
