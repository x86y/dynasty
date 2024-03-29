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
        const LINE_COLOR: RGBColor = colors::WHITE;
        const HOVER_COLOR: RGBColor = colors::YELLOW;
        const PREVIEW_COLOR: RGBColor = colors::GREEN;

        let mut chart = builder
            .x_label_area_size(28_i32)
            .y_label_area_size(28_i32)
            .margin(20_i32)
            .build_cartesian_2d(0_f32..100_f32, 0_f32..100_f32)
            .expect("Failed to build chart");

        chart
            .configure_mesh()
            .bold_line_style(colors::WHITE.mix(0.1))
            .light_line_style(colors::WHITE.mix(0.05))
            .axis_style(ShapeStyle::from(colors::TRANSPARENT).stroke_width(0))
            .y_labels(10)
            .y_label_style(
                ("sans-serif", 15)
                    .into_font()
                    .color(&colors::BLACK.mix(0.65))
                    .transform(FontTransform::Rotate90),
            )
            .y_label_formatter(&|y| format!("{}", y))
            .draw()
            .expect("Failed to draw chart mesh");

        /* chart
        .draw_series(
            vec![&(1.0, 1.0), &(2.0, 2.0), &(3.0, 3.0)]
                .iter()
                .map(|p| Circle::new(*p, 5_i32, POINT_COLOR.filled())),
        )
        .expect("Failed to draw points"); */

        for line in vec![(1.0, 1.0), (2.0, 2.0), (3.0, 3.0)].iter() {
            chart
                .draw_series(LineSeries::new(
                    vec![(line.0, line.1)].into_iter(),
                    LINE_COLOR.filled(),
                ))
                .expect("Failed to draw line");
        }
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
        container(
            ChartWidget::new(self)
                .width(Length::Fixed(200.0))
                .height(Length::Fixed(200.0)),
        )
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
        .center_x()
        .center_y()
    }
}
