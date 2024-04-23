//! Loading wdiget

use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::gradient;
use iced::widget::canvas::stroke::{self, Stroke};
use iced::widget::canvas::{Geometry, Path};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme, Vector};

use std::time::Instant;

use crate::views::dashboard::DashboardMessage;

#[derive(Default, Debug)]
pub struct Loader {
    pub state: State,
}

impl Loader {
    pub fn view(&self) -> Element<DashboardMessage> {
        canvas(&self.state)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[derive(Debug)]
pub struct State {
    space_cache: canvas::Cache,
    system_cache: canvas::Cache,
    start: Instant,
    now: Instant,
}

impl State {
    const SUN_RADIUS: f32 = 35.0;
    const ORBIT_RADIUS: f32 = 75.0;
    const EARTH_RADIUS: f32 = 6.0;
    const MOON_RADIUS: f32 = 2.0;
    const MOON_DISTANCE: f32 = 14.0;

    pub fn new() -> State {
        let now = Instant::now();

        State {
            space_cache: canvas::Cache::default(),
            system_cache: canvas::Cache::default(),
            start: now,
            now,
        }
    }

    pub fn update(&mut self, now: Instant) {
        self.now = now;
        self.system_cache.clear();
    }
}

impl<Message> canvas::Program<Message> for State {
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

        let system = self.system_cache.draw(renderer, bounds.size(), |frame| {
            let center = frame.center();

            let sun = Path::circle(center, Self::SUN_RADIUS);
            let orbit = Path::circle(center, Self::ORBIT_RADIUS);

            frame.fill(&sun, Color::from_rgb8(0xF9, 0xD7, 0x1C));
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

            let elapsed = self.now - self.start;
            let rotation = (2.0 * PI / 60.0) * elapsed.as_secs() as f32
                + (2.0 * PI / 60_000.0) * elapsed.subsec_millis() as f32;

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

                frame.with_save(|frame| {
                    frame.rotate(rotation * 10.0);
                    frame.translate(Vector::new(0.0, Self::MOON_DISTANCE));

                    let moon = Path::circle(Point::ORIGIN, Self::MOON_RADIUS);
                    frame.fill(&moon, Color::WHITE);
                });
            });
        });

        vec![system]
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
