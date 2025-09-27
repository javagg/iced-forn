use fj_math::{Transform, Vector};

use crate::fj_viewer::{Camera, FocusPoint};

pub struct Rotation;

impl Rotation {
    pub fn apply(
        angle_x: f64,
        angle_y: f64,
        focus_point: FocusPoint,
        camera: &mut Camera,
    ) {
        let rotate_around = Transform::translation(focus_point.0.coords);

        // the model rotates not the camera, so invert the transform
        let camera_rotation = camera.rotation.inverse();
        let right_vector = right_vector(&camera_rotation);
        let up_vector = up_vector(&camera_rotation);

        let rotation = Transform::rotation(right_vector * angle_x)
            * Transform::rotation(up_vector * angle_y);

        let transform = camera.camera_to_model()
            * rotate_around
            * rotation
            * rotate_around.inverse();

        camera.rotation = transform.extract_rotation();
        camera.translation = transform.extract_translation();
    }
}

fn up_vector(rotation: &Transform) -> Vector<3> {
    let d = rotation.data();
    Vector::from([d[4], d[5], d[6]])
}

fn right_vector(rotation: &Transform) -> Vector<3> {
    let d = rotation.data();
    Vector::from([d[0], d[1], d[2]])
}
