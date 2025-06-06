pub enum Command {
    DeleteWindow(winit::window::WindowId),
    CreateWindow(Box<dyn Fn(&Gpu, &WindowBundle) -> EventHandlerPtr>),
    None,
}

pub trait EventHandler {
    fn window_event(
        &mut self,
        window_bundle: &mut WindowBundle,
        gpu: &mut Gpu,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: winit::event::WindowEvent,
    ) -> Command {
        Command::None
    }
}

pub enum EventHandlerPtr {
    Box(Box<dyn EventHandler>),
    Rc(std::rc::Rc<std::cell::RefCell<dyn EventHandler>>),
    Arc(std::sync::Arc<std::sync::Mutex<dyn EventHandler>>),
}

impl<T: EventHandler + 'static> From<T> for EventHandlerPtr {
    fn from(value: T) -> Self {
        EventHandlerPtr::Box(Box::new(value))
    }
}

impl From<Box<dyn EventHandler>> for EventHandlerPtr {
    fn from(value: Box<dyn EventHandler>) -> Self {
        EventHandlerPtr::Box(value)
    }
}

impl From<std::rc::Rc<std::cell::RefCell<dyn EventHandler>>> for EventHandlerPtr {
    fn from(value: std::rc::Rc<std::cell::RefCell<dyn EventHandler>>) -> Self {
        EventHandlerPtr::Rc(value)
    }
}

impl From<std::sync::Arc<std::sync::Mutex<dyn EventHandler>>> for EventHandlerPtr {
    fn from(value: std::sync::Arc<std::sync::Mutex<dyn EventHandler>>) -> Self {
        EventHandlerPtr::Arc(value)
    }
}

pub struct WindowBundle<'a> {
    pub window: std::sync::Arc<winit::window::Window>,
    pub surface: wgpu::Surface<'a>,
    pub config: wgpu::SurfaceConfiguration,
}

pub struct Gpu {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

pub struct App<'a> {
    gpu: Gpu,
    add_event_handler_queue: Vec<Box<dyn Fn(&Gpu, &WindowBundle) -> EventHandlerPtr>>,
    windows:
        std::collections::HashMap<winit::window::WindowId, (WindowBundle<'a>, EventHandlerPtr)>,
}

impl App<'_> {
    pub fn new() -> Self {
        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        };

        let instance = wgpu::Instance::new(&instance_descriptor);

        let adapter_descriptor = wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        };

        let adapter =
            crate::util::insync(async { instance.request_adapter(&adapter_descriptor).await })
                .expect("failed to get adapter");

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("Device"),
            memory_hints: wgpu::MemoryHints::Performance,
        };

        let (device, queue) =
            crate::util::insync(async { adapter.request_device(&device_descriptor, None).await })
                .expect("failed to get device/queue");

        Self {
            gpu: Gpu {
                instance,
                adapter,
                device,
                queue,
            },
            add_event_handler_queue: Vec::new(),
            windows: std::collections::HashMap::new(),
        }
    }

    pub fn create_window_on_resumed<F>(&mut self, callaback: F)
    where
        F: Fn(&Gpu, &WindowBundle) -> EventHandlerPtr + 'static,
    {
        self.add_event_handler_queue.push(Box::new(callaback));
    }

    fn create_window(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event_handler_callback: &Box<dyn Fn(&Gpu, &WindowBundle) -> EventHandlerPtr>,
    ) {
        let window = std::sync::Arc::new(
            event_loop
                .create_window(
                    winit::window::WindowAttributes::default()
                        .with_inner_size(winit::dpi::LogicalSize::new(1024, 1024))
                        .with_transparent(true)
                        .with_decorations(false),
                )
                .expect("failed to create window"),
        );

        let id = window.id();

        let surface = self
            .gpu
            .instance
            .create_surface(window.clone())
            .expect("failed to create window surface");

        let winit::dpi::PhysicalSize { width, height } = window.inner_size();

        let mut config = surface
            .get_default_config(&self.gpu.adapter, width, height)
            .expect("failed to get default surface config");

        let capabilities = surface.get_capabilities(&self.gpu.adapter);

        config.format = *capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .unwrap_or(&config.format);

        config.alpha_mode = wgpu::CompositeAlphaMode::Inherit;

        surface.configure(&self.gpu.device, &config);

        let window_bundle = WindowBundle {
            window,
            surface,
            config,
        };

        let event_handler = event_handler_callback(&self.gpu, &window_bundle);

        self.windows.insert(id, (window_bundle, event_handler));
    }
}

impl winit::application::ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        while let Some(handler) = self.add_event_handler_queue.pop() {
            self.create_window(event_loop, &handler);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some((window_bundle, event_handler)) = self.windows.get_mut(&window_id) {
            let command = match event_handler {
                EventHandlerPtr::Box(event_handler) => {
                    event_handler.window_event(window_bundle, &mut self.gpu, event_loop, event)
                }
                EventHandlerPtr::Rc(ref_cell) => ref_cell.borrow_mut().window_event(
                    window_bundle,
                    &mut self.gpu,
                    event_loop,
                    event,
                ),
                EventHandlerPtr::Arc(mutex) => match mutex.lock() {
                    Ok(mut event_handler) => {
                        event_handler.window_event(window_bundle, &mut self.gpu, event_loop, event)
                    }
                    Err(_) => Command::DeleteWindow(window_id),
                },
            };

            match command {
                Command::DeleteWindow(window_id) => {
                    self.windows.remove(&window_id);
                }
                Command::CreateWindow(event_handler) => {
                    self.create_window(event_loop, &event_handler);
                }
                Command::None => (),
            }
        }
    }
}
