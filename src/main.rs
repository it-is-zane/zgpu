#![warn(clippy::all, clippy::pedantic)]

use image::EncodableLayout;

mod app;
mod app_structure;
mod imgui_platform_impl;
mod render_pipeline;
mod renderer_backend;
mod util;

struct DevEvents {
    pipeline: wgpu::RenderPipeline,
    text_bind_group: wgpu::BindGroup,
    start_time: std::time::Instant,
}
impl DevEvents {
    fn new(gpu: &app::Gpu, window: &app::WindowBundle) -> DevEvents {
        let module = &gpu
            .device
            .create_shader_module(wgpu::include_wgsl!("uv_tris.wgsl"));

        let pipeline = render_pipeline::Builder::new()
            .add_device(&gpu.device)
            .add_label("text render pipeline")
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

        let image = image::open("src/atlas.png").unwrap().to_rgba8();

        let texture = &gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("font atlas"),
            size: wgpu::Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });

        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 1,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            image.as_bytes(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image.width()),
                rows_per_image: Some(image.width()),
            },
            wgpu::Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            },
        );

        let sampler = &gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sampler linear"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let text_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("text bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });

        let mut dev_events = DevEvents {
            pipeline,
            text_bind_group,
            start_time: std::time::Instant::now(),
        };

        dev_events
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
                    pass.set_bind_group(0, &self.text_bind_group, &[]);
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
