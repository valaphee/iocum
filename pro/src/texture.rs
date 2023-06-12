use bitflags::bitflags;
use bytemuck::{Contiguous, Pod, Zeroable};

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TextureHeader {
    pub flags: u16,
    pub mip_level_count: u8,
    pub format: u8,
    pub depth_or_array_layers: u8,
    pub usage: u8,
    pub payload_count: u8,
    pub u8_1: u8, // stencil?
    pub width: u16,
    pub height: u16,
    pub size: u32,
    pub u16_0: u16,
    pub u16_1: u16, // always 0, padding?
    pub u32_0: u32, // always 0, padding?
    pub u64_0: u64, // always 0, padding?
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TexturePayloadHeader {
    pub mip_level: u32,
    pub mip_level_count: u32,
    pub size: u32,
    pub u32_0: u32,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum TextureUsage {
    None,
    Terrain,
    Map,
    Tree,
    Hero,
    OverlayEffect,
    Effect,
    Weapon,
    Object,
    Unknown9,
    Unknown10,
    Ui,
    Unknown12,
    Unknown13,
    Unknown14,
    Sticker,
    Unknown16,
}

unsafe impl Contiguous for TextureUsage {
    type Int = u8;
    const MAX_VALUE: Self::Int = TextureUsage::None as u8;
    const MIN_VALUE: Self::Int = TextureUsage::Unknown16 as u8;
}

bitflags! {
    #[derive(Debug)]
    pub struct TextureFlags: u16 {
        const D1 = 1 << 0;
        const D2 = 1 << 1;
        const D3 = 1 << 2;
        const Cubemap = 1 << 3;
        const Unknown4 = 1 << 4;
        const Color = 1 << 5;
        const Array = 1 << 6;
        const Unknown7 = 1 << 7;
        const Unknown8 = 1 << 8;
        const Unknown9 = 1 << 9;
        const Unknown10 = 1 << 10;
    }
}
