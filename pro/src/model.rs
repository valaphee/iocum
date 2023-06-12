use bytemuck::{Pod, Zeroable};

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ModelChunkHeader {
    pub u64_0: u64,
    pub material_offset: u64,
    pub bounding_box: [f32; 16],
    pub u64_1: u64,
    pub u64_2: u64,
    pub material_count: u16,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ModelRenderMeshHeader {
    pub vertex_buffer_descriptor_offset: u64,
    pub index_buffer_descriptor_offset: u64,
    pub submesh_descriptor_offset: u64,
    pub vertex_count: u32,
    pub submesh_count: u32,
    pub material_count: u16,
    pub vertex_buffer_descriptor_count: u8,
    pub index_buffer_descriptor_count: u8,
    pub u8_0: u8,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SubmeshDescriptor {
    pub f32_0: [f32; 3],
    pub f32_1: [f32; 4],
    pub f32_2: [f32; 3],
    pub u64_0: u64,
    pub u64_1: u64,
    pub f32_3: f32,
    pub vertex_start: u32,
    pub index_start: u32,
    pub u32_0: [u32; 7],
    pub index_count: u32,
    pub indices_to_draw: u32,
    pub vertices_to_draw: u16,
    pub u16_0: u16,
    pub index_buffer: u8,
    pub u8_0: [u8; 7],
    pub vertex_buffer: u8,
    pub u8_1: u8,
    pub u8_2: u8,
    pub u8_3: u8,
    pub u32_1: u32,
    pub u8_4: u8,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct VertexBufferDescriptor {
    pub vertex_count: u32,
    pub u32_0: u32,
    pub u8_0: u8,
    pub u8_1: u8,
    pub vertex_element_descriptor_count: u8,
    pub u8_2: u8,
    pub u32_1: u32,
    pub vertex_element_descriptor_offset: u64,
    pub u64_0: u64,
    pub u64_1: u64,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct VertexElementDescriptor {
    pub type_: u8,
    pub index: u8,
    pub format: u8,
    pub slot: u8,
    pub u32_0: u8,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct IndexBufferDescriptor {
    pub index_count: u32,
    pub format: u32,
    pub u64_0: u64,
}
