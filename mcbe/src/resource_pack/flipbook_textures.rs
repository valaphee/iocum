use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FlipbookTexture {
    /// Path to texture.
    pub flipbook_texture: String,

    /// The index of the texture array inside the definition of that shortname.*
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub atlas_index: Option<u32>,

    /// The variant of the block's texture array inside the shortname's block
    /// variation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub atlas_tile_variant: Option<u32>,

    /// The shortname defined in the terrain_textures.json.
    pub atlas_tile: String,

    /// How fast frames should be changed. 20 ticks = 1 second.
    #[serde(default = "default_1", skip_serializing_if = "if_1")]
    pub ticks_per_frame: u32,

    /// List with numbers of frames which defines their order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frames: Vec<u32>,

    /// Sets the size of pixels. Default: 1.**
    #[serde(default = "default_1", skip_serializing_if = "if_1")]
    pub replicate: u32,

    /// Defines should frames transition be smooth or not. Default: true.
    #[serde(default = "default_true", skip_serializing_if = "if_true")]
    pub blend_frames: bool,
}

fn default_true() -> bool {
    true
}

fn default_1() -> u32 {
    1
}

fn if_true(value: &bool) -> bool {
    *value
}

fn if_1(value: &u32) -> bool {
    *value == 1
}
