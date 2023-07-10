use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Blocks {
    pub format_version: String,

    #[serde(flatten)]
    pub blocks: HashMap<String, Block>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isotropic: Option<Face<bool>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textures: Option<Face<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub carried_textures: Option<Face<String>>,

    pub brightness_gamma: f32,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Face<V> {
    Cube {
        up: V,
        down: V,
        north: V,
        south: V,
        east: V,
        west: V,
    },
    CubeBottomTop {
        up: V,
        down: V,
        side: V,
    },
    CubeAll(V),
}
