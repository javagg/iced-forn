
use iced::futures::executor::block_on;
use iced::widget::shader::wgpu;
use iced::Size;
use iced::{mouse, widget::shader, Rectangle};
use crate::fj_viewer::viewer::Viewer;

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
pub struct Primitive {}

impl Primitive {
    pub fn new() -> Self {
        Self {}
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
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(
                device,
                queue,
                format,
                viewport.physical_size(),
            ));
        }
        let pipeline = storage.get_mut::<Pipeline>().unwrap();
        // Upload data to GPU
        pipeline.update(viewport.physical_size());
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        // At this point our pipeline should always be initialized
        let pipeline = storage.get::<Pipeline>().unwrap();

        pipeline.render(
            target,
            encoder,
            clip_bounds,
            // self.cubes.len() as u32,
            // self.show_depth_buffer,
        );
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

    pub fn update(&mut self, viewport_size: Size<u32>) {

    }

    pub fn render(
        &self,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        clip_bounds: &Rectangle<u32>,
    ) {
        self.viewer.draw(target, encoder, clip_bounds);
    }
}
