use binance::account::Account;
use binance::api::Binance;
use binance::errors::Error;
use binance::rest_model::{Balance, Order, OrderSide, OrderStatus, Transaction};
use futures::future::join_all;

pub async fn orders_history(public: String, secret: String) -> Vec<Order> {
    let b: Account = Binance::new(Some(public), Some(secret));
    let now = chrono::offset::Local::now();
    let ago = now
        .checked_sub_signed(chrono::Duration::try_weeks(8).unwrap())
        .unwrap();
    let assets = [
        "LINKUSDT",
        "UNIUSDT",
        "1INCHUSDT",
        "OPUSDT",
        "ARBUSDT",
        "SYNUSDT",
    ];
    let mut os: Vec<Order> = join_all(assets.iter().map(|a: &&str| {
        b.get_all_orders(binance::account::OrdersQuery {
            symbol: a.to_string(),
            order_id: None,
            start_time: Some(ago.timestamp_millis() as u64),
            end_time: None,
            limit: None,
            recv_window: None,
        })
    }))
    .await
    .into_iter()
    .flatten()
    .flatten()
    .filter(|o| matches!(o.status, OrderStatus::Filled | OrderStatus::PartiallyFilled))
    .collect();
    os.sort_by(|o, n| n.time.cmp(&o.time));
    os
}

pub async fn trade_spot(
    public: String,
    secret: String,
    pair: String,
    price: f64,
    amt: f64,
    side: OrderSide,
) -> Result<Transaction, Error> {
    let b: Account = Binance::new(Some(public), Some(secret));
    b.place_order(binance::account::OrderRequest {
        symbol: pair,
        side,
        order_type: binance::rest_model::OrderType::Limit,
        time_in_force: Some(binance::rest_model::TimeInForce::GTC),
        quantity: Some(amt),
        quote_order_qty: None,
        price: Some(price),
        new_client_order_id: None,
        stop_price: None,
        iceberg_qty: None,
        new_order_resp_type: None,
        recv_window: None,
    })
    .await
}

pub async fn balances(public: String, secret: String) -> Vec<Balance> {
    let b: Account = Binance::new(Some(public), Some(secret));
    let assets = ["LINK", "UNI", "ARB", "OP", "SYN", "USDT", "OP"];
    join_all(assets.iter().map(|a| b.get_balance(a.to_string())))
        .await
        .into_iter()
        .flatten()
        .collect()
}
