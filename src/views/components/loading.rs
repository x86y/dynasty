//! Loading wdiget based on solar system iced example

use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::gradient;
use iced::widget::canvas::stroke::{self, Stroke};
use iced::widget::canvas::{Geometry, Path};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme, Vector};

use std::time::Instant;

use crate::views::dashboard::DashboardMessage;

#[derive(Debug)]
pub struct Loader {
    system_cache: canvas::Cache,
    start: Instant,
}

impl Loader {
    const ORBIT_RADIUS: f32 = 20.0;
    const EARTH_RADIUS: f32 = 6.0;

    pub(crate) fn new() -> Self {
        Self {
            system_cache: canvas::Cache::default(),
            start: Instant::now(),
        }
    }

    pub(crate) fn view(&self) -> Element<DashboardMessage> {
        canvas(self).width(Length::Fill).height(Length::Fill).into()
    }
}

impl<Message> canvas::Program<Message> for Loader {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        use std::f32::consts::PI;

        self.system_cache.clear();
        let system = self.system_cache.draw(renderer, bounds.size(), |frame| {
            let center = frame.center();
            let orbit = Path::circle(center, Self::ORBIT_RADIUS);

            frame.stroke(
                &orbit,
                Stroke {
                    style: stroke::Style::Solid(Color::from_rgba8(0, 153, 255, 0.1)),
                    width: 1.0,
                    line_dash: canvas::LineDash {
                        offset: 0,
                        segments: &[3.0, 6.0],
                    },
                    ..Stroke::default()
                },
            );

            let elapsed = self.start.elapsed();
            let rotation = (2.0 * PI / 3.0) * elapsed.as_secs() as f32
                + (2.0 * PI / 3_000.0) * elapsed.subsec_millis() as f32;

            frame.with_save(|frame| {
                frame.translate(Vector::new(center.x, center.y));
                frame.rotate(rotation);
                frame.translate(Vector::new(Self::ORBIT_RADIUS, 0.0));

                let earth = Path::circle(Point::ORIGIN, Self::EARTH_RADIUS);

                let earth_fill = gradient::Linear::new(
                    Point::new(-Self::EARTH_RADIUS, 0.0),
                    Point::new(Self::EARTH_RADIUS, 0.0),
                )
                .add_stop(0.2, Color::from_rgb(0.15, 0.50, 1.0))
                .add_stop(0.8, Color::from_rgb(0.0, 0.20, 0.47));

                frame.fill(&earth, earth_fill);
            });
        });

        vec![system]
    }
}
