use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct McMeta {
    /// Contains data for the animation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation: Option<Animation>,
}

/// Block, item, particle, painting, item frame, and status effect icon
/// (assets/minecraft/textures/mob_effect) textures support animation by placing
/// each additional frame below the last. The animation is then controlled using
/// a .mcmeta file in JSON format with the same name and .png at the end of the
/// filename, in the same directory. For example, the .mcmeta file for stone.png
/// would be stone.png.mcmeta.
#[derive(Debug, Serialize, Deserialize)]
pub struct Animation {
    /// If true, Minecraft generates additional frames between frames with a
    /// frame time greater than 1 between them. Defaults to false.
    #[serde(default, skip_serializing_if = "if_false")]
    pub interpolate: bool,

    /// The width of the tile, as a direct ratio rather than in pixels. This is
    /// unused in vanilla's files but can be used by resource packs to have
    /// frames that are not perfect squares.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,

    /// The height of the tile as a ratio rather than in pixels. This is unused
    /// in vanilla's files but can be used by resource packs to have frames that
    /// are not perfect squares.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,

    /// Sets the default time for each frame in increments of one game tick.
    /// Defaults to 1.
    #[serde(default = "default_1", skip_serializing_if = "if_1")]
    pub frametime: u32,

    /// Contains a list of frames. Defaults to displaying all the frames from
    /// top to bottom.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frames: Vec<Frame>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Frame {
    /// A number corresponding to position of a frame from the top, with the top
    /// frame being 0.
    Index(u32),
    /// A frame specifies a frame with additional data.
    IndexTime {
        /// A number corresponding to position of a frame from the top, with the
        /// top frame being 0.
        index: u32,
        /// The time in ticks to show this frame, overriding "frametime" above.
        time: u32,
    },
}

fn default_1() -> u32 {
    1
}

fn if_false(value: &bool) -> bool {
    !*value
}

fn if_1(value: &u32) -> bool {
    *value == 1
}
