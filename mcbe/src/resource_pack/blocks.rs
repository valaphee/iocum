use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Blocks {
    pub format_version: String,
    pub blocks: HashMap<String, Block>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    pub textures: String,
    pub carried_textures: String,
    pub brightness_gamma: f32,
    pub sound: String,
}
