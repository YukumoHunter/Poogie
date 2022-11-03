use std::mem::size_of;

use ash::vk;
use glam::{Mat4, Vec3, Vec4};
use gpu_allocator::vulkan::Allocator;
use memoffset::offset_of;

use super::{buffer::Buffer, device::Device};

#[derive(Debug)]
pub struct VertexInputDescription {
    pub bindings: Vec<vk::VertexInputBindingDescription>,
    pub attributes: Vec<vk::VertexInputAttributeDescription>,
    pub flags: vk::PipelineVertexInputStateCreateFlags,
}

pub trait HasVertexInputDescription {
    fn describe() -> VertexInputDescription;
}

#[derive(Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
}

impl HasVertexInputDescription for Vertex {
    fn describe() -> VertexInputDescription {
        #[allow(clippy::fn_to_numeric_cast_with_truncation)]
        let main_binding = vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build();

        let position_attr = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(offset_of!(Vertex, position) as u32)
            .build();

        // let normal_attr = vk::VertexInputAttributeDescription::builder()
        //     .binding(0)
        //     .location(1)
        //     .format(vk::Format::R32G32B32_SFLOAT)
        //     .offset(offset_of!(Vertex, normal) as u32)
        //     .build();

        let color_attr = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(offset_of!(Vertex, color) as u32)
            .build();

        VertexInputDescription {
            bindings: vec![main_binding],
            attributes: vec![position_attr, color_attr],
            // attributes: vec![position_attr, normal_attr, color_attr],
            flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MeshPushConstants {
    pub data: Vec4,
    pub render_matrix: Mat4,
}

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub vertex_buffer: Buffer,
}

impl Mesh {
    pub fn new(allocator: &mut Allocator, device: &Device) -> Self {
        let positions = [
            Vec3::new(0.6, -0.6, 0.0),
            Vec3::new(-0.6, -0.6, 0.0),
            Vec3::new(0.0, 0.6, 0.0),
        ];

        let normals = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ];

        let colors = [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];

        let vertices = positions
            .into_iter()
            .zip(normals.into_iter())
            .zip(colors.into_iter())
            .map(|((position, normal), color)| Vertex {
                position,
                normal,
                color,
            })
            .collect::<Vec<Vertex>>();

        let vertex_buffer = Buffer::new(
            allocator,
            device,
            vertices.len() * size_of::<Vertex>(),
            vk::BufferUsageFlags::VERTEX_BUFFER,
            "triangle",
        );

        let alloc = vertex_buffer.allocation.as_ref().unwrap();

        // get the underlying mapped pointer and copy vertex data inside
        unsafe {
            (alloc.mapped_ptr().unwrap().as_ptr() as *mut Vertex)
                .copy_from_nonoverlapping(vertices.as_ptr(), vertices.len())
        };

        Mesh {
            vertices,
            vertex_buffer,
        }
    }
}
