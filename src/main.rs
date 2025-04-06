mod model;
mod renderer_backend;
mod state;
mod util;

use model::game_object::Object;
use state::State;

struct World {
    quads: Vec<Object>,
    tris: Vec<Object>,
}

impl World {
    fn new() -> Self {
        World {
            quads: Vec::new(),
            tris: Vec::new(),
        }
    }

    fn update(&mut self, dt: f32) {
        for i in 0..self.tris.len() {
            self.tris[i].angle = self.tris[i].angle + 0.001 * dt;
            if self.tris[i].angle > 360.0 {
                self.tris[i].angle -= 360.0;
            }
        }
    }
}

struct App<'a> {
    state: Option<State<'a>>,
    world: World,
}
impl App<'_> {
    fn new() -> Self {
        let world = World::new();

        Self { state: None, world }
    }
}

impl winit::application::ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(winit::window::WindowAttributes::default().with_transparent(true))
            .unwrap();

        window.set_title("my yet to be named daw");
        _ = window.request_inner_size(winit::dpi::PhysicalSize::new(600, 600));

        let mut state = util::insync(State::new(window));
        self.world.quads.push(Object {
            position: glm::vec3(0.5, 0.0, 0.0),
            angle: 0.0,
        });
        self.world.tris.push(Object {
            position: glm::vec3(0.0, 0.0, 0.0),
            angle: 0.0,
        });
        state.build_ubos_for_objects(1, wgpu::ShaderStages::FRAGMENT);

        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::RedrawRequested => {
                if let Some(state) = self.state.as_mut() {
                    self.world.update(1.0);

                    state
                        .render(&self.world.quads, &self.world.tris)
                        .unwrap_or_else(|e| eprintln!("i know you!{:?}", e));

                    // state.window.request_redraw();
                }
            }
            winit::event::WindowEvent::Resized(size) => {
                if let Some(state) = self.state.as_mut() {
                    state.resize(size);
                }
            }
            winit::event::WindowEvent::CloseRequested => {
                self.state = None;
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
                    if let Some(state) = self.state.as_mut() {
                        match state.window.fullscreen() {
                            Some(_) => state.window.set_fullscreen(None),
                            None => state
                                .window
                                .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None))),
                        }
                    }
                }
                winit::keyboard::KeyCode::KeyQ => {
                    self.state = None;
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
