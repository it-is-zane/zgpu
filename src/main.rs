#![warn(clippy::all, clippy::pedantic)]

use wgpu::util::DeviceExt;

mod app;
mod app_structure;
mod imgui_platform_impl;
mod render_pipeline;
mod renderer_backend;
mod util;

struct DevEvents {
    pipeline: wgpu::RenderPipeline,
    char_bind_group: Option<wgpu::BindGroup>,
    fonts: Vec<font::Font<std::fs::File>>,
    char: char,
    render_percent: f32,
    start_time: std::time::Instant,
}
#[allow(clippy::too_many_lines)]
impl DevEvents {
    fn new(gpu: &app::Gpu, window: &app::WindowBundle) -> DevEvents {
        let module = &gpu
            .device
            .create_shader_module(wgpu::include_wgsl!("font_renderer.wgsl"));

        let pipeline = render_pipeline::Builder::new()
            .add_device(&gpu.device)
            .add_label("test pipeline")
            .add_vertex(wgpu::VertexState {
                module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            })
            .add_fragment(wgpu::FragmentState {
                module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: window.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            })
            .build();

        let font::File { fonts } =
            font::File::open("/usr/share/fonts/aajohan-comfortaa-fonts/Comfortaa-Bold.otf")
                // font::File::open("src/minecraft_font.ttf")
                .unwrap();

        let mut dev_events = DevEvents {
            pipeline,
            char_bind_group: None,
            fonts,
            char: 'e',
            render_percent: 1.0,
            start_time: std::time::Instant::now(),
        };

        dev_events.set_char(gpu, 'e', &1.0);

        dev_events
    }

    fn set_char(&mut self, gpu: &app::Gpu, char: char, percent: &f32) {
        let glyph = self.fonts[0].glyph(char).unwrap().unwrap();

        let mut segments: Vec<f32> = Vec::new();
        let mut contour_markers: Vec<u32> = Vec::new();

        let push_offset = |offset: font::Offset, vec: &mut Vec<f32>| {
            vec.push(offset.0);
            vec.push(offset.1);
        };

        let mut pen = font::Offset(0.0, 0.0);

        for contour in glyph.iter() {
            pen += contour.offset;
            contour_markers.push(contour.segments.len() as u32);

            for segment in contour.iter() {
                match segment {
                    font::glyph::Segment::Linear(offset) => {
                        let a = pen;
                        let b = a + *offset;

                        push_offset(a, &mut segments);
                        push_offset(b, &mut segments);

                        pen = b;
                    }
                    font::glyph::Segment::Quadratic(offset, offset1) => {
                        let a = pen;
                        let b = a + *offset;
                        let c = b + *offset1;

                        let quad = |a: font::Offset, b: font::Offset, c: font::Offset, w: f32| {
                            a + (b - a) * 2.0 * w + (a - b * 2.0 + c) * w * w
                        };

                        let quad_to_seg =
                            |a: font::Offset,
                             b: font::Offset,
                             c: font::Offset,
                             vec: &mut Vec<f32>| {
                                let step = 100;

                                for i in 0..step {
                                    push_offset(quad(a, b, c, i as f32 / step as f32), vec);
                                    push_offset(quad(a, b, c, (i + 1) as f32 / step as f32), vec);
                                }
                            };

                        quad_to_seg(a, b, c, &mut segments);

                        pen = c;
                    }
                    font::glyph::Segment::Cubic(offset, offset1, offset2) => {
                        let a = pen;
                        let b = a + *offset;
                        let c = b + *offset1;
                        let d = c + *offset2;

                        let cubic = |a: font::Offset,
                                     b: font::Offset,
                                     c: font::Offset,
                                     d: font::Offset,
                                     w: f32| {
                            a + (b - a) * 3 * w
                                + (a + c - b * 2) * 3 * w * w
                                + (b * 3 + d - a - c * 3) * w * w * w
                        };

                        let cubic_to_seg =
                            |a: font::Offset,
                             b: font::Offset,
                             c: font::Offset,
                             d: font::Offset,
                             vec: &mut Vec<f32>| {
                                let step = 100;

                                for i in 0..step {
                                    push_offset(cubic(a, b, c, d, i as f32 / step as f32), vec);
                                    push_offset(
                                        cubic(a, b, c, d, (i + 1) as f32 / step as f32),
                                        vec,
                                    );
                                }
                            };

                        cubic_to_seg(a, b, c, d, &mut segments);

                        pen = d;
                    }
                }
            }
        }

        let segment_buffer = if segments.is_empty() {
            gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("segments buffer"),
                size: 32,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            })
        } else {
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("segments buffer"),
                    contents: unsafe { crate::util::as_u8_slice_from_slice(&segments) },
                    usage: wgpu::BufferUsages::STORAGE,
                })
        };

        let contour_markers_buffer = if contour_markers.is_empty() {
            gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("contour_markers buffer"),
                size: 32,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            })
        } else {
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("contour_markers buffer"),
                    contents: unsafe { crate::util::as_u8_slice_from_slice(&contour_markers) },
                    usage: wgpu::BufferUsages::STORAGE,
                })
        };

        let render_percent = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("render percent uniform"),
                contents: unsafe { crate::util::as_u8_slice(percent) },
                usage: wgpu::BufferUsages::UNIFORM,
            });

        self.char_bind_group = Some(gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{char}")),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &segment_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &contour_markers_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &render_percent,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        }));

        gpu.queue.submit(std::iter::empty());
        gpu.device.poll(wgpu::Maintain::Wait);
    }
}

#[allow(clippy::too_many_lines)]
impl app::EventHandler for DevEvents {
    fn window_event(
        &mut self,
        window_bundle: &mut app::WindowBundle,
        gpu: &mut app::Gpu,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: winit::event::WindowEvent,
    ) -> app::Command {
        match event {
            winit::event::WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key:
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            }
            | winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
                return app::Command::DeleteWindow(window_bundle.window.id());
            }
            winit::event::WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key:
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::F11),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => window_bundle.window.set_fullscreen(
                if window_bundle.window.fullscreen().is_some() {
                    window_bundle.window.set_cursor_visible(true);
                    None
                } else {
                    window_bundle.window.set_cursor_visible(false);
                    Some(winit::window::Fullscreen::Borderless(None))
                },
            ),
            winit::event::WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key: winit::keyboard::Key::Character(chars),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if let Some(char) = chars.chars().next() {
                    let percent = self.render_percent;
                    self.char = char;
                    self.set_char(gpu, char, &percent);
                    window_bundle.window.request_redraw();
                }
            }
            winit::event::WindowEvent::MouseInput {
                device_id: _,
                state: winit::event::ElementState::Pressed,
                button: winit::event::MouseButton::Left,
            } => {
                window_bundle
                    .window
                    .drag_window()
                    .expect("failed to drag window");
            }
            winit::event::WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        self.render_percent += y / 1000.0;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(physical_position) => todo!(),
                }
                let percent = self.render_percent;
                self.set_char(gpu, self.char, &percent);
                window_bundle.window.request_redraw();
            }
            winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize { width, height }) => {
                window_bundle.config.width = width;
                window_bundle.config.height = height;

                window_bundle
                    .surface
                    .configure(&gpu.device, &window_bundle.config);
            }
            winit::event::WindowEvent::RedrawRequested => {
                let t = ((1.0 - (self.start_time.elapsed().as_secs_f32() * 0.1) % 1.0) * 2.0 - 1.0)
                    .abs();
                self.set_char(gpu, self.char, &t);

                let mut ce = gpu
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                let surface_texture = window_bundle
                    .surface
                    .get_current_texture()
                    .expect("failed to get surface texture");

                {
                    let mut pass = ce.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &surface_texture
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default()),
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        ..Default::default()
                    });

                    pass.set_pipeline(&self.pipeline);
                    pass.set_bind_group(0, &self.char_bind_group, &[]);
                    pass.draw(0..3, 0..1);
                }

                gpu.queue.submit([ce.finish()]);
                window_bundle.window.pre_present_notify();
                surface_texture.present();
                std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / 30.0));
                window_bundle.window.request_redraw();
            }
            // event => println!("{:#?}", event),
            _ => {}
        }

        app::Command::None
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().expect("failed to create event_loop");

    let mut app = app::App::new();

    app.create_window_on_resumed(|gpu, window| DevEvents::new(gpu, window).into());

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut app).expect("failed to run app");
}
