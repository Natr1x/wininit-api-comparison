use common::Graphics;
use wasm_bindgen::{prelude::wasm_bindgen, UnwrapThrowExt};
use winit::{event::{Event, StartCause, WindowEvent}, event_loop::{EventLoop, EventLoopWindowTarget}, window::Window};

const CANVAS_ID: &str = "old-api-canvas";

async fn app_loop(event_loop: EventLoop<()>, window: std::rc::Rc<Window>) {
    let size = window.inner_size();
    let mut graphics = Graphics::new(size.width, size.height, window.clone())
        .await.expect_throw("To prepare graphics context");

    let event_handler = move |event: Event<()>, target: &EventLoopWindowTarget<()>| {
        match event {
            Event::NewEvents(StartCause::Init) if !cfg!(target_os = "android") => {
                let size = window.inner_size();
                log::info!("Setting size from window inner size {size:?}");
                graphics.resized(size.width, size.height);
                log::info!("Old api started");
            }

            Event::Resumed if cfg!(target_os = "android") => {
                let size = window.inner_size();
                log::info!("Setting size from window inner size {size:?}");
                graphics.resized(size.width, size.height);
                log::info!("This shouldn't run on android");
            }

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => graphics.resized(size.width, size.height),
                WindowEvent::RedrawRequested => graphics.render().expect_throw("To render our graphics"),
                WindowEvent::CloseRequested => target.exit(),
                _ => (),
            },
            _ => (),
        }
    };

    use winit::platform::web::EventLoopExtWebSys;
    EventLoop::spawn(event_loop, event_handler);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run() {
    common::init_logger("old-api");
    log::info!("Old Api Example is starting");
    let event_loop = EventLoop::new().unwrap_throw();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    let mut builder = winit::window::WindowBuilder::new();

    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::wasm_bindgen::JsCast;
        use winit::platform::web::WindowBuilderExtWebSys;
        let window = web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
        let html_canvas_element = canvas.unchecked_into();
        builder = builder.with_canvas(Some(html_canvas_element));
    }

    let window = std::rc::Rc::new(builder.build(&event_loop).unwrap_throw());
    wasm_bindgen_futures::spawn_local(app_loop(event_loop, window));
}
