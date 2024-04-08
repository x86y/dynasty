mod api;
mod app;
mod config;
mod message;
mod svg_logos;
mod theme;
mod views;
mod ws;

use crate::config::Config;

use std::env;

use app::App;
use iced::Application;
use iced::{Font, Settings};

fn main() -> iced::Result {
    let config = Config::load()
        .unwrap_or_default();

    App::run(Settings {
        window: iced::window::Settings {
            size: iced::Size {
                width: 1920.0,
                height: 1080.0,
            },
            min_size: Some(iced::Size {
                width: 1280.0,
                height: 720.0,
            }),
            icon: Some(
                iced::window::icon::from_file_data(
                    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/dynasty.png")),
                    Some(iced::advanced::graphics::image::image_rs::ImageFormat::Png),
                )
                .unwrap(),
            ),
            ..Default::default()
        },
        default_font: Font {
            family: iced::font::Family::Name("Iosevka"),
            weight: iced::font::Weight::Normal,
            ..Default::default()
        },
        antialiasing: true,
        flags: config.unwrap_or_default(),
        ..Default::default()
    })
}
