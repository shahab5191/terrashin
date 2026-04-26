use std::sync::{Arc, mpsc};
use glam::Vec4;
use winit::{
    application::ApplicationHandler, dpi::LogicalSize, event::WindowEvent, event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy}, window::{Window, WindowAttributes}
};
use crate::{gpu::{context::GpuContext, renderer::Renderer}, terrain::{NodeId, TerrainSystem, node::{CheckerNode, PerlinNoiseNode, SolidColorNode}, resource_registry::ResourceKey}};

pub enum AppEvent {
    EvalComplete,
}

pub struct App {
    eval_tx: Option<mpsc::SyncSender<NodeId>>,
    result_rx: Option<mpsc::Receiver<wgpu::TextureView>>,
    current_view: Option<wgpu::TextureView>,
    proxy: Option<EventLoopProxy<AppEvent>>,
    pub window: Option<Arc<Window>>,
    pub gpu_context: Option<Arc<GpuContext>>,
    pub renderer: Option<Renderer>,
    pub root_node_id: Option<NodeId>,
}


impl Default for App {
    fn default() -> Self {
        Self {
            eval_tx: None,
            result_rx: None,
            current_view: None,
            proxy: None,
            window: None,
            gpu_context: None,
            renderer: None,
            root_node_id: None,
        }
    }
}

impl App {
    pub fn new(proxy: EventLoopProxy<AppEvent>) -> Self {
        let mut app = Self::default();
        app.proxy = Some(proxy);
        app
    }
}

impl ApplicationHandler<AppEvent> for App {

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);

        let window = Arc::new(event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("wgpu compute test")
                    .with_inner_size(LogicalSize::new(800.0, 600.0)),
            )
            .unwrap());

        let gpu_context = pollster::block_on(GpuContext::new(window.clone()));
        let renderer = Renderer::new(&gpu_context);

        // store everything
        self.window = Some(window);
        self.gpu_context = Some(Arc::new(gpu_context));
        self.renderer = Some(renderer);

        let mut terrain = TerrainSystem::new();
        let gpu = self.gpu_context.as_ref().unwrap();
        let red_node = Box::new(SolidColorNode::new(Vec4::new(1.0, 0.0, 0.0, 1.0)));
        let checker_node = Box::new(CheckerNode::new(
            gpu,
            8.0,
            Vec4::new(1.0, 0.5, 1.0, 1.0),
            Vec4::new(0.5, 0.0, 1.0, 1.0),
        ));
        let checker_node_id = terrain.graph.add_node(checker_node);
        let perlin_node = Box::new(
            PerlinNoiseNode::new(
                gpu, 15.0
            )
        );
        let perlin_node_id = terrain.graph.add_node(perlin_node);
        let red_node_id = terrain.graph.add_node(red_node);
        let root_node_id = perlin_node_id; // Start with the Perlin noise as the root
        terrain.graph.connect(
            red_node_id,
            "Output".to_string(),
            checker_node_id,
            "Color1".to_string(),
        );
        self.root_node_id = Some(root_node_id);

        let (eval_tx, eval_rx) = mpsc::sync_channel::<NodeId>(1);
        let (result_tx, result_rx) = mpsc::channel::<wgpu::TextureView>();
        let gpu_arc = Arc::clone(self.gpu_context.as_ref().unwrap());
        let proxy = self.proxy.as_ref().unwrap().clone();
        std::thread::spawn(move || {
            while let Ok(node_id) = eval_rx.recv() {
                terrain.evaluate_node(node_id, &gpu_arc);
                if let Some(view) = terrain.get_output(ResourceKey::output(node_id, "Output".to_string())) {
                    result_tx.send(view.clone()).ok();
                    proxy.send_event(AppEvent::EvalComplete).ok();
                }
            }
        });

        self.eval_tx = Some(eval_tx);
        self.result_rx = Some(result_rx);

        // Trigger the initial evaluation
        self.eval_tx.as_ref().unwrap().try_send(root_node_id).ok();
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
                let gpu = self.gpu_context.as_ref().unwrap();
                let window = self.window.as_ref().unwrap();
                let renderer = self.renderer.as_ref().unwrap();

                if let Some(view) = &self.current_view {
                    window.pre_present_notify();
                    renderer.render(gpu, view);
                }
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        println!("Received user event");
        match event {
            AppEvent::EvalComplete => {
                println!("Evaluation complete, trying to receive result...");
                if let Ok(view) = self.result_rx.as_ref().unwrap().try_recv() {
                    self.current_view = Some(view);
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Wait);
    }
}
