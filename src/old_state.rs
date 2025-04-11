use glm::ext;

use crate::{
    model::game_object::Object,
    renderer_backend::{bind_group_layout, material::Material, mesh_builder, pipeline, ubo::UBO},
};

pub struct State<'a> {
    instance: wgpu::Instance,
    surface: Option<wgpu::Surface<'a>>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    pub window: std::sync::Arc<winit::window::Window>,
    render_pipeline: wgpu::RenderPipeline,
    triangle_mesh: wgpu::Buffer,
    quad_mesh: mesh_builder::Mesh,
    triangle_material: Material,
    quad_material: Material,
    ubo: Option<UBO>,
    sdf_pipeline: wgpu::RenderPipeline,
}

impl<'a> State<'a> {
    pub async fn new(window: winit::window::Window) -> Self {
        let size = window.inner_size();
        let window = std::sync::Arc::new(window);

        let instace_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        };

        let instance = wgpu::Instance::new(&instace_descriptor);

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter_descriptor = wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            // compatible_surface: Some(&surface),
            compatible_surface: None,
            force_fallback_adapter: false,
        };

        let adapter = instance.request_adapter(&adapter_descriptor).await.unwrap();

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("Device"),
            memory_hints: wgpu::MemoryHints::Performance,
        };

        let (device, queue) = adapter
            .request_device(&device_descriptor, None)
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb() && f.components() == 4)
            .next()
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: *surface_capabilities
                .alpha_modes
                .iter()
                .find(|m| **m != wgpu::CompositeAlphaMode::Opaque)
                .unwrap(),
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let triangle_mesh = mesh_builder::make_triangle(&device);
        let quad_mesh = mesh_builder::make_quad(&device);

        let material_bind_group_layout: wgpu::BindGroupLayout;
        {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_material();
            material_bind_group_layout = builder.build("Material Bind Group Layout");
        }

        let ubo_bind_group_layout;
        {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_ubo(wgpu::ShaderStages::VERTEX);
            ubo_bind_group_layout = builder.build("UBO")
        }

        let render_pipeline;
        {
            let mut builder = pipeline::Builder::new(&device);
            builder.set_shader_module("shader.wgsl", "vs_main", "fs_main");
            builder.set_pixel_format(config.format);
            builder.add_vertex_buffer_layout(mesh_builder::Vertex::get_layout());
            builder.add_bind_group_layout(&material_bind_group_layout);
            builder.add_bind_group_layout(&ubo_bind_group_layout);
            render_pipeline = builder.build_pipeline("Render Pipeline");
        }

        let quad_material =
            Material::new("src/zim.jpg", &device, &queue, &material_bind_group_layout);
        let triangle_material = Material::new(
            "src/MillerGoogly.webp",
            &device,
            &queue,
            &material_bind_group_layout,
        );

        let ubo_bind_group_layout;
        {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_ubo(wgpu::ShaderStages::FRAGMENT);
            ubo_bind_group_layout = builder.build("UBO")
        }

        let sdf_pipeline = {
            let mut builder = pipeline::Builder::new(&device);
            builder.set_shader_module("sdf_shader.wgsl", "vs_main", "fs_main");
            builder.set_pixel_format(config.format);
            builder.add_bind_group_layout(&ubo_bind_group_layout);
            builder.build_pipeline("SDF Render Pipeline")
        };

        Self {
            instance,
            surface: Some(surface),
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            triangle_mesh,
            quad_mesh,
            triangle_material,
            quad_material,
            ubo: None,
            sdf_pipeline,
        }
    }

    pub fn build_ubos_for_objects(&mut self, object_count: usize, visibility: wgpu::ShaderStages) {
        let ubo_bind_group_layout;
        {
            let mut builder = bind_group_layout::Builder::new(&self.device);
            builder.add_ubo(visibility);
            ubo_bind_group_layout = builder.build("UBO");
        }

        self.ubo = Some(UBO::new(&self.device, object_count, ubo_bind_group_layout));
    }

    pub fn render(
        &mut self,
        quads: &Vec<Object>,
        tris: &Vec<Object>,
    ) -> Result<(), wgpu::SurfaceError> {
        let mut offset: u64 = 0;

        for i in 0..quads.len() {
            let c0 = glm::Vec4::new(1.0, 0.0, 0.0, 0.0);
            let c1 = glm::Vec4::new(0.0, 1.0, 0.0, 0.0);
            let c2 = glm::Vec4::new(0.0, 0.0, 1.0, 0.0);
            let c3 = glm::Vec4::new(0.0, 0.0, 0.0, 1.0);
            let m1 = glm::Mat4::new(c0, c1, c2, c3);
            let m2 = glm::Mat4::new(c0, c1, c2, c3);

            let matrix = ext::rotate(&m1, quads[i].angle, glm::Vec3::new(0.0, 0.0, 1.0))
                * ext::translate(&m2, quads[i].position);

            self.ubo
                .as_mut()
                .unwrap()
                .upload(offset + i as u64, &matrix, &self.queue);
        }

        offset += quads.len() as u64;

        // for i in 0..tris.len() {
        //     let c0 = glm::Vec4::new(1.0, 0.0, 0.0, 0.0);
        //     let c1 = glm::Vec4::new(0.0, 1.0, 0.0, 0.0);
        //     let c2 = glm::Vec4::new(0.0, 0.0, 1.0, 0.0);
        //     let c3 = glm::Vec4::new(0.0, 0.0, 0.0, 1.0);
        //     let m1 = glm::Mat4::new(c0, c1, c2, c3);
        //     let m2 = glm::Mat4::new(c0, c1, c2, c3);

        //     let matrix = ext::rotate(&m1, tris[i].angle, glm::Vec3::new(0.0, 0.0, 1.0))
        //         * ext::translate(&m2, tris[i].position);

        //     self.ubo
        //         .as_mut()
        //         .unwrap()
        //         .upload(offset + i as u64, &matrix, &self.queue);
        // }

        self.ubo.as_mut().unwrap().upload(
            0,
            &glm::Vec2::new(self.size.width as f32, self.size.height as f32),
            &self.queue,
        );

        let event = self.queue.submit([]);
        let maintain = wgpu::Maintain::WaitForSubmissionIndex(event);
        self.device.poll(maintain);

        let surface = match self.surface.as_ref() {
            Some(s) => s,
            None => return Ok(()),
        };

        let drawable = surface.get_current_texture()?;
        let image_view_descriptor = wgpu::TextureViewDescriptor::default();
        let image_view = drawable.texture.create_view(&image_view_descriptor);

        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        };

        let mut command_encoder = self
            .device
            .create_command_encoder(&command_encoder_descriptor);

        let color_attachment = wgpu::RenderPassColorAttachment {
            view: &image_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        };

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Renderpass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };

        {
            let mut renderpass = command_encoder.begin_render_pass(&render_pass_descriptor);

            // renderpass.set_pipeline(&self.render_pipeline);
            // renderpass.set_bind_group(0, &self.quad_material.bind_group, &[]);

            // renderpass.set_vertex_buffer(0, self.quad_mesh.buffer.slice(0..self.quad_mesh.offset));
            // renderpass.set_index_buffer(
            //     self.quad_mesh.buffer.slice(self.quad_mesh.offset..),
            //     wgpu::IndexFormat::Uint16,
            // );
            // let mut offset: usize = 0;
            // for i in 0..quads.len() {
            //     renderpass.set_bind_group(
            //         1,
            //         &self.ubo.as_ref().unwrap().bind_groups[offset + i],
            //         &[],
            //     );

            //     renderpass.draw_indexed(0..6, 0, 0..1);
            // }

            // renderpass.set_bind_group(0, &self.triangle_material.bind_group, &[]);
            // renderpass.set_vertex_buffer(0, self.triangle_mesh.slice(..));

            // offset += quads.len();
            // for i in 0..tris.len() {
            //     renderpass.set_bind_group(
            //         1,
            //         &self.ubo.as_ref().unwrap().bind_groups[offset + i],
            //         &[],
            //     );
            //     renderpass.draw(0..3, 0..1);
            // }
            //
            renderpass.set_pipeline(&self.sdf_pipeline);
            renderpass.set_bind_group(0, &self.ubo.as_ref().unwrap().bind_groups[0], &[]);
            renderpass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));

        self.window.pre_present_notify();
        drawable.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        let surface = match self.surface.as_ref() {
            Some(s) => s,
            None => return,
        };

        if new_size.width * new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            surface.configure(&self.device, &self.config);
        }
    }
}
