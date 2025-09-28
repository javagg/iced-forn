use fj_core::algorithms::approx:: Tolerance;
use fj_core::algorithms::bounding_volume::BoundingVolume;
use fj_core::algorithms::triangulate::Triangulate;
use fj_core::objects::{Region, Sketch};
use fj_core::operations::build::{BuildRegion, BuildSketch};
use fj_core::operations::sweep::SweepSketch;
use fj_core::operations::update::UpdateSketch;
use fj_math::{Aabb, Point, Scalar, Vector};
use iced::widget::shader::wgpu;
use iced::Size;
use iced::{mouse, widget::shader, Rectangle};
use crate::fj_viewer::graphics::drawables::Drawables;
use crate::fj_viewer::graphics::geometries::Geometries;
use crate::fj_viewer::graphics::transform::Transform;
use crate::fj_viewer::graphics::pipelines::Pipelines;
use crate::fj_viewer::{Camera, FocusPoint, NormalizedScreenPosition};
use crate::fj_viewer::graphics::{
    DrawConfig, 
};
use crate::fj_viewer::graphics::uniforms::Uniforms;
use crate::fj_viewer::graphics::vertices::Vertices;


use wgpu::util::DeviceExt;

#[derive(Clone)]
pub struct Model {
    m: fj_interop::Model,
    // pub size: f32,
}

impl Model {
    pub fn new() -> Self {
        let [x, y, z] = [3.0, 2.0, 1.0];
        let mut core = fj_core::Core::new();
        let bottom_surface = core.layers.objects.surfaces.xy_plane();
        let sweep_path = Vector::from([Scalar::ZERO, Scalar::ZERO, (-z).into()]);
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

        core.layers.validation.take_errors().expect("Model is invalid");
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

        // if let Some(path) = args.export {
        //     fj_export::export(&mesh, &path)?;
        //     return Ok(());
        // }

        let m = fj_interop::Model { mesh, aabb };

        Self {
            m,
            // size: x as f32, 
        }
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
        Primitive::new(
            self.m.clone(),
            bounds,
        )
    }
}

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const SAMPLE_COUNT: u32 = 4;

#[derive(Debug)]
pub struct Primitive {
    camera: Camera,
    cursor: Option<NormalizedScreenPosition>,
    draw_config: DrawConfig,
    focus_point: Option<FocusPoint>,
    model: fj_interop::Model,
}

impl Primitive {
    pub fn new(model: fj_interop::Model, bounds: Rectangle) -> Self {
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
        bounds: &Rectangle,
        viewport: &shader::Viewport,
    ) {
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

        // Upload data to GPU
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
        // At this point our pipeline should always be initialized
        let pipeline = storage.get::<Pipeline>().unwrap();

        // Render primitive
        pipeline.render(
            target,
            encoder,
            *clip_bounds,
            // self.cubes.len() as u32,
            // self.show_depth_buffer,
        );
    }
}

pub struct Pipeline<'a> {
    pipeline: wgpu::RenderPipeline,
    m: &'a fj_interop::Model,
    // viewer: Viewer<'a>
}

impl<'a> Pipeline<'a> {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        target_size: Size<u32>,
        m: fj_interop::Model,
    ) -> Self {
        let width = target_size.width;
        let height = target_size.height;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,//: surface_config.width,
                height,//: surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format,//: surface_config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let frame_buffer = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // let depth_view =
        //     Self::create_depth_buffer(device, &surface_config);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,//: surface_config.width,
                height,//: surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[Uniforms::default()]),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            },
        );
        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(size_of::<
                            Uniforms,
                        >(
                        )
                            as u64),
                    },
                    count: None,
                }],
                label: None,
            },
        );
        let bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        wgpu::BufferBinding {
                            buffer: &uniform_buffer,
                            offset: 0,
                            size: None,
                        },
                    ),
                }],
                label: None,
            });

        let geometries = Geometries::new(device, &Vertices::empty());
        let pipelines = Pipelines::new(
            device,
            &bind_group_layout,
            format, // color_format,
            // features,
        );

        // let navigation_cube_renderer = NavigationCubeRenderer::new(
        //     device,
        //     queue,
        //     &surface_config,
        // );
        let aabb = self
            .model
            .as_ref()
            .map(|shape| shape.aabb)
            .unwrap_or_default();

        self.camera.update_planes(&aabb);

        let aspect_ratio = f64::from(surface_config.width)
        / f64::from(surface_config.height);
        let uniforms = Uniforms {
            transform: Transform::for_vertices(&self.camera, aspect_ratio),
            transform_normals: Transform::for_normals(&self.camera),
        };

        queue.write_buffer(
            &uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniforms]),
        );

        Self { 
            // viewer
        }
    }

      pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_size: Size<u32>,
        // uniforms: &Uniforms,
        // num_cubes: usize,
        // cubes: &[cube::Raw],
    ) {

    }

  pub fn render(
        &self,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        viewport: Rectangle<u32>,
        // num_cubes: u32,
        // show_depth: bool,
    ) {
       let color_view = target;

        // Need this block here, as a render pass only takes effect once it's
        // dropped.
        {
            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &self.frame_buffer,
                            resolve_target: Some(color_view),
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                // Not necessary, due to MSAA being enabled.
                                store: wgpu::StoreOp::Discard,
                            },
                        },
                    )],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachment {
                            view: &self.depth_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        },
                    ),
                    ..Default::default()
                });
            render_pass.set_bind_group(0, &self.bind_group, &[]);

            let drawables = Drawables::new(&self.geometries, &self.pipelines);

            if config.draw_model {
                drawables.model.draw(&mut render_pass);
            }

            if let Some(drawable) = drawables.mesh {
                if config.draw_mesh {
                    drawable.draw(&mut render_pass);
                }
            }
        }

        self.navigation_cube_renderer.draw(
            color_view,
            encoder,
            &self.queue,
            aspect_ratio,
            camera.rotation,
        );
        
        // self.viewer.draw(target, encoder, clip_bounds);
    }
}
