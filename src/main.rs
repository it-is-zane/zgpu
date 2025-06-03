#![warn(clippy::all, clippy::pedantic)]

mod app;
mod app_structure;
mod imgui_platform_impl;
mod render_pipeline;
mod renderer_backend;
mod util;

struct DevEvents {
    pipeline: wgpu::RenderPipeline,
    material_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    time_uniform: wgpu::Buffer,
    start: std::time::Instant,
}
#[allow(clippy::too_many_lines)]
impl DevEvents {
    fn new(gpu: &app::Gpu, window: &app::WindowBundle) -> DevEvents {
        let module = &gpu
            .device
            .create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline = render_pipeline::Builder::new()
            .add_device(&gpu.device)
            .add_label("test pipeline")
            .add_vertex(wgpu::VertexState {
                module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[glm::Vec3; 2]>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: std::mem::size_of::<glm::Vec3>() as u64,
                            shader_location: 1,
                        },
                    ],
                }],
            })
            .add_fragment(wgpu::FragmentState {
                module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: window.config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            })
            .build();

        let image = image::open("src/MillerGoogly.webp")
            .unwrap()
            .flipv()
            .to_rgba8();

        let size = image.dimensions();
        let e3d = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };

        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("src/miller_googly.webp"),
            size: e3d,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });

        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.0),
                rows_per_image: Some(size.1),
            },
            e3d,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let time_uniform = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("time uniform"),
            size: std::mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let material_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &time_uniform,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        let vertices = [
            [glm::Vec3::new(-1., 1., 0.), glm::Vec3::new(0., 1., 0.)],
            [glm::Vec3::new(1., 1., 0.), glm::Vec3::new(1., 1., 0.)],
            [glm::Vec3::new(1., -1., 0.), glm::Vec3::new(1., 0., 0.)],
            //
            [glm::Vec3::new(-1., 1., 0.), glm::Vec3::new(0., 1., 0.)],
            [glm::Vec3::new(-1., -1., 0.), glm::Vec3::new(0., 0., 0.)],
            [glm::Vec3::new(1., -1., 0.), glm::Vec3::new(1., 0., 0.)],
        ];

        let bytes = unsafe { crate::util::as_u8_slice(&vertices) };

        let vertex_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertices"),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        gpu.queue.write_buffer(&vertex_buffer, 0, bytes);

        DevEvents {
            pipeline,
            material_bind_group,
            vertex_buffer,
            time_uniform,
            start: std::time::Instant::now(),
        }
    }
}

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
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyQ),
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
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyF),
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
            winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize { width, height }) => {
                window_bundle.config.width = width;
                window_bundle.config.height = height;

                window_bundle
                    .surface
                    .configure(&gpu.device, &window_bundle.config);
            }
            winit::event::WindowEvent::RedrawRequested => {
                let since = self.start.elapsed().as_secs_f32();
                gpu.queue.write_buffer(&self.time_uniform, 0, unsafe {
                    crate::util::as_u8_slice(&since)
                });

                let mut ce = gpu
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                let surface_texture = window_bundle
                    .surface
                    .get_current_texture()
                    .expect("failed to get surface texture");

                let view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor {
                        ..Default::default()
                    });

                {
                    let mut pass = ce.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &surface_texture
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default()),
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        ..Default::default()
                    });

                    pass.set_pipeline(&self.pipeline);
                    pass.set_bind_group(0, Some(&self.material_bind_group), &[]);
                    pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    pass.draw(0..6, 0..1);
                }

                gpu.queue.submit([ce.finish()]);
                window_bundle.window.pre_present_notify();
                surface_texture.present();

                // std::thread::sleep(std::time::Duration::from_secs_f64(1.0 / 30.0));
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
