use glm::*;
use wgpu::util::DeviceExt;

#[repr(C)]
pub struct Vertex {
    position: Vec3,
    color: Vec3,
}

pub struct Mesh {
    pub buffer: wgpu::Buffer,
    pub offset: u64,
}

impl Vertex {
    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub fn make_triangle(device: &wgpu::Device) -> wgpu::Buffer {
    let vertices: [Vertex; 3] = [
        Vertex {
            position: vec3(-0.75, -0.75, 0.0),
            color: vec3(0.0, 1.0, 0.0),
        },
        Vertex {
            position: vec3(0.75, -0.75, 0.0),
            color: vec3(1.0, 1.0, 0.0),
        },
        Vertex {
            position: vec3(0.0, 0.75, 0.0),
            color: vec3(0.5, 0.0, 1.0),
        },
    ];

    let buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Triangle vertex buffer"),
        contents: unsafe { crate::util::as_u8_slice(&vertices) },
        usage: wgpu::BufferUsages::VERTEX,
    };

    device.create_buffer_init(&buffer_descriptor)
}

pub fn make_quad(device: &wgpu::Device) -> Mesh {
    let vertices: [Vertex; 4] = [
        Vertex {
            position: vec3(-1.0, -1.0, 0.0),
            color: vec3(0.0, 1.0, 0.0),
        },
        Vertex {
            position: vec3(1.0, -1.0, 0.0),
            color: vec3(1.0, 1.0, 0.0),
        },
        Vertex {
            position: vec3(1.0, 1.0, 0.0),
            color: vec3(1.0, 0.0, 0.0),
        },
        Vertex {
            position: vec3(-1.0, 1.0, 0.0),
            color: vec3(0.0, 0.0, 0.0),
        },
    ];

    let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

    let bytes_1: &[u8] = unsafe { crate::util::as_u8_slice(&vertices) };
    let bytes_2: &[u8] = unsafe { crate::util::as_u8_slice(&indices) };
    let bytes_merged = &[bytes_1, bytes_2].concat();

    let buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Quad vertex &  index buffer"),
        contents: bytes_merged,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
    };

    let buffer = device.create_buffer_init(&buffer_descriptor);
    let offset = bytes_1.len().try_into().unwrap();

    Mesh { buffer, offset }
}
