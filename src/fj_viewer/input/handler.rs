use super::{movement::Movement, rotation::Rotation, zoom::Zoom, InputEvent};
use crate::fj_viewer::{Camera, FocusPoint};

/// Input handling abstraction
///
/// Takes user input and applies them to application state.
#[derive(Default)]
pub struct InputHandler;

impl InputHandler {
    /// Handle an input event
    pub fn handle_event(
        event: InputEvent,
        focus_point: FocusPoint,
        camera: &mut Camera,
    ) {
        match event {
            InputEvent::Translation { previous, current } => {
                Movement::apply(previous, current, focus_point, camera);
            }
            InputEvent::Rotation { angle_x, angle_y } => {
                Rotation::apply(angle_x, angle_y, focus_point, camera);
            }
            InputEvent::Zoom(zoom_delta) => {
                Zoom::apply(zoom_delta, focus_point, camera);
            }
        }
    }
}
