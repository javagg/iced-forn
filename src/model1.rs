use crate::fjviewer::graphics::DrawConfig;
use crate::fjviewer::{Camera, FocusPoint, NormalizedScreenPosition};

use iced::widget::shader::wgpu;
use iced::Size;
use iced::{mouse, widget::shader, Rectangle};

use std::borrow::Cow;
use std::sync::Arc;

pub struct Program {
    model: Arc<fj_interop::Model>,
}

impl Program {
    pub fn new(model: fj_interop::Model) -> Self {
        Self { 
            model: Arc::new(model)
        }
    }
}

impl<Message> shader::Program<Message> for Program {
    type State = ();

    type Primitive = Primitive;

    fn draw(
        &self,
        state: &Self::State,
        cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        // println!("{:?}", bounds);
        Primitive::new(Arc::clone(&self.model), bounds)
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        // println!("{:?}", cursor);
        mouse::Interaction::default()
    }
}


#[derive(Debug)]
pub struct Primitive {
    camera: Camera,
    cursor: Option<NormalizedScreenPosition>,
    draw_config: DrawConfig,
    focus_point: Option<FocusPoint>,
    model: Arc<fj_interop::Model>,
}

impl Primitive {
    pub fn new(model: Arc<fj_interop::Model>, bounds: Rectangle,) -> Self {
        Self {
            camera: Camera::default(),
            cursor: None,
            draw_config: DrawConfig::default(),
            focus_point: None,
            model,
        }
    }
}

impl shader::Primitive for Primitive {
    /// Processes the [`Primitive`], allowing for GPU buffer allocation.
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut shader::Storage,
        _bounds: &Rectangle,
        viewport: &shader::Viewport,
    ) {
        // println!("prepare...");
        // println!("{:?}", bounds);
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(
                device,
                queue,
                format,
                viewport.physical_size(),
                &self.model,
            ));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        pipeline.update(
            device,
            queue,
            viewport.physical_size(),
            // &self.uniforms,
            // self.cubes.len(),
            // &self.cubes,
        );
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        // println!("render...");
        // println!("clip_bounds: {:?}", clip_bounds);        

        // At this point our pipeline should always be initialized
        let pipeline = storage.get::<Pipeline>().unwrap();

        // // Render primitive
        pipeline.render(
            target,
            encoder,
            *clip_bounds,
        );
    }
}

pub struct Pipeline {
    // camera: Camera,
    // model: Arc<fj_interop::Model>,
    // frame_buffer: wgpu::TextureView,
    // depth_view: wgpu::TextureView,
    // uniform_buffer: wgpu::Buffer,
    // bind_group: wgpu::BindGroup,
    // geometries: Geometries,
    // pipelines: Pipelines,
    // config: DrawConfig,
    render_pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        target_size: Size<u32>,
        model: &Arc<fj_interop::Model>,
    ) -> Self {
    // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

        Self {
            render_pipeline,
        }
    }

    pub fn update(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        target_size: Size<u32>,
    ) {
    }

    pub fn render(
        &self,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        viewport: Rectangle<u32>,
    ) {
        {
            let mut rpass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &target,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                rpass.set_scissor_rect(
                    viewport.x,
                    viewport.y,
                    viewport.width,
                    viewport.height,
                );
                rpass.set_pipeline(&self.render_pipeline);
                rpass.draw(0..3, 0..1);
        }
    }
}
