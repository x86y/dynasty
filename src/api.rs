use std::env;

use binance::account::Account;
use binance::api::Binance;
use binance::errors::Error;
use binance::rest_model::{Balance, Order, OrderSide, Transaction};
use binance::wallet::Wallet;
use futures::future::join_all;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PUB: Option<String> = env::var_os("DYN_PUB").map(|s| s.into_string().unwrap());
    static ref SEC: Option<String> = env::var_os("DYN_SEC").map(|s| s.into_string().unwrap());
    static ref B: Account = Binance::new(PUB.clone(), SEC.clone());
    static ref W: Wallet = Binance::new(PUB.clone(), SEC.clone());
}

pub async fn orders_history() -> Vec<Order> {
    let now = chrono::offset::Local::now();
    let ago = now.checked_sub_signed(chrono::Duration::weeks(8)).unwrap();
    let assets = ["LINKUSDT", "UNIUSDT", "1INCHUSDT", "OPUSDT", "ARBUSDT"];
    let mut os: Vec<Order> = join_all(assets.iter().map(async move |a| {
        match B
            .get_all_orders(binance::account::OrdersQuery {
                symbol: a.to_string(),
                order_id: None,
                start_time: Some(ago.timestamp_millis() as u64),
                end_time: None,
                limit: None,
                recv_window: None,
            })
            .await
        {
            Ok(r) => r,
            Err(e) => {
                println!("Binance Error: {e}");
                [].to_vec()
            }
        }
    }))
    .await
    .into_iter()
    .flatten()
    .collect();
    os.sort_by(|o, n| n.time.cmp(&o.time));
    os
}

pub async fn trade_spot(
    pair: String,
    price: f64,
    amt: f64,
    side: OrderSide,
) -> Result<Transaction, Error> {
    B.place_order(binance::account::OrderRequest {
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

pub async fn balances() -> Vec<Balance> {
    let assets = ["LINK", "UNI", "ARB", "OP", "SYN", "USDT", "OP"];
    join_all(
        assets
            .iter()
            .map(async move |a| match B.get_balance(a.to_string()).await {
                Ok(r) => r,
                Err(e) => {
                    println!("Binance Error: {e}");
                    Balance {
                        asset: a.to_string(),
                        free: 0.0,
                        locked: 0.0,
                    }
                }
            }),
    )
    .await
    .into_iter()
    .collect()
}
