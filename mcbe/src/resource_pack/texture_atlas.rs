use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, OneOrMany, PickFirst};

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureAtlas {
    pub resource_pack_name: String,
    pub texture_name: String,
    pub padding: u32,
    pub num_mip_levels: u32,
    pub texture_data: HashMap<String, TextureData>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct TextureData {
    #[serde_as(as = "OneOrMany<PickFirst<(DisplayFromStr, _)>>")]
    pub textures: Vec<Texture>,
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

impl Display for Texture {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.overlay_color.is_some() || self.tint_color.is_some() || !self.variations.is_empty()
        {
            return Err(Default::default());
        }
        write!(f, "{}", self.path)
    }
}

impl FromStr for Texture {
    type Err = std::fmt::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Texture {
            path: s.to_string(),
            overlay_color: None,
            tint_color: None,
            variations: vec![],
        })
    }
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
