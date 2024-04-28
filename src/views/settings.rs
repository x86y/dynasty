use crate::{config::Config, message::Message};

use iced::{
    widget::{button, column, container, row, text, text_input, Container},
    Color, Command, Length,
};

use super::panes::orders::tb;

pub(crate) struct SettingsView {
    new_config: Config,
}

#[derive(Debug, Clone)]
pub(crate) enum SettingsMessage {
    /// Write config from settings to disk and memory
    SaveConfig,

    /// on_input events
    ApiKeyInput(String),
    ApiSecretInput(String),
}

impl SettingsView {
    pub(crate) fn new(config: Config) -> Self {
        Self { new_config: config }
    }

    pub(crate) fn update(&mut self, message: SettingsMessage) -> Command<Message> {
        match message {
            SettingsMessage::SaveConfig => {
                let new_config = self.new_config.clone();

                Command::perform(
                    async {
                        new_config.save().map_err(|err| err.to_string())?;

                        Ok(new_config)
                    },
                    Message::ConfigUpdated,
                )
            }
            SettingsMessage::ApiKeyInput(value) => {
                self.new_config.api_key = value;
                Command::none()
            }
            SettingsMessage::ApiSecretInput(value) => {
                self.new_config.api_secret_key = value;
                Command::none()
            }
        }
    }

    pub(crate) fn view(&self) -> Container<'_, Message> {
        let api_key_input = text_input("API Key", &self.new_config.api_key)
            .secure(true)
            .width(Length::Fill)
            .on_input(|s| Message::Settings(SettingsMessage::ApiKeyInput(s)));
        let api_secret_key_input = text_input("API Secret Key", &self.new_config.api_secret_key)
            .secure(true)
            .width(Length::Fill)
            .on_input(|s| Message::Settings(SettingsMessage::ApiSecretInput(s)));

        container(
            column![
                row![text("API Key:").width(Length::Fixed(100.0)), api_key_input].spacing(10),
                row![
                    text("API Secret Key:").width(Length::Fixed(100.0)),
                    api_secret_key_input,
                ]
                .spacing(10),
                button(tb("Save")).on_press(SettingsMessage::SaveConfig.into()),
            ]
            .spacing(10)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(iced::Alignment::Center),
        )
        .style(|_t| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.07, 0.07, 0.07))),
            border: iced::Border {
                radius: 16.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .center_x()
        .center_y()
    }
}
