use std::sync::Arc;

use winit::{
    application::ApplicationHandler, dpi::LogicalSize, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowAttributes}
};

struct App {
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("resumed called");

        let window = Arc::new(event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("wgpu compute test")
                    .with_inner_size(LogicalSize::new(800.0, 600.0)),
            )
            .unwrap());

        // --- WGPU INIT ---
        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            },
        )).unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor::default()
        )).unwrap();

        let size = window.inner_size();

        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        window.request_redraw();

        // store everything
        self.window = Some(window);
        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                _event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let surface = self.surface.as_ref().unwrap();
                let device = self.device.as_ref().unwrap();
                let queue = self.queue.as_ref().unwrap();
                let window = self.window.as_ref().unwrap();

                let frame = match surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(frame) => frame,
                    // It's usually fine to keep rendering to suboptimal frames
                    wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame, 
                    wgpu::CurrentSurfaceTexture::Outdated |
                    wgpu::CurrentSurfaceTexture::Timeout |
                    wgpu::CurrentSurfaceTexture::Occluded => {
                        // The surface isn't ready, is minimized, or changed size.
                        // Bailing out of the function skips rendering for this tick.
                        return; 
                    }
                    _ => {
                        // Any other error is unexpected and should be handled.
                        panic!("Failed to acquire next swap chain texture!");
                    }
                };

                let view = frame.texture.create_view(&Default::default());

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("render encoder"),
                    });

                {
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("clear pass"),
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
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                }

                queue.submit(Some(encoder.finish()));

                window.pre_present_notify(); // 👈 VERY important on Wayland
                frame.present();

                // keep rendering
                window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut app = App {
        window: None,
        surface: None,
        device: None,
        queue: None,
        config: None,
    };

    event_loop.run_app(&mut app).unwrap();
}
