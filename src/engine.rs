use winit::dpi::{PhysicalSize, Size};
use winit::event::{ElementState, KeyEvent};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowBuilder};

struct Engine<'a> {
    event_loop: EventLoop<()>,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window_size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: &'a Window,
}
// https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#state-new
impl<'a> Engine<'a> {
    async fn new(title: &str, width: u32, height: u32) -> Engine<'a> {
        assert!(width > 0 && height > 0);

        let event_loop = EventLoop::new().unwrap();

        let window_size = PhysicalSize::new(width, height);
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(Size::Physical(window_size))
            .build(&event_loop)
            .unwrap();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            // TODO(TOM): if wasm, GL.
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

        Engine {
            event_loop,
            surface,
            device,
            queue,
            config,
            window_size,
            window,
        }
    }

    async fn run(&mut self) {
        self.event_loop
            .run(move |event, control_flow| match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),
                    _ => {}
                },
                _ => {}
            })
            .unwrap()
    }

    // pub fn window(&self) -> &Window {
    //     &self.window
    // }
    //
    // fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    //     todo!()
    // }
    //
    // fn input(&mut self, event: &WindowEvent) -> bool {
    //     todo!()
    // }
    //
    // fn update(&mut self) {
    //     todo!()
    // }
    //
    // fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    //     todo!()
    // }
}

// use winit::event_loop::EventLoop;

// const BYTES_PER_PIXEL: usize = 4;

// pub trait Renderer {
//     fn get_window_size(&self) -> (u32, u32);
//     fn get_cursor_pos(&self) -> (u32, u32);
//     fn is_mouse_down(&self) -> bool;
//     fn is_running(&self) -> bool;
//
//     fn push_change(&mut self, colour: u32, x: u32, y: u32, scale: u32);
//     fn handle_events(&mut self);
//     fn render_frame(&mut self);
//     fn new(title: &str, width: u32, height: u32) -> Self;
// }

// Windows Render contains a Window and a buffer
// The "Window" is an all-encompassing view of a system window and io events
// The "Buffer" is a pixel array (0RGB format),
// .. the buf_width & height represent the dimensions of the buffer and NOT the dimensions of the window.
// .. The buffer will be equal to or bigger than the window view to avoid re-allocations, keep it large!
// pub struct WindowsRenderer {
//     sdl_context: Sdl,
//     event_handler: EventPump,
//     video_subsystem: VideoSubsystem,
//     canvas: Canvas<Window>,
//     texture: Texture,
//     pixel_buf: Vec<u8>,
//     running: bool,
//     buf_size: (u32, u32),
// }
// pub struct WindowsRenderer {}
//
// impl Renderer for WindowsRenderer {
//     fn get_window_size(&self) -> (u32, u32) {
//         todo!()
//     }
//
//     fn get_cursor_pos(&self) -> (u32, u32) {
//         todo!()
//     }
//
//     fn is_mouse_down(&self) -> bool {
//         todo!()
//     }
//
//     fn is_running(&self) -> bool {
//         todo!()
//     }
//
//     fn push_change(&mut self, colour: u32, x: u32, y: u32, scale: u32) {
//         todo!()
//     }
//
//     fn handle_events(&mut self) {
//         todo!()
//     }
//
//     fn render_frame(&mut self) {
//         todo!()
//     }
//
//     fn new(title: &str, width: u32, height: u32) -> Self {
//         todo!()
//     }
// }
