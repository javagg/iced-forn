
use iced::futures::executor::block_on;
use iced::widget::shader::wgpu;
use iced::Size;
use iced::{mouse, widget::shader, Rectangle};
use crate::fj_viewer::viewer::Viewer;
use crate::fj_viewer::{Camera, FocusPoint, NormalizedScreenPosition};
use crate::fj_viewer::graphics::{
    DrawConfig, 
    Renderer
};

#[derive(Clone)]
pub struct Model {
    pub size: f32,
}

impl Model {
    pub fn new() -> Self {
        Self { size: 0.2 }
    }
}

impl<Message> shader::Program<Message> for Model {
    type State = ();

    type Primitive = Primitive;

    fn draw(
        &self,
        state: &Self::State,
        cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        Primitive::new()
    }
}

#[derive(Debug)]
pub struct Primitive {
    camera: Camera,
    cursor: Option<NormalizedScreenPosition>,
    draw_config: DrawConfig,
    focus_point: Option<FocusPoint>,
    // renderer: Renderer,
    model: Option<fj_interop::Model>,
}

impl Primitive {
    pub fn new() -> Self {
        // let renderer = Renderer::new(size, device, queue, format)?;

        Self {
            camera: Camera::default(),
            cursor: None,
            draw_config: DrawConfig::default(),
            focus_point: None,
            // renderer,
            model: None,
        }
        // Self { viewer: None }
    }
}

impl shader::Primitive for Primitive {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut shader::Storage,
        bounds: &Rectangle,
        viewport: &shader::Viewport,
    ) {
        // if !storage.has::<Pipeline>() {
        //     storage.store(Pipeline::new(
        //         device,
        //         queue,
        //         format,
        //         viewport.physical_size(),
        //     ));
        // }
        // let pipeline = storage.get_mut::<Pipeline>().unwrap();
        // Upload data to GPU
        // pipeline.update(viewport.physical_size());
        // if !storage.has::<Viewer>() {
        //     let viewer = block_on(Viewer::new(target_size, device, queue, format)).expect("Failed to create viewer");
        //     storage.store(viewer);
        // }

        // let viewer = block_on(Viewer::new(target_size, device, queue, format)).expect("Failed to create viewer");
        // self.viewer = Some(viewer);
        let target_size = viewport.physical_size();

    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        // At this point our pipeline should always be initialized
        let viewer = storage.get_mut::<Viewer>().unwrap();
        viewer.draw(target, encoder, clip_bounds);

        // pipeline.render(
        //     target,
        //     encoder,
        //     clip_bounds,
        //     // self.cubes.len() as u32,
        //     // self.show_depth_buffer,
        // );
    }
}

pub struct Pipeline {
    // pipeline: wgpu::RenderPipeline,
    viewer: Viewer
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        target_size: Size<u32>,
    ) -> Self {
        // let screen = VScreen::new(target_size.width, target_size.height);
        let viewer = block_on(Viewer::new(target_size, device, queue, format)).expect("Failed to create viewer");
        Self { 
            viewer
        }
    }

    // pub fn update(&mut self, viewport_size: Size<u32>) {

    // }

    pub fn render(
        &mut self,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        clip_bounds: &Rectangle<u32>,
    ) {
        self.viewer.draw(target, encoder, clip_bounds);
    }
}
