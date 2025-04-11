mod state;
mod util;

struct App<'a> {
    state: state::State,
    window: Option<std::sync::Arc<winit::window::Window>>,
    surface: Option<wgpu::Surface<'a>>,
    sdf_curve_pipeline: state::SdfCurve,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    render_sub_view: state::RenderSubView,
}

impl App<'_> {
    fn new() -> Self {
        let mut state = util::insync(state::State::new(None));
        let sdf_curve_pipeline = state::SdfCurve::new(&state.device);

        let texture = state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sub view"),
            size: wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let render_sub_view = state::RenderSubView::new(&state.device, &texture_view);

        sdf_curve_pipeline.upload_uniform(&state.queue, &glm::vec2(64.0, 64.0));

        state.render(&texture_view, |mut rp, state| {
            sdf_curve_pipeline.render(&mut rp);
        });

        Self {
            state,
            window: None,
            surface: None,
            sdf_curve_pipeline,
            texture,
            texture_view,
            render_sub_view,
        }
    }
}

impl winit::application::ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = std::sync::Arc::new(
            event_loop
                .create_window(winit::window::WindowAttributes::default())
                .unwrap(),
        );

        window.set_title("my yet to be named daw");
        _ = window.request_inner_size(winit::dpi::PhysicalSize::new(600, 600));

        self.window = Some(window.clone());

        let surface = self.state.instance.create_surface(window.clone()).unwrap();

        surface.configure(
            &self.state.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                width: window.inner_size().width,
                height: window.inner_size().height,
                present_mode: wgpu::PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: Vec::new(),
            },
        );

        self.surface = Some(surface);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::RedrawRequested => {
                if let (Some(window), Some(surface)) = (self.window.as_mut(), self.surface.as_mut())
                {
                    let surface_texture = surface.get_current_texture().unwrap();
                    let texture_view = surface_texture
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    self.render_sub_view.upload_uniform(
                        &self.state.queue,
                        &glm::Vec2::new(
                            window.inner_size().width as f32,
                            window.inner_size().height as f32,
                        ),
                    );

                    self.state.render(&texture_view, |mut rp, state| {
                        self.render_sub_view.render(&mut rp);
                    });

                    window.pre_present_notify();
                    surface_texture.present();
                }
            }
            winit::event::WindowEvent::Resized(size) => {
                if let Some(surface) = self.surface.as_ref() {
                    surface.configure(
                        &self.state.device,
                        &wgpu::SurfaceConfiguration {
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            format: wgpu::TextureFormat::Rgba8UnormSrgb,
                            width: size.width,
                            height: size.height,
                            present_mode: wgpu::PresentMode::Fifo,
                            desired_maximum_frame_latency: 2,
                            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                            view_formats: Vec::new(),
                        },
                    );
                }
            }
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state: winit::event::ElementState::Pressed,
                        physical_key: winit::keyboard::PhysicalKey::Code(c),
                        ..
                    },
                ..
            } => match c {
                winit::keyboard::KeyCode::KeyF => {
                    if let Some(window) = self.window.as_mut() {
                        match window.fullscreen() {
                            Some(_) => window.set_fullscreen(None),
                            None => window
                                .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None))),
                        }
                    }
                }
                winit::keyboard::KeyCode::KeyQ => {
                    event_loop.exit();
                }
                _ => (),
            },
            _ => (),
        };
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().expect("failed to create event_loop");

    let mut app = App::new();

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut app).expect("failed to run app");
}
