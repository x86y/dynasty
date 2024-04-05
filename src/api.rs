use std::sync::Arc;

use binance::{
    account::Account,
    api::Binance,
    errors::Error,
    rest_model::{Balance, Order, OrderSide, OrderStatus, Transaction},
};
use futures::future::join_all;
use regex::Regex;
use tokio::sync::Mutex;

pub(crate) struct Client {
    binance_account: Arc<Mutex<Account>>,
}

impl Client {
    fn make_client(public: String, secret: String) -> Account {
        Binance::new(Some(public), Some(secret))
    }

    pub(crate) fn new(public: String, secret: String) -> Self {
        Self {
            binance_account: Arc::new(Mutex::new(Self::make_client(public, secret))),
        }
    }

    pub(crate) async fn swap_credentials(&mut self, public: String, secret: String) {
        let mut account = self.binance_account.lock().await;

        *account = Self::make_client(public, secret)
    }

    pub(crate) async fn orders_history(&self) -> Vec<Order> {
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
            let account = self.binance_account.lock().await;

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
            .filter(|o| matches!(o.status, OrderStatus::Filled | OrderStatus::PartiallyFilled))
            .collect()
        };

        os.sort_by(|o, n| n.time.cmp(&o.time));
        os
    }

    pub(crate) async fn trade_spot(
        &self,
        pair: String,
        price: f64,
        amt: f64,
        side: OrderSide,
    ) -> Result<Transaction, Error> {
        self.binance_account
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
    }

    pub(crate) async fn balances(&self) -> Vec<Balance> {
        let assets = ["LINK", "UNI", "ARB", "OP", "SYN", "USDT", "OP"];

        let account = self.binance_account.lock().await;

        join_all(assets.iter().map(|a| account.get_balance(a.to_string())))
            .await
            .into_iter()
            .flatten()
            .collect()
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
