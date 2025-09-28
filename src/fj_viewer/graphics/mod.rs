//! Rendering primitives, routines, and structures.

mod device;
mod draw_config;
pub mod drawables;
pub mod geometries;
mod model;
pub mod navigation_cube;
pub mod pipelines;
// mod renderer;
mod shaders;
mod texture;
pub mod transform;
pub mod uniforms;
pub mod vertices;

use iced_wgpu::wgpu;

pub use self::{
    device::DeviceError,
    draw_config::DrawConfig,
    // renderer::{Renderer, RendererInitError},
};

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const SAMPLE_COUNT: u32 = 4;
