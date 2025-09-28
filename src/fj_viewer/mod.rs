//! Viewer camera module
use std::{f64::consts::FRAC_PI_2, sync::Arc};

use fj_interop::{Mesh, Model};
use fj_math::{Aabb, Point, Scalar, Transform, Vector};
use iced_wgpu::wgpu::rwh::{HasDisplayHandle,HasWindowHandle};

pub mod graphics;
mod input;
mod assets;
pub mod viewer;

/// Needs to be implemented by types that can serve as a screen to render to
pub trait Screen {
    /// The window
    type Window: HasDisplayHandle + HasWindowHandle + Send + Sync + 'static;

    /// Access the size of the screen
    fn size(&self) -> ScreenSize;

    /// Access the window
    fn window(&self) -> Arc<Self::Window>;
}

/// Cursor position in normalized coordinates (-1 to +1)
///
/// The center of the screen is at (0, 0). The aspect ratio is taken into
/// account.
#[derive(Clone, Copy, Debug)]
pub struct NormalizedScreenPosition {
    /// The x coordinate of the position [-1, 1]
    pub x: f64,

    /// The y coordinate of the position [-1, 1]
    pub y: f64,
}

/// The size of the screen
#[derive(Clone, Copy, Debug)]
pub struct ScreenSize {
    /// The width of the screen
    pub width: u32,

    /// The height of the screen
    pub height: u32,
}

impl ScreenSize {
    /// Convert size to `f64`
    pub fn as_f64(&self) -> [f64; 2] {
        [self.width, self.height].map(Into::into)
    }
}

/// The camera abstraction
///
/// Please note that the metaphor we're using (which influences how mouse input
/// is handled, for example) is not that of a camera freely flying through a
/// static scene. Instead, the camera is static, and the model is freely
/// translated and rotated.
#[derive(Debug)]
pub struct Camera {
    /// The distance to the near plane
    near_plane: f64,

    /// The distance to the far plane
    far_plane: f64,

    /// The rotational part of the transform
    pub rotation: Transform,

    /// The locational part of the transform
    pub translation: Transform,
}

impl Camera {
    const DEFAULT_NEAR_PLANE: f64 = 0.0001;
    const DEFAULT_FAR_PLANE: f64 = 1000.0;

    const INITIAL_FIELD_OF_VIEW_IN_X: f64 = FRAC_PI_2; // 90 degrees

    /// Returns a new camera aligned for viewing a bounding box
    pub fn new() -> Self {
        Self {
            near_plane: Self::DEFAULT_NEAR_PLANE,
            far_plane: Self::DEFAULT_FAR_PLANE,

            rotation: Transform::identity(),
            translation: Transform::identity(),
        }
    }

    /// Returns the distance between the camera and the minimum distance for rendering.
    pub fn near_plane(&self) -> f64 {
        self.near_plane
    }

    /// Returns the distance between the camera and the maximum distance for rendering..
    pub fn far_plane(&self) -> f64 {
        self.far_plane
    }

    /// Returns the horizontal field of view of the camera.
    pub fn field_of_view_in_x(&self) -> f64 {
        Self::INITIAL_FIELD_OF_VIEW_IN_X
    }

    /// Returns the position of the camera in world space.
    pub fn position(&self) -> Point<3> {
        self.camera_to_model()
            .inverse_transform_point(&Point::<3>::origin())
    }

    /// Transform a normalized cursor position on the near plane to model space.
    pub fn cursor_to_model_space(
        &self,
        cursor: NormalizedScreenPosition,
    ) -> Point<3> {
        // Cursor position in camera space.
        let f = (self.field_of_view_in_x() / 2.).tan() * self.near_plane();
        let cursor = Point::origin()
            + Vector::from([cursor.x * f, cursor.y * f, -self.near_plane()]);

        self.camera_to_model().inverse_transform_point(&cursor)
    }

    /// Compute the point on the model, that the cursor currently points to.
    pub fn focus_point(
        &self,
        cursor: Option<NormalizedScreenPosition>,
        model: &Model,
    ) -> FocusPoint {
        self.calculate_focus_point(cursor, &model.mesh)
            .unwrap_or_else(|| FocusPoint(model.aabb.center()))
    }

    fn calculate_focus_point(
        &self,
        cursor: Option<NormalizedScreenPosition>,
        mesh: &Mesh<Point<3>>,
    ) -> Option<FocusPoint> {
        // Transform camera and cursor positions to model space.
        let origin = self.position();
        let cursor = self.cursor_to_model_space(cursor?);
        let dir = (cursor - origin).normalize();

        let mut min_t = None;

        for triangle in mesh.triangles() {
            let t =
                triangle
                    .inner
                    .cast_local_ray(origin, dir, f64::INFINITY, true);

            if let Some(t) = t {
                if t <= min_t.unwrap_or(t) {
                    min_t = Some(t);
                }
            }
        }

        Some(FocusPoint(origin + dir * min_t?))
    }

    /// Access the transform from camera to model space.
    pub fn camera_to_model(&self) -> Transform {
        // Using a mutable variable cleanly takes care of any type inference
        // problems that this operation would otherwise have.
        let mut transform = Transform::identity();

        transform = transform * self.translation;
        transform = transform * self.rotation;

        transform
    }

    /// Initialize the planes
    ///
    /// Call this, if a shape is available for the first time.
    pub fn init_planes(&mut self, aabb: &Aabb<3>) {
        let initial_distance = {
            // Let's make sure we choose a distance, so that the model fills
            // most of the screen.
            //
            // To do that, first compute the model's highest point, as well
            // as the furthest point from the origin, in x and y.
            let highest_point = aabb.max.z;
            let furthest_point =
                [aabb.min.x.abs(), aabb.max.x, aabb.min.y.abs(), aabb.max.y]
                    .into_iter()
                    .reduce(Scalar::max)
                    // `reduce` can only return `None`, if there are no items in
                    // the iterator. And since we're creating an array full of
                    // items above, we know this can't panic.
                    .expect("Array should have contained items");

            // The actual furthest point is not far enough. We don't want
            // the model to fill the whole screen.
            let furthest_point = furthest_point * 2.;

            // Having computed those points, figuring out how far the camera
            // needs to be from the model is just a bit of trigonometry.
            let distance_from_model =
                furthest_point / (Self::INITIAL_FIELD_OF_VIEW_IN_X / 2.).atan();

            // And finally, the distance from the origin is trivial now.
            highest_point + distance_from_model
        };

        let initial_offset = {
            let mut offset = aabb.center();
            offset.z = Scalar::ZERO;
            -offset
        };

        let translation = Transform::translation([
            initial_offset.x,
            initial_offset.y,
            -initial_distance,
        ]);

        self.translation = translation;
    }

    /// Update the max and minimum rendering distance for this camera.
    pub fn update_planes(&mut self, aabb: &Aabb<3>) {
        let view_transform: Transform = self.camera_to_model();
        let view_direction = Vector::from([0., 0., -1.]);

        let mut dist_min = f64::INFINITY;
        let mut dist_max = f64::NEG_INFINITY;

        for vertex in aabb.vertices() {
            let point = view_transform.transform_point(&vertex);

            // Project `point` onto `view_direction`. See this Wikipedia page:
            // https://en.wikipedia.org/wiki/Vector_projection
            //
            // Let's rename the variables first, so they fit the names in that
            // page.
            let (a, b) = (point.coords, view_direction);
            let a1 = b * a.dot(&b) / b.dot(&b);

            let dist = a1.magnitude().into_f64();

            if dist < dist_min {
                dist_min = dist;
            }
            if dist > dist_max {
                dist_max = dist;
            }
        }

        self.near_plane = if dist_min > 0. {
            // Setting `self.near_plane` to `dist_min` should theoretically
            // work, but results in the front of the model being clipped. I
            // wasn't able to figure out why, and for the time being, this
            // factor seems to work well enough.
            dist_min * 0.5
        } else {
            Self::DEFAULT_NEAR_PLANE
        };
        self.far_plane = if dist_max > 0. {
            dist_max
        } else {
            Self::DEFAULT_FAR_PLANE
        };
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

/// The point around which camera movement happens.
///
/// This will be the point on the model that the cursor is currently pointing at if such a point exists,
/// falling back to the center point of the model's bounding volume otherwise.
#[derive(Clone, Copy, Debug)]
pub struct FocusPoint(pub Point<3>);