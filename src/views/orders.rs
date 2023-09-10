use binance::rest_model::Order;
use iced::{
    widget::{column, text, Column},
    Element,
};

use crate::Message;

pub fn orders_view(os: &[Order]) -> Element<'_, Message> {
    column![
        text("ORDERS").size(24.0),
        Column::with_children(
            os.iter()
                .map(|b| text(format!(
                    "{} {} {} {:?} {:?}",
                    b.symbol, b.price, b.executed_qty, b.side, b.status
                )))
                .map(Element::from)
                .collect()
        )
    ]
    .into()
}
