use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureAtlas {
    pub resource_pack_name: String,
    pub texture_name: String,
    pub padding: u32,
    pub num_mip_levels: u32,
    pub texture_data: HashMap<String, TextureData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureData {
    pub textures: Texture,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Texture {
    pub path: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlay_color: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tint_color: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variations: Vec<Variation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Variation {
    pub path: String,

    #[serde(default, skip_serializing_if = "if_0")]
    pub weight: u32,
}

fn if_0(value: &u32) -> bool {
    *value == 0
}
