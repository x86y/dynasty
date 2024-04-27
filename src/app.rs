use crate::api::Client;
use crate::config::Config;
use crate::data::AppData;
use crate::message::MaybeError;
use crate::message::Message;
use crate::svg_logos;
use crate::views::dashboard::DashboardView;
use crate::views::settings::SettingsView;
use crate::ws::Websockets;

use std::env;
use std::time::Duration;

use binance::rest_model::KlineSummaries;
use iced::executor;
use iced::font;
use iced::widget::button;
use iced::widget::scrollable;
use iced::widget::svg;
use iced::widget::Row;
use iced::widget::Space;
use iced::widget::{column, container, row, text};
use iced::{Application, Color, Command, Element, Length, Subscription, Theme};

pub(crate) struct App {
    config: Config,
    data: AppData,
    api: Client,
    errors: Vec<String>,
    settings_opened: bool,
    dashboard: DashboardView,
    settings: SettingsView,
    ws: Websockets,
}

impl App {
    fn new(config: Config) -> Self {
        let api = Client::new(config.api_key.clone(), config.api_secret_key.clone());
        App {
            config: config.clone(),
            data: Default::default(),
            api,
            errors: Vec::new(),
            settings_opened: !config.complete(),
            dashboard: DashboardView::new(),
            ws: Websockets::new(config.api_key.clone(), "BTCUSDT"),
            settings: SettingsView::new(config),
        }
    }

    fn fetch_data(&self) -> Command<Message> {
        Command::batch([
            self.api.orders_history(
                vec![
                    "LINKUSDT",
                    "UNIUSDT",
                    "1INCHUSDT",
                    "OPUSDT",
                    "ARBUSDT",
                    "SYNUSDT",
                ]
                .into_iter()
                .map(ToOwned::to_owned)
                .collect(),
            ),
            self.api.balances(
                vec!["LINK", "UNI", "ARB", "OP", "SYN", "USDT", "OP"]
                    .into_iter()
                    .map(ToOwned::to_owned)
                    .collect(),
            ),
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
        self.settings_opened = !(self.settings_opened && self.config.complete());
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
                        self.api.update_credentials(
                            self.config.api_key.clone(),
                            self.config.api_secret_key.clone(),
                        );
                        self.ws.relogin_user(&self.config.api_key);
                        self.fetch_data()
                    } else {
                        Command::none()
                    }
                }
                Err(err) => Command::perform(async {}, move |_| {
                    Message::DispatchErr(("config".to_string(), err.to_string()))
                }),
            },
            Message::Ws(msg) => {
                self.ws.update(msg, &mut self.data, &mut self.dashboard);
                Command::none()
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
                tracing::error!("error: {source}: {message}");
                // FIXME: error panel cannot be closed and covers settings button
                // self.errors.push(message);

                Command::none()
            }
            Message::SettingsToggled => {
                self.toggle_settings();

                Command::none()
            }
            Message::Dashboard(msg) => self.dashboard.update(msg, &self.api, &self.data, &self.ws),
            Message::Settings(msg) => self.settings.update(msg),
            Message::NoOp => Command::none(),
            Message::KlinesRecieved(kr) => match kr {
                KlineSummaries::AllKlineSummaries(klines) => {
                    let closes = klines.iter().map(|kline| kline.close);
                    self.dashboard.prepend_chart_data(closes).map(Message::from)
                }
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(Duration::from_millis(1000)).map(|_| Message::Tick),
            self.ws.subscription(),
            self.dashboard.subscription(&self.data),
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
                            let price_now = &self.data.prices.get(t).unwrap_or(&0.0);

                            let ticker = t.strip_suffix("USDT").unwrap_or(t);
                            let handle = match svg_logos::LOGOS.get(ticker) {
                                Some(bytes) => svg::Handle::from_memory(*bytes),
                                // this silently fails
                                None => svg::Handle::from_path("NONEXISTENT"),
                            };

                            let svg = svg(handle)
                                .width(Length::Fixed(16.0))
                                .height(Length::Fixed(16.0));
                            return row![svg, text(format!("{price_now:.2}")).size(14)]
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
                    container(
                        self.dashboard
                            .view(&self.data, &self.config)
                            .map(Message::from),
                    )
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
