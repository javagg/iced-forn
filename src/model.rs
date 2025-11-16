use crate::fjviewer::graphics::drawables::Drawables;
use crate::fjviewer::graphics::geometries::Geometries;
use crate::fjviewer::graphics::pipelines::Pipelines;
use crate::fjviewer::graphics::transform::Transform;
use crate::fjviewer::graphics::uniforms::Uniforms;
use crate::fjviewer::graphics::vertices::Vertices;
use crate::fjviewer::graphics::DrawConfig;
use crate::fjviewer::{Camera, FocusPoint, NormalizedScreenPosition};

use iced::widget::shader::wgpu;
use iced::Size;
use iced::{mouse, widget::shader, Rectangle};

use std::mem::size_of;
use std::sync::Arc;
use wgpu::util::DeviceExt;


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

pub const SAMPLE_COUNT: u32 = 4;

#[derive(Debug)]
pub struct Primitive {
    // camera: Camera,
    // cursor: Option<NormalizedScreenPosition>,
    // draw_config: DrawConfig,
    // focus_point: Option<FocusPoint>,
    model: Arc<fj_interop::Model>,
}

impl Primitive {
    pub fn new(model: Arc<fj_interop::Model>, bounds: Rectangle,) -> Self {
        Self {
            // camera: Camera::default(),
            // cursor: None,
            // draw_config: DrawConfig::default(),
            // focus_point: None,
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
                self.model.clone(),
            ));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        pipeline.update(
            device,
            queue,
            viewport.physical_size(),
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
    camera: Camera,
    model: Arc<fj_interop::Model>,
    depth_view: wgpu::TextureView,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    geometries: Geometries,
    pipelines: Pipelines,
    config: DrawConfig,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        target_size: Size<u32>,
        model: Arc<fj_interop::Model>,
    ) -> Self {
        let width = target_size.width;
        let height = target_size.height;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[Uniforms::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(size_of::<Uniforms>() as u64),
                },
                count: None,
            }],
            label: None,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
            label: None,
        });

        let pipelines = Pipelines::new(
            device,
            &bind_group_layout,
            format,
            wgpu::Features::empty(),
        );


        // // let navigation_cube_renderer = NavigationCubeRenderer::new(
        // //     device,
        // //     queue,
        // //     &surface_config,
        // // );
        // // let aabb = model.aabb.as_ref().map(|shape| shape.aabb).unwrap_or_default();

        let mut camera = Camera::default();
        camera.update_planes(&model.aabb);

        let aspect_ratio = f64::from(width) / f64::from(height);
        let uniforms = Uniforms {
            transform: Transform::for_vertices(&camera, aspect_ratio),
            transform_normals: Transform::for_normals(&camera),
        };

        queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));


        let geometries = Geometries::new(device, &((&model.mesh).into()));

        Self {
            camera,
            model: model.clone(),
            depth_view,
            uniform_buffer,
            bind_group,
            geometries,
            pipelines,
            config: DrawConfig::default(),
        }
    }

    pub fn update(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        target_size: Size<u32>,
        // uniforms: &Uniforms,
    ) {
    }

    pub fn render(
        &self,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        viewport: Rectangle<u32>,
    ) {
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                // depth_stencil_attachment: None,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            render_pass.set_scissor_rect(
                viewport.x,
                viewport.y,
                viewport.width,
                viewport.height,
            );
            render_pass.set_bind_group(0, &self.bind_group, &[]);

            let drawables = Drawables::new(&self.geometries, &self.pipelines);

            // if self.config.draw_model {
                drawables.model.draw(&mut render_pass);
            // }

            // if let Some(drawable) = drawables.mesh {
            //     if self.config.draw_mesh {
            //         drawable.draw(&mut render_pass);
            //     }
            // }
        }
        // self.navigation_cube_renderer.draw(
        //     target,
        //     encoder,
        //     &self.queue,
        //     aspect_ratio,
        //     self.camera.rotation,
        // );
    }
}
