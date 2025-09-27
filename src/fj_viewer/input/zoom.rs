use fj_math::{Transform, Vector};

use crate::fj_viewer::{Camera, FocusPoint};

pub struct Zoom;

impl Zoom {
    pub fn apply(
        zoom_delta: f64,
        focus_point: FocusPoint,
        camera: &mut Camera,
    ) {
        let distance = (focus_point.0 - camera.position()).magnitude();
        let displacement = zoom_delta * distance.into_f64();
        camera.translation = camera.translation
            * Transform::translation(Vector::from([0.0, 0.0, displacement]));
    }
}
