pub struct DeviceMissing;
pub struct VertexMissing;

pub struct Builder<'a, T = DeviceMissing, V = VertexMissing> {
    device: Option<&'a wgpu::Device>,
    label: Option<&'a str>,
    layout: Option<&'a wgpu::PipelineLayout>,
    vertex: Option<wgpu::VertexState<'a>>,
    primitive: Option<wgpu::PrimitiveState>,
    depth_stencil: Option<wgpu::DepthStencilState>,
    multisample: Option<wgpu::MultisampleState>,
    fragment: Option<wgpu::FragmentState<'a>>,
    multiview: Option<std::num::NonZero<u32>>,
    cache: Option<&'a wgpu::PipelineCache>,
    _phantom: std::marker::PhantomData<(T, V)>,
}

impl Builder<'_, DeviceMissing, VertexMissing> {
    pub fn new() -> Self {
        Self {
            device: None,
            label: None,
            layout: None,
            vertex: None,
            primitive: None,
            depth_stencil: None,
            multisample: None,
            fragment: None,
            multiview: None,
            cache: None,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, D, V> Builder<'a, D, V> {
    pub fn add_device(self, device: &'a wgpu::Device) -> Builder<'a, (), V> {
        Builder {
            device: Some(device),
            label: self.label,
            layout: self.layout,
            vertex: self.vertex,
            primitive: self.primitive,
            depth_stencil: self.depth_stencil,
            multisample: self.multisample,
            fragment: self.fragment,
            multiview: self.multiview,
            cache: self.cache,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn add_label(self, label: &'a str) -> Builder<'a, D, V> {
        Builder {
            device: self.device,
            label: Some(label),
            layout: self.layout,
            vertex: self.vertex,
            primitive: self.primitive,
            depth_stencil: self.depth_stencil,
            multisample: self.multisample,
            fragment: self.fragment,
            multiview: self.multiview,
            cache: self.cache,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn add_layout(self, layout: &'a wgpu::PipelineLayout) -> Builder<'a, D, V> {
        Builder {
            device: self.device,
            label: self.label,
            layout: Some(layout),
            vertex: self.vertex,
            primitive: self.primitive,
            depth_stencil: self.depth_stencil,
            multisample: self.multisample,
            fragment: self.fragment,
            multiview: self.multiview,
            cache: self.cache,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn add_vertex(self, vertex: wgpu::VertexState<'a>) -> Builder<'a, D, ()> {
        Builder {
            device: self.device,
            label: self.label,
            layout: self.layout,
            vertex: Some(vertex),
            primitive: self.primitive,
            depth_stencil: self.depth_stencil,
            multisample: self.multisample,
            fragment: self.fragment,
            multiview: self.multiview,
            cache: self.cache,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn add_primitive(self, primitive: wgpu::PrimitiveState) -> Builder<'a, D, V> {
        Builder {
            device: self.device,
            label: self.label,
            layout: self.layout,
            vertex: self.vertex,
            primitive: Some(primitive),
            depth_stencil: self.depth_stencil,
            multisample: self.multisample,
            fragment: self.fragment,
            multiview: self.multiview,
            cache: self.cache,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn add_depth_stencil(self, depth_stencil: wgpu::DepthStencilState) -> Builder<'a, D, V> {
        Builder {
            device: self.device,
            label: self.label,
            layout: self.layout,
            vertex: self.vertex,
            primitive: self.primitive,
            depth_stencil: Some(depth_stencil),
            multisample: self.multisample,
            fragment: self.fragment,
            multiview: self.multiview,
            cache: self.cache,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn add_multisample(self, multisample: wgpu::MultisampleState) -> Builder<'a, D, V> {
        Builder {
            device: self.device,
            label: self.label,
            layout: self.layout,
            vertex: self.vertex,
            primitive: self.primitive,
            depth_stencil: self.depth_stencil,
            multisample: Some(multisample),
            fragment: self.fragment,
            multiview: self.multiview,
            cache: self.cache,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn add_fragment(self, fragment: wgpu::FragmentState<'a>) -> Builder<'a, D, V> {
        Builder {
            device: self.device,
            label: self.label,
            layout: self.layout,
            vertex: self.vertex,
            primitive: self.primitive,
            depth_stencil: self.depth_stencil,
            multisample: self.multisample,
            fragment: Some(fragment),
            multiview: self.multiview,
            cache: self.cache,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn add_multiview(self, multiview: std::num::NonZero<u32>) -> Builder<'a, D, V> {
        Builder {
            device: self.device,
            label: self.label,
            layout: self.layout,
            vertex: self.vertex,
            primitive: self.primitive,
            depth_stencil: self.depth_stencil,
            multisample: self.multisample,
            fragment: self.fragment,
            multiview: Some(multiview),
            cache: self.cache,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn add_cache(self, cache: &'a wgpu::PipelineCache) -> Builder<'a, D, V> {
        Builder {
            device: self.device,
            label: self.label,
            layout: self.layout,
            vertex: self.vertex,
            primitive: self.primitive,
            depth_stencil: self.depth_stencil,
            multisample: self.multisample,
            fragment: self.fragment,
            multiview: self.multiview,
            cache: Some(cache),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Builder<'_, (), ()> {
    pub fn build(self) -> wgpu::RenderPipeline {
        self.device
            .expect("Failed to create")
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: self.label,
                layout: self.layout,
                vertex: self
                    .vertex
                    .expect("unable to create render pipeline missing vertex"),
                primitive: self.primitive.unwrap_or_default(),
                depth_stencil: self.depth_stencil,
                multisample: self.multisample.unwrap_or_default(),
                fragment: self.fragment,
                multiview: self.multiview,
                cache: self.cache,
            })
    }
}
