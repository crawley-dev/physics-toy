use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

pub struct State<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub window_size: PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    pub window: &'a Window,
}

impl<'a> State<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let window_size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            // TODO(TOM): if wasm, use GL.
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: Default::default(),
                    required_limits: Default::default(),
                    memory_hints: Default::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|x| x.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 0,
            alpha_mode: Default::default(),
            view_formats: Vec::new(),
        };

        Self {
            surface,
            device,
            queue,
            config,
            window_size,
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

    // Returns a bool to indicate whether an event has been fully processed.
    // If the event has been fully processed (ret true), the event loop will not continue to
    // process the event.
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // gets the current back SurfaceTexture to use, that will then be presented.
        let output = self.surface.get_current_texture()?;

        // Creates necessary metadata of the texture for the render pass.
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Will create the commands to send to the GPU. Most graphics frameworks expect commands
        // to be stored in a command buffer before being sent to the GPU.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
