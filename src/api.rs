use std::sync::Arc;

use binance::{
    account::Account,
    api::Binance,
    market::Market,
    rest_model::{OrderSide, OrderStatus},
};
use futures::future::join_all;
use iced::Command;
use regex::Regex;
use tokio::sync::Mutex;

use crate::message::Message;

pub(crate) struct Client {
    binance_account: Arc<Mutex<Account>>,
    binance_market: Arc<Mutex<Market>>,
}

impl Client {
    fn make_client(public: String, secret: String) -> Account {
        Binance::new(Some(public), Some(secret))
    }

    fn make_market(public: String, secret: String) -> Market {
        Binance::new(Some(public), Some(secret))
    }

    pub(crate) fn new(public: String, secret: String) -> Self {
        Self {
            binance_account: Arc::new(Mutex::new(Self::make_client(
                public.clone(),
                secret.clone(),
            ))),
            binance_market: Arc::new(Mutex::new(Self::make_market(public, secret))),
        }
    }

    /// Replace credentials in inner client
    pub(crate) fn swap_credentials(&mut self, public: String, secret: String) -> Command<Message> {
        let binance_account = Arc::clone(&self.binance_account);

        Command::perform(
            async move {
                let mut account = binance_account.lock().await;
                *account = Self::make_client(public, secret);
            },
            |_| Message::CredentialsUpdated,
        )
    }

    pub(crate) fn orders_history(&self) -> Command<Message> {
        let binance_account = Arc::clone(&self.binance_account);

        Command::perform(
            async move {
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
                let mut os: Vec<_> = {
                    let account = binance_account.lock().await;

                    join_all(assets.iter().map(|a: &&str| {
                        account.get_all_orders(binance::account::OrdersQuery {
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
                    .filter(|o| {
                        matches!(o.status, OrderStatus::Filled | OrderStatus::PartiallyFilled)
                    })
                    .collect()
                };

                os.sort_by(|o, n| n.time.cmp(&o.time));
                os
            },
            Message::OrdersRecieved,
        )
    }

    pub(crate) fn balances(&self) -> Command<Message> {
        let binance_account = Arc::clone(&self.binance_account);

        Command::perform(
            async move {
                let assets = ["LINK", "UNI", "ARB", "OP", "SYN", "USDT", "OP"];

                let account = binance_account.lock().await;

                join_all(assets.iter().map(|a| account.get_balance(a.to_string())))
                    .await
                    .into_iter()
                    .flatten()
                    .collect()
            },
            Message::BalancesRecieved,
        )
    }

    pub(crate) fn klines(&self, pair: String, tf: String) -> Command<Message> {
        let market = Arc::clone(&self.binance_market);
        Command::perform(
            async move {
                let acc = market.lock().await;
                acc.get_klines(
                    pair,
                    if tf.is_empty() { "5m" } else { &tf },
                    500,
                    None,
                    None,
                )
                .await
                .unwrap()
            },
            Message::KlinesRecieved,
        )
    }

    pub(crate) fn trade_spot(
        &self,
        pair: String,
        price: f64,
        amt: f64,
        side: OrderSide,
    ) -> Command<Message> {
        let binance_account = Arc::clone(&self.binance_account);

        Command::perform(
            async move {
                binance_account
                    .lock()
                    .await
                    .place_order(binance::account::OrderRequest {
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
            },
            |m| {
                println!("{m:?}");
                Message::MarketChanged("REEEEE".to_string())
            },
        )
    }

    pub(crate) fn split_symbol(symbol: &str) -> Option<(String, String)> {
        let quote_assets = vec![
            "BTC", "ETH", "USDT", "BNB", "TUSD", "PAX", "USDC", "XRP", "USDS", "TRX", "BUSD",
            "NGN", "RUB", "TRY", "EUR", "ZAR", "BKRW", "IDRT", "GBP", "UAH", "BIDR", "AUD", "DAI",
            "BRL", "BVND", "VAI", "USDP", "DOGE", "UST", "DOT", "PLN", "RON", "ARS",
        ];

        let quote_assets_regex = format!("({})", quote_assets.join("|"));
        let regex = Regex::new(&format!(r"^(.+)({})$", quote_assets_regex)).unwrap();

        regex
            .captures(symbol)
            .map(|captures| (captures[1].into(), captures[2].into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_base_qty() {
        let test_cases = vec![
            "BTCUSDT",
            "ETHBTC",
            "XRPUSDC",
            "LTCETH",
            "BNBTUSD",
            "TRXBUSD",
            "ZRXUSDT",
            "INVALIDMARKET",
            "BTCUSDC",
            "ETHUSDT",
            "BNBETH",
            "XRPBTC",
            "LTCUSDT",
            "DOTUSDT",
            "DOGEUSDT",
            "USDTUSD",
            "BTCTUSD",
            "BTCPAX",
            "BTCUSDS",
            "BTCNGN",
            "BTCRUB",
            "BTCTRY",
            "BTCEUR",
            "BTCZAR",
            "BTCBKRW",
            "BTCIDRT",
            "ETHGBP",
            "ETHUAH",
            "ETHBIDR",
            "ETHAUD",
            "ETHDAI",
            "ETHBRL",
            "ETHBVND",
            "USDTDAI",
            "USDCUSDT",
            "USDTBRL",
            "BNBBUSD",
            "BTCBRL",
            "BTCVAI",
            "BUSDUSDT",
            "BTCUSDP",
            "BTCDOT",
            "ETHUST",
            "BTCUST",
            "BTCPLN",
            "BTCRON",
            "BTCARS",
        ];

        for symbol in test_cases {
            match Client::split_symbol(symbol) {
                Some((base, quote)) => {
                    assert_eq!(
                        format!("{}{}", base, quote),
                        symbol,
                        "Split symbol should recombine to the original symbol"
                    );
                }
                None => {
                    assert_eq!(
                        symbol, "INVALIDMARKET",
                        "Invalid market symbol should be INVALIDMARKET"
                    );
                }
            }
        }
    }
}
