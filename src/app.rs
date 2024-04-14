use crate::api::Client;
use crate::config::Config;
use crate::message::MaybeError;
use crate::message::Message;
use crate::svg_logos;
use crate::views::dashboard::DashboardView;
use crate::views::settings::SettingsView;
use crate::ws::prices;
use crate::ws::user;
use crate::ws::WsEvent;
use crate::ws::WsMessage;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::env;
use std::time::Duration;

use binance::rest_model::KlineSummaries;
use binance::rest_model::OrderStatus;
use binance::rest_model::{Balance, Order};
use binance::ws_model::TradesEvent;
use iced::executor;
use iced::font;
use iced::widget::button;
use iced::widget::scrollable;
use iced::widget::svg;
use iced::widget::Row;
use iced::widget::Space;
use iced::widget::{column, container, row, text};
use iced::{Application, Color, Command, Element, Length, Subscription, Theme};

#[derive(Debug, Default)]
pub(crate) struct AppData {
    pub(crate) prices: Option<HashMap<String, f32>>,
    pub(crate) book: (String, BTreeMap<String, f64>, BTreeMap<String, f64>),
    pub(crate) trades: VecDeque<TradesEvent>,
    pub(crate) balances: Vec<Balance>,
    pub(crate) orders: Vec<Order>,
    pub(crate) quote: String,
}

pub(crate) struct App {
    config: Config,
    data: AppData,
    api: Client,
    errors: Vec<String>,
    settings_opened: bool,
    dashboard: DashboardView,
    settings: SettingsView,
}

impl App {
    fn new(config: Config) -> Self {
        let api = Client::new(config.api_key.clone(), config.api_secret_key.clone());
        App {
            config: config.clone(),
            data: Default::default(),
            api,
            errors: vec![],
            settings_opened: !config.valid(),
            dashboard: DashboardView::new(),
            settings: SettingsView::new(config),
        }
    }

    fn fetch_data(&self) -> Command<Message> {
        Command::batch([
            self.api.orders_history(),
            self.api.balances(),
            self.api.klines(
                if self.data.quote.is_empty() {
                    "BTCUSDT".into()
                } else {
                    self.data.quote.clone()
                },
                String::new(),
            ),
        ])
    }

    fn toggle_settings(&mut self) {
        self.settings_opened = !(self.settings_opened && self.config.valid());
    }
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Flags = Config;
    type Executor = executor::Default;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let app = App::new(flags);
        let fetch_data_cmd = app.fetch_data();

        (
            app,
            Command::batch([
                fetch_data_cmd,
                font::load(
                    include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/assets/fonts/icons.ttf"
                    ))
                    .as_slice(),
                )
                .map(|r| {
                    MaybeError::new("icons.ttf".to_string())
                        .maybe(&r.map_err(|_| "error loading"))
                        .into()
                }),
                font::load(
                    include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/assets/fonts/iosevka.ttf"
                    ))
                    .as_slice(),
                )
                .map(|r| {
                    MaybeError::new("iosevka.ttf".to_string())
                        .maybe(&r.map_err(|_| "error loading"))
                        .into()
                }),
                font::load(
                    include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/assets/fonts/iosevkab.ttc"
                    ))
                    .as_slice(),
                )
                .map(|r| {
                    MaybeError::new("iosevkab.ttc".to_string())
                        .maybe(&r.map_err(|_| "error loading"))
                        .into()
                }),
            ]),
        )
    }

    fn title(&self) -> String {
        String::from("Dynasty")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick => {
                self.dashboard.tick(&self.data);
                Command::none()
            }
            Message::ConfigUpdated(update) => match update {
                Ok(new_config) => {
                    let credentials_updated = self.config.credentials() != new_config.credentials();

                    self.config = new_config;
                    self.toggle_settings();

                    if credentials_updated {
                        self.api.swap_credentials(
                            self.config.api_key.clone(),
                            self.config.api_secret_key.clone(),
                        )
                    } else {
                        Command::none()
                    }
                }
                Err(err) => Command::perform(async {}, move |_| {
                    Message::DispatchErr(("config".to_string(), err.to_string()))
                }),
            },
            Message::CredentialsUpdated => self.fetch_data(),
            Message::Ws(update) => {
                match &update {
                    WsMessage::Book(event) => {
                        if let WsEvent::Message(bt) = event {
                            self.data.book = (bt.sym.clone(), bt.bids.clone(), bt.asks.clone());
                        }
                    }
                    WsMessage::Trade(event) => {
                        if let WsEvent::Message(te) = event {
                            if self.data.trades.len() >= 1000 {
                                self.data.trades.pop_back();
                            }
                            self.data.trades.push_front(te.clone());
                        }
                    }
                    WsMessage::User(u) => {
                        match u {
                            binance::ws_model::WebsocketEvent::AccountPositionUpdate(p) => {
                                for b in p.balances.iter() {
                                    let ib =
                                        self.data.balances.iter_mut().find(|a| a.asset == b.asset);
                                    if let Some(uib) = ib {
                                        *uib = unsafe { std::mem::transmute(b.clone()) }
                                    }
                                }
                            }
                            binance::ws_model::WebsocketEvent::OrderUpdate(o) => {
                                let existing_order = self.data.orders.iter_mut().find(|order| {
                                    // order.client_order_id == o.order_id&&
                                    order.symbol == o.symbol
                                        && order.side == o.side
                                        && order.status == OrderStatus::PartiallyFilled
                                });

                                if let Some(order) = existing_order {
                                    // Update the existing order with the new values
                                    order.executed_qty += o.qty_last_executed;
                                    order.cummulative_quote_qty += o.qty;
                                    order.update_time = o.trade_order_time;
                                } else {
                                    self.data.orders.insert(
                                        0,
                                        Order {
                                            symbol: o.symbol.clone(),
                                            order_id: o.order_id,
                                            order_list_id: o.order_list_id as i32,
                                            client_order_id: o.client_order_id.clone().unwrap(),
                                            price: o.price,
                                            orig_qty: o.qty,
                                            executed_qty: o.qty_last_executed,
                                            cummulative_quote_qty: o.qty,
                                            status: o.current_order_status.clone(),
                                            time_in_force: o.time_in_force.clone(),
                                            order_type: o.order_type.clone(),
                                            side: o.side.clone(),
                                            stop_price: o.stop_price,
                                            iceberg_qty: o.iceberg_qty,
                                            time: o.event_time,
                                            update_time: o.trade_order_time,
                                            is_working: false,
                                            orig_quote_order_qty: o.qty,
                                        },
                                    );
                                }
                            }
                            binance::ws_model::WebsocketEvent::BalanceUpdate(_p) => {
                                // not needed imo?
                            }
                            binance::ws_model::WebsocketEvent::ListOrderUpdate(_lo) => {
                                // not needed imo?
                            }
                            _ => unreachable!(),
                        }
                    }
                    WsMessage::Price(m) => {
                        match m {
                            crate::ws::WsEvent::Connected(_) => {
                                self.data.prices = Some(Default::default())
                            }
                            crate::ws::WsEvent::Disconnected => {
                                self.data.prices.as_mut().map(|prices| prices.clear());
                            }
                            crate::ws::WsEvent::Message(m) => {
                                self.data
                                    .prices
                                    .as_mut()
                                    .expect("websocket connected")
                                    .insert(m.name.clone(), m.price);
                            }
                        };
                    }
                };

                self.dashboard.ws(update)
            }
            Message::OrdersRecieved(orders) => {
                self.data.orders = orders;
                Command::none()
            }
            Message::BalancesRecieved(bals) => {
                self.data.balances = bals;
                Command::none()
            }
            Message::MarketChanged(new_market) => {
                self.data.quote = new_market;
                Command::none()
            }
            Message::DispatchErr((source, message)) => {
                eprintln!("error: {source}: {message}");
                // FIXME: error panel cannot be closed and covers settings button
                // self.errors.push(message);

                Command::none()
            }
            Message::SettingsToggled => {
                self.toggle_settings();

                Command::none()
            }
            Message::Dashboard(msg) => self.dashboard.update(msg, &self.api, &self.data),
            Message::Settings(msg) => self.settings.update(msg),
            Message::NoOp => Command::none(),
            Message::KlinesRecieved(kr) => match kr {
                KlineSummaries::AllKlineSummaries(klines) => {
                    let closes: Vec<f64> = klines.iter().map(|kline| kline.close).collect();
                    self.dashboard.prepend_chart_data(&closes)
                }
            },
            Message::TimeframeChanged(tf) => {
                self.api.klines(self.dashboard.textbox_pair.clone(), tf)
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(Duration::from_millis(1000)).map(|_| Message::Tick),
            prices::connect().map(Message::from),
            user::connect(self.config.api_key.clone()).map(Message::from),
            self.dashboard.subscription(),
            /*
            keyboard::on_key_press(|key_code, modifiers| {
                if !modifiers.command() {
                    return None;
                }
                handle_hotkey(key_code)
            })
            */
        ])
    }

    fn view(&self) -> Element<Message> {
        let header = container(
            row![
                Row::with_children(
                    self.config
                        .watchlist_favorites
                        .iter()
                        .map(|t| {
                            let price_now = if let Some(prices) = &self.data.prices {
                                prices.get(t).unwrap_or(&0.0)
                            } else {
                                &0.0
                            };

                            let ticker = t.strip_suffix("USDT").unwrap_or(t);
                            let handle = match svg_logos::LOGOS.get(ticker) {
                                Some(bytes) => svg::Handle::from_memory(*bytes),
                                // this silently fails
                                None => svg::Handle::from_path("NONEXISTENT"),
                            };

                            let svg = svg(handle)
                                .width(Length::Fixed(16.0))
                                .height(Length::Fixed(16.0));
                            return row![svg, text(format!("{:.2}", price_now)).size(14)]
                                .spacing(4)
                                .align_items(iced::Alignment::Center);
                        })
                        .map(Element::from)
                )
                .spacing(12),
                Space::new(Length::Fill, 1),
                button(text("Settings").size(14))
                    .padding(8)
                    .style(iced::theme::Button::Text)
                    .on_press(Message::SettingsToggled)
            ]
            .align_items(iced::Alignment::Center),
        )
        .padding([0, 16])
        .style(container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.07, 0.07, 0.07))),
            border: iced::Border {
                radius: 16.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let err_header = container(
            row![
                Row::with_children(self.errors.last().map(text).map(Element::from)).spacing(12),
                Space::new(Length::Fill, 1),
                button(text("X").size(14))
                    .padding(8)
                    .style(iced::theme::Button::Text)
                    .on_press(Message::SettingsToggled)
            ]
            .align_items(iced::Alignment::Center),
        )
        .padding([0, 16])
        .style(container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.99, 0.03, 0.03))),
            border: iced::Border {
                radius: 16.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let message_log = scrollable(column![container(
            column![
                if self.errors.is_empty() {
                    header
                } else {
                    err_header
                },
                if self.settings_opened {
                    container(self.settings.view())
                } else {
                    container(self.dashboard.view(&self.data, &self.config))
                }
            ]
            .spacing(8)
        )
        .width(Length::Fill)
        .height(1000.0)
        .padding(10),]);

        container(message_log)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .style(|_: &_| container::Appearance {
                background: Some(iced::Background::Color(Color::BLACK)),
                ..Default::default()
            })
            .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
