use serde::{Deserialize, Serialize};

use crate::{behavior_pack::block::Block, resource_pack::geometry::Geometry};

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionedData {
    pub format_version: String,

    #[serde(flatten)]
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Data {
    #[serde(rename = "minecraft:block")]
    Block(Block),
    #[serde(rename = "minecraft:geometry")]
    Geometry(Vec<Geometry>),
}
