mod gpu;
mod app;
mod terrain;

use app::{App, AppEvent};
use winit::event_loop::EventLoop;

fn main() {
    let event_loop: EventLoop<AppEvent> = EventLoop::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    let mut app = App::new(proxy);

    event_loop.run_app(&mut app).unwrap();
}
