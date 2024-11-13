use common::Graphics;
use wasm_bindgen::{prelude::wasm_bindgen, UnwrapThrowExt};

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

const CANVAS_ID: &str = "new-api-canvas";

type AppWindow = std::rc::Rc<Window>;
type AppGraphics = Graphics<'static>;
type AppEvent = (AppWindow, AppGraphics);

enum Application {
    Building(Option<EventLoopProxy<AppEvent>>),
    Running {
        #[allow(unused)]
        window: AppWindow,
        graphics: AppGraphics,
    },
}

impl Application {
    fn new(event_loop: &EventLoop<AppEvent>) -> Self {
        let loop_proxy = Some(event_loop.create_proxy());
        log::info!("Creating a new application");
        Self::Building(loop_proxy)
    }

    fn render(&mut self) {
        let Self::Running { graphics, .. } = self else {
            log::info!("Draw call rejected because graphics doesn't exist yet");
            return;
        };

        graphics.render().expect_throw("To render our graphics");
    }

    fn resized(&mut self, size: PhysicalSize<u32>) {
        let Self::Running { graphics, .. } = self else {
            log::info!("Resized called before having graphics");
            return;
        };
        graphics.resized(size.width, size.height);
    }
}

impl ApplicationHandler<AppEvent> for Application {
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => self.resized(size),
            WindowEvent::RedrawRequested => self.render(),
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => (),
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let Self::Building(builder) = self else {
            return; // Graphics have been built.
        };
        let Some(loop_proxy) = builder.take() else {
            return; // Graphics are being built.
        };

        let mut window_attrs = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;
            let window = web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attrs = window_attrs.with_canvas(Some(html_canvas_element));
        }

        let window = std::rc::Rc::new(event_loop.create_window(window_attrs).unwrap_throw());
        let size = window.inner_size();
        let graphics = Graphics::new(size.width, size.height, window.clone());

        log::info!("Spawning future to build the graphics context");
        wasm_bindgen_futures::spawn_local(async move {
            let graphics = graphics.await.expect_throw("To build a graphics context");
            loop_proxy.send_event((window, graphics))
                .expect("To send the built graphics back to us");
        });
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, app_event: AppEvent) {
        let (window, mut graphics) = app_event;
        if matches!(self, Self::Running { .. }) {
            log::error!("Received a new graphics context when we already have one");
            return;
        }

        // The surface is not actually configured yet.
        let size = window.inner_size();
        graphics.resized(size.width, size.height);

        log::info!("App is now up and running");
        *self = Self::Running { window, graphics };
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run() {
    use winit::platform::web::EventLoopExtWebSys;
    common::init_logger("new-api");
    log::info!("New Api Example is starting");
    let event_loop = EventLoop::with_user_event().build().unwrap_throw();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    let app = Application::new(&event_loop);
    event_loop.spawn_app(app);
}
