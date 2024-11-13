#[derive(Debug)]
struct Triangle {
    pipeline: wgpu::RenderPipeline,
}

impl Triangle {

    fn init(device: &wgpu::Device, color_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Triangle Render Pipeling"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(color_format.into())],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        Self { pipeline }
    }

    fn draw(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Triangle Drawer"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.draw(0..3, 0..1);
    }
}

#[derive(Debug)]
pub struct Graphics<'window> {
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'window>,
    surface_config: wgpu::SurfaceConfiguration,
    triangle: Triangle,
}

impl<'window> Graphics<'window> {
    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        log::trace!("Rendering graphics");
        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        self.triangle.draw(&mut encoder, &view);
        let command_buffer = encoder.finish();
        self.queue.submit([command_buffer]);
        frame.present();
        Ok(())
    }
    
    /// Gets (width, height) from the surface configuration.
    pub fn physical_size(&self) -> (u32, u32) {
        (self.surface_config.width, self.surface_config.height)
    }

    pub async fn new(
        width: u32, height: u32, window: impl Into<wgpu::SurfaceTarget<'window>>
    ) -> Result<Self, GraphicsError> {
        log::info!("Begin creating graphics context");

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window)?;

        let Some(adapter) = instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }).await else { return Err(GraphicsError::NoCompatibleAdapter) };

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                ..Default::default()
            }, None).await?;

        let Some(surface_config) = surface.get_default_config(
            &adapter, width.max(1), height.max(1)) else {
            return Err(GraphicsError::IncompatibleAdapter)
        };
        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];
        let triangle = Triangle::init(&device, swapchain_format);

        log::info!("Finished creating graphics context");
        Ok(Self { _instance: instance, _adapter: adapter, device, queue, surface, surface_config, triangle })
    }

    pub fn resized(&mut self, width: u32, height: u32) {
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
        log::info!("Resized surface to {{ width: {width}, height: {height} }}");
    }
}

#[derive(Debug)]
pub enum GraphicsError {
    NoCompatibleAdapter,
    IncompatibleAdapter,
    RequestDeviceError(Box<wgpu::RequestDeviceError>),
    CreateSurfaceError(Box<wgpu::CreateSurfaceError>),
}

macro_rules! impl_errors {
    (@ours [$($our:path => $desc:literal),+ $(,)?]
     @theirs [$($wrapper:path => $theirs:ty),+ $(,)?]) => {
        $(impl From<$theirs> for GraphicsError {
            fn from(item: $theirs) -> Self { $wrapper(Box::new(item)) }
        })*
        impl core::fmt::Display for GraphicsError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $($our => write!(f, $desc),)+
                    $($wrapper(nested) => nested.fmt(f)),+
                }
            }
        }
        impl core::error::Error for GraphicsError {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    $($wrapper(nested) => Some(nested),)+
                    _ => None,
                }
            }
        }
    };
}

impl_errors!(
    @ours [
        GraphicsError::NoCompatibleAdapter => "Could not find a compatible adapter",
        GraphicsError::IncompatibleAdapter => "Adapter and surface are not compatible",
    ]
    @theirs [
        GraphicsError::RequestDeviceError => wgpu::RequestDeviceError,
        GraphicsError::CreateSurfaceError => wgpu::CreateSurfaceError,
    ]
);

fn parse_url_query_string<'a>(
    query: &'a str, key_prefix: &'a str
) -> impl Iterator<Item = (&'a str, &'a str)> + 'a {
    query.split('&')
        .filter_map(move |s| {
            let mut pair = s.split('=');
            let key = pair.next()?;
            let value = pair.next()?;
            if key.starts_with(key_prefix) {
                Some((key, value))
            } else {
                None
            }
        })
}

/// Initiate log levels using the querystring. Module names should be prefixed with 'loglvl_'.
///
/// You can specify just 'loglvl_base' to set the base_level except for wgpu_core and wgpu_hal libraries
/// because of their noisiness.
pub fn init_logger(topic: &'static str) {
    use std::sync::LazyLock;
    static QUERY_STRING: std::sync::LazyLock<String> = LazyLock::new(||
        web_sys::window().unwrap().location().search().unwrap());

    let mut dispatch = fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .level_for("wgpu_core", log::LevelFilter::Error)
        .level_for("wgpu_hal", log::LevelFilter::Error);

    let mut parse_errors = vec![];

    if let Some(query_string) = QUERY_STRING.strip_prefix('?') {
        for (key, value) in parse_url_query_string(query_string, "loglvl_") {
            let level: log::LevelFilter = match value.parse() {
                Ok(level) => level,
                Err(error) => {
                    parse_errors.push((key, value, error));
                    continue;
                }
            };

            dispatch = match key.strip_prefix("loglvl_") {
                Some("base") => dispatch.level(level),
                Some(module) => dispatch.level_for(module, level),
                _ => dispatch,
            };
        }
    }

    dispatch.chain(fern::Output::call(console_log::log))
        .format(move |out, message, record| {
            out.finish(format_args!("[{} {} {}] {}",
                topic, record.level(), record.target(), message))
        })
        .apply()
        .unwrap();
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    for (key, value, error) in parse_errors {
        log::error!("Could not parse LevelFilter for '{key}={value}', {error}");
    }

}
