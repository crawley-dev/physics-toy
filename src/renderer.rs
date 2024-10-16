use log::info;
use std::time::Instant;
use winit::dpi::PhysicalSize;
use winit::window::Window;
use winit_input_helper::WinitInputHelper;

// This is all the nitty gritty code of the backend. whilst "backend.rs" is the interface
pub struct State<'a> {
    pub input: WinitInputHelper,
    pub timer: Instant,
    pub start: Instant,
    pub frame: u64,

    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub window_size: PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,

    pub texture: wgpu::Texture,
    pub gpu_data: GpuData,
    pub gpu_buffer: wgpu::Buffer,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    pub window: &'a Window,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GpuData {
    pub time: f32,
}

unsafe impl bytemuck::Zeroable for GpuData {}
unsafe impl bytemuck::Pod for GpuData {}

impl<'a> State<'a> {
    pub async fn new(
        window: &'a Window,
        texture_size: (u32, u32),
        scale: u32,
        sim_data: &[u8],
    ) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            // TODO(TOM): if wasm, use GL.
            ..Default::default()
        });
        info!("Instance created");

        let surface = instance.create_surface(window).unwrap();
        info!("Surface created");

        // >> Requesting Adapter (gpu abstraction) <<
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        info!("Adapter created");

        // >> Creating Device and Queue <<
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();
        info!("Device and Queue created");

        // >> Creating Surface Config <<
        let window_size = window.inner_size();

        let capabilities = surface.get_capabilities(&adapter);
        let surface_format = capabilities
            .formats
            .iter()
            .find(|x| x.is_srgb())
            .copied()
            .unwrap_or(capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Immediate, // Immediate = no vsync, Fifo = vsync
            desired_maximum_frame_latency: 0,
            alpha_mode: Default::default(),
            view_formats: Vec::new(),
        };
        surface.configure(&device, &config);
        info!("Surface configured with format '{surface_format:?}', {window_size:?}");

        // Loading an image.
        // let bytes = include_bytes!("patSilhouette.png");
        // let image = image::load_from_memory(bytes).unwrap();
        // let image_size = image::GenericImageView::dimensions(&image);
        // let texture_data = image.to_rgba8().into_raw();
        // info!("Image loaded with size {image_size:?}");

        // >> Creating Texture <<
        let texture_size = wgpu::Extent3d {
            width: texture_size.0,
            height: texture_size.1,
            depth_or_array_layers: 1, // set to 1 for 2D textures
        };
        // let texture_data =
        //     vec![44u8; texture_size.width as usize * texture_size.height as usize * 4];
        let texture_desc = wgpu::TextureDescriptor {
            label: Some("RGBA Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // SRGB (3 bpp)
            format: config.format,
            // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
            // COPY_DST means that we want to copy data to this texture
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            // This specifies what texture formats can be used to create TextureViews for this texture.
            // The base texture format is always supported. Note that using a different texture format
            // is not supported on the WebGL2 backend.
            view_formats: &[],
        };
        let texture = device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        info!("Texture created");

        // Initial write of the texture data, have no 'self' so cannot use method.
        // TODO(TOM): verify this stays in sync with method self.update_texture()
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &sim_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture_size.width),
                rows_per_image: Some(texture_size.height),
            },
            texture_size,
        );
        info!("RGBA Buffer uploaded to texture.");

        // >> Creating bind group layout <<
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // >> Creating Render Pipeline <<
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });
        info!("Render Pipeline created");

        // >> Creating Bind Group <<
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create a GPU buffer to hold time values, for shader code!
        let gpu_data = GpuData { time: 0.0 };
        let gpu_buffer = wgpu::util::DeviceExt::create_buffer_init(
            &device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[gpu_data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );
        info!("Uniform Buffer created");

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: gpu_buffer.as_entire_binding(),
                },
            ],
        });
        info!("Bind Group created");

        Self {
            timer: Instant::now(),
            start: Instant::now(),
            input: WinitInputHelper::new(),
            frame: 0,
            surface,
            device,
            queue,
            config,
            window_size,
            render_pipeline,
            bind_group,
            texture,
            gpu_data,
            gpu_buffer,
            window,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        assert!(new_size > PhysicalSize::from((0, 0)));
        self.window_size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn update_texture(&self, data: &[u8], window_size: PhysicalSize<u32>) {
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * window_size.width),
                rows_per_image: Some(window_size.height),
            },
            wgpu::Extent3d {
                width: window_size.width,
                height: window_size.height,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn update(&mut self) {
        self.frame += 1;
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // gets the current back SurfaceTexture to use, that will then be presented.
        let frame = self.surface.get_current_texture()?;

        // Creates necessary metadata of the texture for the render pass.
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Creates the GPU commands. Most graphics frameworks expect commands
        // to be stored in a command buffer before being sent to the GPU.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    // Load field determines what is done with the previous frame's contents
                    // >> in this case, we clear the frame to a block color.
                    // load: wgpu::LoadOp::Clear(wgpu::Color {
                    //     r: 0.1,
                    //     g: 0.2,
                    //     b: 0.3,
                    //     a: 1.0,
                    // }),
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        // Writing new time value to a GPU buffer, for shader code to access!
        let elapsed = self.start.elapsed().as_secs_f32();
        self.queue
            .write_buffer(&self.gpu_buffer, 0, bytemuck::cast_slice(&[elapsed]));

        // Takes 6 vertices (2 triangles = 1 square) and the vertex & fragment shader
        render_pass.draw(0..6, 0..1);

        // Drop render_pass' mutable reference to encoder, crashes otherwise.
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }
}
