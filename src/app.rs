use std::sync::Arc;
use glam::Vec4;
use winit::{
    application::ApplicationHandler, dpi::LogicalSize, event::WindowEvent, event_loop::ActiveEventLoop, window::{Window, WindowAttributes}
};
use crate::{gpu::{context::GpuContext, renderer::Renderer}, terrain::{NodeId, TerrainSystem, node::SolidColorNode, resource_registry::ResourceKey}};

pub struct App {
    pub window: Option<Arc<Window>>,
    pub gpu_context: Option<GpuContext>,
    pub renderer: Option<Renderer>,
    pub terrain_system: Option<TerrainSystem>,
    pub root_node_id: Option<NodeId>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            gpu_context: None,
            renderer: None,
            terrain_system: Some(TerrainSystem::new()),
            root_node_id: None,
        }
    }
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

        let gpu_context = pollster::block_on(GpuContext::new(window.clone()));
        let renderer = Renderer::new(&gpu_context);

        window.request_redraw();

        // store everything
        self.window = Some(window);
        self.gpu_context = Some(gpu_context);
        self.renderer = Some(renderer);
        let red_node = Box::new(SolidColorNode::new(
            Vec4::new(1.0, 0.0, 0.0, 1.0))
        );
        let root_node_id = self.terrain_system.as_mut().unwrap().graph.add_node(red_node);
        self.root_node_id = Some(root_node_id);
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
                let root_node_id = self.root_node_id.unwrap();

                // 1. Evaluate the graph (this calls encode and populates views)
                let terrain = self.terrain_system.as_mut().unwrap();
                terrain.evaluate_node(root_node_id, gpu);

                // 2. Get the output view
                let output_view = terrain.get_output(
                    ResourceKey::output(root_node_id, "Output".to_string())
                ).expect("Node output not found after evaluation");

                // 3. Render it to the window surface
                renderer.render(gpu, output_view);

                window.pre_present_notify();

                // keep rendering
                window.request_redraw();
            }
            _ => {}
        }
    }
}
