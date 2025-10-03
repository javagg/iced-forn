mod model;

mod fjviewer;
use fj_core::algorithms::approx::Tolerance;
use fj_core::algorithms::bounding_volume::BoundingVolume;
use fj_core::algorithms::triangulate::Triangulate;
use fj_core::objects::{Region, Sketch};
use fj_core::operations::build::{BuildRegion, BuildSketch};
use fj_core::operations::sweep::SweepSketch;
use fj_core::operations::update::UpdateSketch;
use fj_math::{Aabb, Point, Scalar};

use iced::widget::{center, column, shader};
use iced::Length::{self, Fill};
use iced::{Center, Element};

use model::Program;

fn main() -> iced::Result {
    iced::application(
        "Custom Shader - Iced",
        App::update,
        App::view,
    )
    .run()
}

struct App;

#[derive(Debug, Clone)]
enum Message {}

impl App {
    fn new() -> Self {
        Self {}
    }

    fn update(&mut self, message: Message) {
    }

    fn view(&self) -> Element<'_, Message> {
        let [x, y, z] = [3.0, 2.0, 1.0];
        let mut core = fj_core::Core::new();
        let bottom_surface = core.layers.objects.surfaces.xy_plane();
        let sweep_path = fj_math::Vector::from([fj_math::Scalar::ZERO, fj_math::Scalar::ZERO, (-z).into()]);
        let model = Sketch::empty()
            .add_regions(
                [Region::polygon(
                    [
                        [-x / 2., -y / 2.],
                        [x / 2., -y / 2.],
                        [x / 2., y / 2.],
                        [-x / 2., y / 2.],
                    ],
                    &mut core,
                )],
                &mut core,
            )
            .sweep_sketch(bottom_surface, sweep_path, &mut core);

        core.layers
            .validation
            .take_errors()
            .expect("Model is invalid");
        let aabb = model.aabb(&core.layers.geometry).unwrap_or(Aabb {
            min: Point::origin(),
            max: Point::origin(),
        });

        let mut min_extent = Scalar::MAX;
        for extent in aabb.size().components {
            if extent > Scalar::ZERO && extent < min_extent {
                min_extent = extent;
            }
        }

        let tolerance = min_extent / Scalar::from_f64(1000.);
        let tolerance = Tolerance::from_scalar(tolerance).unwrap();

        let mesh = (&model, tolerance).triangulate(&mut core);
        let m = fj_interop::Model { mesh, aabb };
        center(column![
            "Text1",
            "Text2",
            shader(Program::new(m)).width(Length::Fill).height(Length::Fill),
            "Text3",].align_x(Center)).into()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}