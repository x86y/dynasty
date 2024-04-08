use crate::message::Message;
use plotters::prelude::*;
use plotters::style::colors;
use plotters::style::IntoFont;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

use iced::widget::{container, Container};
use ringbuf::Rb;

#[derive(Debug, Clone)]
pub(crate) enum ChartMessage {}

pub struct ChartPane {
    pub data: ringbuf::HeapRb<f64>
}

impl Chart<Message> for ChartPane {
    type State = ();
    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        const LINE_COLOR: RGBColor = colors::GREEN;

        let (min, max) = self.data.iter().fold((f32::MAX, f32::MIN), |acc, &x| {
            (acc.0.min(x as f32), acc.1.max(x as f32))
        });

        let mut chart = builder
            .x_label_area_size(0_i32)
            .y_label_area_size(0_i32)
            .margin(0_i32)
            .build_cartesian_2d(0..self.data.len(), min..max)
            .expect("Failed to build chart");

        chart
            .configure_mesh()
            .disable_mesh()
            .bold_line_style(plotters::style::colors::full_palette::GREY_600.stroke_width(3))
            .light_line_style(plotters::style::colors::full_palette::GREY_800.stroke_width(3))
            .axis_style(
                ShapeStyle::from(plotters::style::colors::full_palette::GREY_500).stroke_width(0),
            )
            .y_max_light_lines(2)
            .y_labels(2)
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
            .draw_series(LineSeries::new(
                self.data.iter().enumerate().map(|(x, y)| (x, (*y as f32))),
                LINE_COLOR,
            ))
            .expect("failed to draw chart data");
    }
}

impl ChartPane {
    pub(crate) fn new() -> Self {
        Self { data: ringbuf::HeapRb::new(500) }
    }

    pub(crate) fn update_data(&mut self, new_p: f64) {
        let _ = self.data.push(new_p);
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
