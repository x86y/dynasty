//! Loading wdiget based on loading spinner iced example

use iced::advanced::layout;
use iced::advanced::renderer;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{self, Clipboard, Layout, Shell, Widget};
use iced::event;
use iced::mouse;
use iced::widget::canvas;
use iced::window::{self, RedrawRequest};
use iced::{Background, Color, Element, Event, Length, Radians, Rectangle, Renderer, Size, Vector};

use easing::Easing;

use std::f32::consts::PI;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;

const MIN_ANGLE: Radians = Radians(PI / 8.0);
const WRAP_ANGLE: Radians = Radians(2.0 * PI - PI / 4.0);
const BASE_ROTATION_SPEED: u32 = u32::MAX / 80;

static STANDARD_EASING: OnceLock<Easing> = OnceLock::new();

pub(crate) struct Loader<Theme>
where
    Theme: StyleSheet,
{
    size: f32,
    bar_height: f32,
    style: <Theme as StyleSheet>::Style,
    cycle_duration: Duration,
    rotation_duration: Duration,
}

// workaround for broken Loader::view()
macro_rules! loader {
    () => {
        iced::widget::container(crate::views::components::loading::Loader::new())
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center_x()
            .center_y()
    };
}

pub(crate) use loader;

impl<Theme> Loader<Theme>
where
    Theme: StyleSheet,
{
    pub(crate) fn new() -> Self {
        Self {
            size: 40.0,
            bar_height: 4.0,
            style: <Theme as StyleSheet>::Style::default(),
            cycle_duration: Duration::from_millis(600),
            rotation_duration: Duration::from_secs(2),
        }
    }

    // TODO: figure out why types break down when container() is used inside this method but work
    //       with macro
    // pub(crate) fn view<'a>(self) -> Element<'a, DashboardMessage, Theme> {
    //     let s: Element<'a, DashboardMessage, Theme> = self.into();
    //     container(s)
    //         .width(Length::Fill)
    //         .height(Length::Fill)
    //         .center_x()
    //         .center_y()
    //         .into()
    // }
}

#[derive(Clone, Copy)]
enum Animation {
    Expanding {
        start: Instant,
        progress: f32,
        rotation: u32,
        last: Instant,
    },
    Contracting {
        start: Instant,
        progress: f32,
        rotation: u32,
        last: Instant,
    },
}

impl Default for Animation {
    fn default() -> Self {
        Self::Expanding {
            start: Instant::now(),
            progress: 0.0,
            rotation: 0,
            last: Instant::now(),
        }
    }
}

impl Animation {
    fn next(&self, additional_rotation: u32, now: Instant) -> Self {
        match self {
            Self::Expanding { rotation, .. } => Self::Contracting {
                start: now,
                progress: 0.0,
                rotation: rotation.wrapping_add(additional_rotation),
                last: now,
            },
            Self::Contracting { rotation, .. } => {
                Self::Expanding {
                    start: now,
                    progress: 0.0,
                    rotation: rotation.wrapping_add(BASE_ROTATION_SPEED.wrapping_add(
                        (f64::from(WRAP_ANGLE / (2.0 * Radians::PI)) * f64::MAX) as u32,
                    )),
                    last: now,
                }
            }
        }
    }

    fn start(&self) -> Instant {
        match self {
            Self::Expanding { start, .. } | Self::Contracting { start, .. } => *start,
        }
    }

    fn last(&self) -> Instant {
        match self {
            Self::Expanding { last, .. } | Self::Contracting { last, .. } => *last,
        }
    }

    fn timed_transition(
        &self,
        cycle_duration: Duration,
        rotation_duration: Duration,
        now: Instant,
    ) -> Self {
        let elapsed = now.duration_since(self.start());
        let additional_rotation = ((now - self.last()).as_secs_f32()
            / rotation_duration.as_secs_f32()
            * (u32::MAX) as f32) as u32;

        match elapsed {
            elapsed if elapsed > cycle_duration => self.next(additional_rotation, now),
            _ => self.with_elapsed(cycle_duration, additional_rotation, elapsed, now),
        }
    }

    fn with_elapsed(
        &self,
        cycle_duration: Duration,
        additional_rotation: u32,
        elapsed: Duration,
        now: Instant,
    ) -> Self {
        let progress = elapsed.as_secs_f32() / cycle_duration.as_secs_f32();
        match self {
            Self::Expanding {
                start, rotation, ..
            } => Self::Expanding {
                start: *start,
                progress,
                rotation: rotation.wrapping_add(additional_rotation),
                last: now,
            },
            Self::Contracting {
                start, rotation, ..
            } => Self::Contracting {
                start: *start,
                progress,
                rotation: rotation.wrapping_add(additional_rotation),
                last: now,
            },
        }
    }

    fn rotation(&self) -> f32 {
        match self {
            Self::Expanding { rotation, .. } | Self::Contracting { rotation, .. } => {
                *rotation as f32 / u32::MAX as f32
            }
        }
    }
}

#[derive(Default)]
struct State {
    animation: Animation,
    cache: canvas::Cache,
}

impl<'a, Message, Theme> Widget<Message, Theme, Renderer> for Loader<Theme>
where
    Message: 'a + Clone,
    Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fixed(self.size),
            height: Length::Fixed(self.size),
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.size, self.size)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let state = tree.state.downcast_mut::<State>();

        if let Event::Window(_, window::Event::RedrawRequested(now)) = event {
            state.animation =
                state
                    .animation
                    .timed_transition(self.cycle_duration, self.rotation_duration, now);

            state.cache.clear();
            shell.request_redraw(RedrawRequest::NextFrame);
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        use advanced::Renderer as _;

        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let custom_style = <Theme as StyleSheet>::appearance(theme, &self.style);

        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let track_radius = frame.width() / 2.0 - self.bar_height;
            let track_path = canvas::Path::circle(frame.center(), track_radius);

            frame.stroke(
                &track_path,
                canvas::Stroke::default()
                    .with_color(custom_style.track_color)
                    .with_width(self.bar_height),
            );

            let mut builder = canvas::path::Builder::new();

            let start = Radians(state.animation.rotation() * 2.0 * PI);

            let easing = STANDARD_EASING.get_or_init(|| {
                Easing::builder()
                    .cubic_bezier_to([0.2, 0.0], [0.0, 1.0], [1.0, 1.0])
                    .build()
            });

            match state.animation {
                Animation::Expanding { progress, .. } => {
                    builder.arc(canvas::path::Arc {
                        center: frame.center(),
                        radius: track_radius,
                        start_angle: start,
                        end_angle: start + MIN_ANGLE + WRAP_ANGLE * (easing.y_at_x(progress)),
                    });
                }
                Animation::Contracting { progress, .. } => {
                    builder.arc(canvas::path::Arc {
                        center: frame.center(),
                        radius: track_radius,
                        start_angle: start + WRAP_ANGLE * (easing.y_at_x(progress)),
                        end_angle: start + MIN_ANGLE + WRAP_ANGLE,
                    });
                }
            }

            let bar_path = builder.build();

            frame.stroke(
                &bar_path,
                canvas::Stroke::default()
                    .with_color(custom_style.bar_color)
                    .with_width(self.bar_height),
            );
        });

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            use iced::advanced::graphics::geometry::Renderer as _;

            renderer.draw_geometry(geometry);
        });
    }
}

impl<'a, Message, Theme> From<Loader<Theme>> for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: StyleSheet + 'a,
{
    fn from(circular: Loader<Theme>) -> Self {
        Self::new(circular)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The [`Background`] of the progress indicator.
    pub background: Option<Background>,
    /// The track [`Color`] of the progress indicator.
    pub track_color: Color,
    /// The bar [`Color`] of the progress indicator.
    pub bar_color: Color,
}

impl std::default::Default for Appearance {
    fn default() -> Self {
        Self {
            background: None,
            track_color: Color::TRANSPARENT,
            bar_color: Color::BLACK,
        }
    }
}

/// A set of rules that dictate the style of an indicator.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the active [`Appearance`] of a indicator.
    fn appearance(&self, style: &Self::Style) -> Appearance;
}

impl StyleSheet for iced::Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        let palette = self.extended_palette();

        Appearance {
            background: None,
            track_color: palette.background.weak.color,
            bar_color: palette.primary.base.color,
        }
    }
}

mod easing {
    use iced::Point;

    use lyon_algorithms::measure::PathMeasurements;
    use lyon_algorithms::path::{builder::NoAttributes, path::BuilderImpl, Path};

    pub struct Easing {
        path: Path,
        measurements: PathMeasurements,
    }

    impl Easing {
        pub fn builder() -> Builder {
            Builder::new()
        }

        pub fn y_at_x(&self, x: f32) -> f32 {
            let mut sampler = self
                .measurements
                .create_sampler(&self.path, lyon_algorithms::measure::SampleType::Normalized);
            let sample = sampler.sample(x);

            sample.position().y
        }
    }

    pub struct Builder(NoAttributes<BuilderImpl>);

    impl Builder {
        pub fn new() -> Self {
            let mut builder = Path::builder();
            builder.begin(lyon_algorithms::geom::point(0.0, 0.0));

            Self(builder)
        }

        /// Adds a cubic bézier curve. Points must be between 0,0 and 1,1
        pub fn cubic_bezier_to(
            mut self,
            ctrl1: impl Into<Point>,
            ctrl2: impl Into<Point>,
            to: impl Into<Point>,
        ) -> Self {
            self.0
                .cubic_bezier_to(Self::point(ctrl1), Self::point(ctrl2), Self::point(to));

            self
        }

        pub fn build(mut self) -> Easing {
            self.0.line_to(lyon_algorithms::geom::point(1.0, 1.0));
            self.0.end(false);

            let path = self.0.build();
            let measurements = PathMeasurements::from_path(&path, 0.0);

            Easing { path, measurements }
        }

        fn point(p: impl Into<Point>) -> lyon_algorithms::geom::Point<f32> {
            let p: Point = p.into();
            lyon_algorithms::geom::point(p.x.min(1.0).max(0.0), p.y.min(1.0).max(0.0))
        }
    }

    impl Default for Builder {
        fn default() -> Self {
            Self::new()
        }
    }
}
