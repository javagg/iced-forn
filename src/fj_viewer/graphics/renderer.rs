use std::{io, mem::size_of, vec};

use iced::{Rectangle, Size};
use thiserror::Error;
use tracing::{ error, trace};
use iced_wgpu::wgpu::{self};
use wgpu::util::DeviceExt as _;

use crate::fj_viewer::{Camera};

use super::{
    draw_config::DrawConfig, drawables::Drawables,
    geometries::Geometries, navigation_cube::NavigationCubeRenderer,
    pipelines::Pipelines, transform::Transform, uniforms::Uniforms,
    vertices::Vertices, DeviceError, DEPTH_FORMAT, SAMPLE_COUNT,
};

/// Graphics rendering state and target abstraction
#[derive(Debug)]
pub struct Renderer<'a> {
    // surface: wgpu::Surface<'static>,
    // device: Device,

    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,

    surface_config: wgpu::SurfaceConfiguration,
    frame_buffer: wgpu::TextureView,
    depth_view: wgpu::TextureView,

    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,

    geometries: Geometries,
    pipelines: Pipelines,

    navigation_cube_renderer: NavigationCubeRenderer,
}

impl<'a> Renderer<'a> {
    /// Returns a new `Renderer`.
    // pub async fn new(screen: &impl Screen) -> Result<Self, RendererInitError> {
    //     let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
    //         backends: wgpu::Backends::all(),
    //         ..Default::default()
    //     });

    //     // This is sound, as `window` is an object to create a surface upon.
    //     let surface = instance.create_surface(screen.window())?;

    //     for adapter in instance.enumerate_adapters(wgpu::Backends::all()) {
    //         debug!("Available adapter: {:?}", adapter.get_info());
    //     }

    //     let result = Device::from_preferred_adapter(&instance, &surface).await;
    //     let (device, adapter, features) = match result {
    //         Ok((device, adapter, features)) => (device, adapter, features),
    //         Err(_) => {
    //             error!("Failed to acquire device from preferred adapter");

    //             match Device::try_from_all_adapters(&instance).await {
    //                 Ok((device, adapter, features)) => {
    //                     (device, adapter, features)
    //                 }
    //                 Err(err) => {
    //                     error!("Prepend `RUST_LOG=fj_viewer=debug` and re-run");
    //                     error!("Then open an issue and post your output");
    //                     error!(
    //                         "https://github.com/hannobraun/fornjot/issues/new"
    //                     );

    //                     return Err(err.into());
    //                 }
    //             }
    //         }
    //     };

    //     let color_format = 'color_format: {
    //         let capabilities = surface.get_capabilities(&adapter);
    //         let supported_formats = capabilities.formats;

    //         // We don't really care which color format we use, as long as we
    //         // find one that's supported. `egui_wgpu` prints a warning though,
    //         // unless we choose one of the following ones.
    //         let preferred_formats = [
    //             wgpu::TextureFormat::Rgba8Unorm,
    //             wgpu::TextureFormat::Bgra8Unorm,
    //         ];

    //         for format in preferred_formats {
    //             if supported_formats.contains(&format) {
    //                 break 'color_format format;
    //             }
    //         }

    //         // None of the preferred color formats are supported. Just use one
    //         // of the supported ones then.
    //         supported_formats
    //             .into_iter()
    //             .next()
    //             .expect("No color formats supported")
    //     };

    //     let ScreenSize { width, height } = screen.size();
    //     let surface_config = wgpu::SurfaceConfiguration {
    //         usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    //         format: color_format,
    //         width,
    //         height,
    //         present_mode: wgpu::PresentMode::AutoVsync,
    //         desired_maximum_frame_latency: 2,
    //         // I don't understand what this option does. It was introduced with
    //         // wgpu 0.14, but we had already been using premultiplied alpha
    //         // blending before that. See the `BlendState` configuration of the
    //         // render pipelines.
    //         //
    //         // For that reason, I tried to set this to `PreMultiplied`, but that
    //         // failed on Linux/Wayland (with in integrated AMD GPU). Setting it
    //         // to `Auto` seems to just work.
    //         //
    //         // @hannobraun
    //         alpha_mode: wgpu::CompositeAlphaMode::Auto,
    //         view_formats: vec![],
    //     };
    //     surface.configure(&device.device, &surface_config);

    //     let frame_buffer =
    //         Self::create_frame_buffer(&device.device, &surface_config);
    //     let depth_view =
    //         Self::create_depth_buffer(&device.device, &surface_config);

    //     let uniform_buffer = device.device.create_buffer_init(
    //         &wgpu::util::BufferInitDescriptor {
    //             label: None,
    //             contents: bytemuck::cast_slice(&[Uniforms::default()]),
    //             usage: wgpu::BufferUsages::UNIFORM
    //                 | wgpu::BufferUsages::COPY_DST,
    //         },
    //     );
    //     let bind_group_layout = device.device.create_bind_group_layout(
    //         &wgpu::BindGroupLayoutDescriptor {
    //             entries: &[wgpu::BindGroupLayoutEntry {
    //                 binding: 0,
    //                 visibility: wgpu::ShaderStages::all(),
    //                 ty: wgpu::BindingType::Buffer {
    //                     ty: wgpu::BufferBindingType::Uniform,
    //                     has_dynamic_offset: false,
    //                     min_binding_size: wgpu::BufferSize::new(size_of::<
    //                         Uniforms,
    //                     >(
    //                     )
    //                         as u64),
    //                 },
    //                 count: None,
    //             }],
    //             label: None,
    //         },
    //     );
    //     let bind_group =
    //         device.device.create_bind_group(&wgpu::BindGroupDescriptor {
    //             layout: &bind_group_layout,
    //             entries: &[wgpu::BindGroupEntry {
    //                 binding: 0,
    //                 resource: wgpu::BindingResource::Buffer(
    //                     wgpu::BufferBinding {
    //                         buffer: &uniform_buffer,
    //                         offset: 0,
    //                         size: None,
    //                     },
    //                 ),
    //             }],
    //             label: None,
    //         });

    //     let geometries = Geometries::new(&device.device, &Vertices::empty());
    //     let pipelines = Pipelines::new(
    //         &device.device,
    //         &bind_group_layout,
    //         color_format,
    //         features,
    //     );

    //     let navigation_cube_renderer = NavigationCubeRenderer::new(
    //         &device.device,
    //         &device.queue,
    //         &surface_config,
    //     );

    //     Ok(Self {
    //         surface,
    //         device,

    //         surface_config,
    //         frame_buffer,
    //         depth_view,

    //         uniform_buffer,
    //         bind_group,

    //         geometries,
    //         pipelines,

    //         navigation_cube_renderer,
    //     })
    // }


    pub fn new(
        // screen: &impl Screen,
        size: Size<u32>,
        device: &wgpu::Device,
        queue: &wgpu::Queue, 
        format: wgpu::TextureFormat,) -> Result<Self, RendererInitError> {
        // let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        //     backends: wgpu::Backends::all(),
        //     ..Default::default()
        // });

        // This is sound, as `window` is an object to create a surface upon.
        // let surface = instance.create_surface(screen.window())?;

        // for adapter in instance.enumerate_adapters(wgpu::Backends::all()) {
        //     debug!("Available adapter: {:?}", adapter.get_info());
        // }

        // let result = Device::from_preferred_adapter(&instance, &surface).await;
        // let (device, adapter, features) = match result {
        //     Ok((device, adapter, features)) => (device, adapter, features),
        //     Err(_) => {
        //         error!("Failed to acquire device from preferred adapter");

        //         match Device::try_from_all_adapters(&instance).await {
        //             Ok((device, adapter, features)) => {
        //                 (device, adapter, features)
        //             }
        //             Err(err) => {
        //                 error!("Prepend `RUST_LOG=fj_viewer=debug` and re-run");
        //                 error!("Then open an issue and post your output");
        //                 error!(
        //                     "https://github.com/hannobraun/fornjot/issues/new"
        //                 );

        //                 return Err(err.into());
        //             }
        //         }
        //     }
        // };

        // let color_format = 'color_format: {
        //     let capabilities = surface.get_capabilities(&adapter);
        //     let supported_formats = capabilities.formats;

        //     // We don't really care which color format we use, as long as we
        //     // find one that's supported. `egui_wgpu` prints a warning though,
        //     // unless we choose one of the following ones.
        //     let preferred_formats = [
        //         wgpu::TextureFormat::Rgba8Unorm,
        //         wgpu::TextureFormat::Bgra8Unorm,
        //     ];

        //     for format in preferred_formats {
        //         if supported_formats.contains(&format) {
        //             break 'color_format format;
        //         }
        //     }

        //     // None of the preferred color formats are supported. Just use one
        //     // of the supported ones then.
        //     supported_formats
        //         .into_iter()
        //         .next()
        //         .expect("No color formats supported")
        // };

        // let ScreenSize { width, height } = screen.size();
        // let { width, height } = size;
        let width = size.width;
        let height = size.height;
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            // format: color_format,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            // I don't understand what this option does. It was introduced with
            // wgpu 0.14, but we had already been using premultiplied alpha
            // blending before that. See the `BlendState` configuration of the
            // render pipelines.
            //
            // For that reason, I tried to set this to `PreMultiplied`, but that
            // failed on Linux/Wayland (with in integrated AMD GPU). Setting it
            // to `Auto` seems to just work.
            //
            // @hannobraun
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        // surface.configure(&device.device, &surface_config);

        let frame_buffer =
            Self::create_frame_buffer(device, &surface_config);
        let depth_view =
            Self::create_depth_buffer(device, &surface_config);

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

        let navigation_cube_renderer = NavigationCubeRenderer::new(
            device,
            queue,
            &surface_config,
        );

        Ok(Self {
            // surface,
            device,
            queue,

            surface_config,
            frame_buffer,
            depth_view,

            uniform_buffer,
            bind_group,

            geometries,
            pipelines,

            navigation_cube_renderer,
        })
    }

    /// Updates the geometry of the model being rendered.
    pub fn update_geometry(&mut self, mesh: Vertices) {
        self.geometries = Geometries::new(&self.device, &mesh);
    }

    /// Resizes the render surface.
    ///
    /// # Arguments
    /// - `size`: The target size for the render surface.
    pub fn handle_resize(&mut self, size: Size<u32>) {
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;

        // self.surface
        //     .configure(&self.device, &self.surface_config);

        self.frame_buffer = Self::create_frame_buffer(
            &self.device,
            &self.surface_config,
        );
        self.depth_view = Self::create_depth_buffer(
            &self.device,
            &self.surface_config,
        );
    }

    /// Draws the renderer, camera, and config state to the window.
    pub fn draw(
        &self,
        camera: &Camera,
        config: &DrawConfig,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        clip_bounds: &Rectangle<u32>,
    ) -> Result<(), DrawError> {
        let aspect_ratio = f64::from(self.surface_config.width)
            / f64::from(self.surface_config.height);
        let uniforms = Uniforms {
            transform: Transform::for_vertices(camera, aspect_ratio),
            transform_normals: Transform::for_normals(camera),
        };

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniforms]),
        );

        // let surface_texture = match self.surface.get_current_texture() {
        //     Ok(surface_texture) => surface_texture,
        //     Err(wgpu::SurfaceError::Timeout) => {
        //         // I'm seeing this all the time now (as in, multiple times per
        //         // microsecond), with `PresentMode::AutoVsync`. Not sure what's
        //         // going on, but for now, it works to just ignore it.
        //         //
        //         // Issues for reference:
        //         // - https://github.com/gfx-rs/wgpu/issues/1218
        //         // - https://github.com/gfx-rs/wgpu/issues/1565
        //         return Ok(());
        //     }
        //     result => result?,
        // };
        // let color_view = surface_texture
        //     .texture
        //     .create_view(&wgpu::TextureViewDescriptor::default());

        // let mut encoder = self.device.create_command_encoder(
        //     &wgpu::CommandEncoderDescriptor { label: None },
        // );
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

        // let command_buffer = encoder.finish();
        // self.queue.submit(Some(command_buffer));

        trace!("Presenting...");
        // surface_texture.present();

        trace!("Finished drawing.");
        Ok(())
    }

    fn create_frame_buffer(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::TextureView {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: surface_config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn create_depth_buffer(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::TextureView {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}

/// Error describing the set of render surface initialization errors
#[derive(Error, Debug)]
pub enum RendererInitError {
    /// General IO error
    #[error("I/O error")]
    Io(#[from] io::Error),

    /// Surface creating error
    #[error("Error creating surface")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),

    /// Device error
    #[error(transparent)]
    Device(#[from] DeviceError),
}

/// Draw error
///
/// Returned by [`Renderer::draw`].
#[derive(Error, Debug)]
#[error("Error acquiring output surface: {0}")]
pub struct DrawError(#[from] wgpu::SurfaceError);
