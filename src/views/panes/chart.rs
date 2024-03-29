use crate::{config::Config, message::Message};
use plotters::style::colors;
use plotters::style::Color;
use plotters::style::IntoFont;
use plotters::{
    coord::{types::RangedCoordf32, ReverseCoordTranslate},
    prelude::*,
};
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

use iced::{
    widget::{container, text, Container},
    Command, Length,
};

#[derive(Debug, Clone)]
pub(crate) enum ChartMessage {}

pub struct ChartPane {}

impl Chart<Message> for ChartPane {
    type State = ();
    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        const POINT_COLOR: RGBColor = colors::WHITE;
        const LINE_COLOR: RGBColor = colors::RED;
        const HOVER_COLOR: RGBColor = colors::YELLOW;
        const PREVIEW_COLOR: RGBColor = colors::GREEN;

        let mut chart = builder
            .x_label_area_size(28_i32)
            .y_label_area_size(28_i32)
            .margin(20_i32)
            .build_cartesian_2d(0_f32..10_f32, 0_f32..10_f32)
            .expect("Failed to build chart");

        chart
            .configure_mesh()
            .disable_mesh()
            .bold_line_style(plotters::style::colors::full_palette::GREY_600)
            .light_line_style(plotters::style::colors::full_palette::GREY_800)
            .axis_style(
                ShapeStyle::from(plotters::style::colors::full_palette::GREY_500).stroke_width(0),
            )
            .y_max_light_lines(2)
            .y_labels(6)
            .y_label_style(
                ("monospace", 12)
                    .into_font()
                    .color(&plotters::style::colors::WHITE)
                    .transform(FontTransform::Rotate90),
            )
            .x_max_light_lines(30)
            .x_labels(3)
            .draw()
            .unwrap();

        chart
            .draw_series(
                AreaSeries::new(
                    vec![
                        (1.0, 1.0),
                        (2.0, 2.0),
                        (3.0, 3.0),
                        (4.0, 4.0),
                        (5.0, 5.0),
                        (6.0, 6.0),
                        (7.0, 7.0),
                        (8.0, 8.0),
                        (9.0, 9.0),
                    ]
                    .iter()
                    .map(|x| (x.0, x.1)),
                    0.0,
                    LINE_COLOR,
                )
                .border_style(ShapeStyle::from(LINE_COLOR).stroke_width(1)),
            )
            .expect("failed to draw chart data");
    }
}

impl ChartPane {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn update(&mut self, message: ChartMessage) -> Command<Message> {
        Command::none()
    }

    pub(crate) fn view(&self) -> Container<'_, Message> {
        container(ChartWidget::new(self))
            .style(container::Appearance {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.07, 0.07, 0.07,
                ))),
                border: iced::Border {
                    radius: 16.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .padding(2)
            .center_x()
            .center_y()
    }
}
