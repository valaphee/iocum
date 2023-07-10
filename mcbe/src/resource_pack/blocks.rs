use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Blocks {
    pub format_version: String,

    #[serde(flatten)]
    pub blocks: HashMap<String, Block>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isotropic: Option<Face<bool>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textures: Option<Face<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub carried_textures: Option<Face<String>>,

    #[serde(default = "default_1p0", skip_serializing_if = "if_1p0")]
    pub brightness_gamma: f32,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

fn default_1p0() -> f32 {
    1.0
}

fn if_1p0(value: &f32) -> bool {
    *value == 1.0
}
