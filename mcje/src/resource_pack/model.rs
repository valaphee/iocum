use std::collections::HashMap;

use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Model {
    /// Loads a different model from the given path, in form of a resource
    /// location. If both "parent" and "elements" are set, the "elements" tag
    /// overrides the "elements" tag from the previous model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,

    /// Whether to use ambient occlusion (true - default), or not (false).
    /// Note:only works on Parent file
    #[serde(default = "default_true", skip_serializing_if = "if_true")]
    pub ambient_occlusion: bool,

    /// Holds the different places where item models are displayed.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub display: HashMap<String, Display>,

    /// Holds the textures of the model, in form of a resource location or can
    /// be another texture variable.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub textures: HashMap<String, String>,

    /// Contains all the elements of the model. They can have only cubic forms.
    /// If both "parent" and "elements" are set, the "elements" tag overrides
    /// the "elements" tag from the previous model.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<Element>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<Group>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Display {
    /// Specifies the rotation of the model according to the scheme [x, y, z].
    #[serde(default)]
    pub rotation: Vec3,

    /// Specifies the position of the model according to the scheme [x, y, z].
    /// The values are clamped between -80 and 80.
    #[serde(default)]
    pub translation: Vec3,

    /// Specifies the scale of the model according to the scheme [x, y, z]. If
    /// the value is greater than 4, it is displayed as 4.
    #[serde(default)]
    pub scale: Vec3,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Element {
    /// Start point of a cuboid according to the scheme [x, y, z]. Values must
    /// be between -16 and 32.
    pub from: Vec3,

    /// Stop point of a cuboid according to the scheme [x, y, z]. Values must be
    /// between -16 and 32.
    pub to: Vec3,

    /// Defines the rotation of an element.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Rotation>,

    /// Defines if shadows are rendered (true - default), not (false).
    #[serde(default = "default_true", skip_serializing_if = "if_true")]
    pub shade: bool,

    /// Holds all the faces of the cuboid. If a face is left out, it does not
    /// render.
    pub faces: HashMap<FaceEnum, Face>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rotation {
    /// Sets the center of the rotation according to the scheme [x, y, z].
    pub origin: Vec3,

    /// Specifies the direction of rotation, can be "x", "y" or "z".
    pub axis: Axis,

    /// Specifies the angle of rotation. Can be 45 through -45 degrees in 22.5
    /// degree increments.
    pub angle: f32,

    /// Specifies whether or not to scale the faces across the whole block. Can
    /// be true or false. Defaults to false.
    #[serde(default, skip_serializing_if = "if_false")]
    pub rescale: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FaceEnum {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Face {
    /// Defines the area of the texture to use according to the scheme [x1, y1,
    /// x2, y2]. The texture behavior is inconsistent if UV extends below 0 or
    /// above 16. If the numbers of x1 and x2 are swapped (e.g. from 0, 0, 16,
    /// 16 to 16, 0, 0, 16), the texture flips. UV is optional, and if not
    /// supplied it automatically generates based on the element's position.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uv: Option<[f32; 4]>,

    /// Specifies the texture in form of the texture variable prepended with a
    /// #.
    pub texture: String,

    /// Specifies whether a face does not need to be rendered when there is a
    /// block touching it in the specified position. The position can be: down,
    /// up, north, south, west, or east. It also determines the side of the
    /// block to use the light level from for lighting the face, and if unset,
    /// defaults to the side.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cull_face: Option<FaceEnum>,

    /// Rotates the texture by the specified number of degrees. Can be 0, 90,
    /// 180, or 270. Defaults to 0. Rotation does not affect which part of the
    /// texture is used. Instead, it amounts to permutation of the selected
    /// texture vertexes (selected implicitly, or explicitly though uv).
    #[serde(default, skip_serializing_if = "if_0")]
    pub rotation: u32,

    /// Determines whether to tint the texture using a hardcoded tint index. The
    /// default value, -1, indicates not to use the tint. Any other number is
    /// provided to BlockColors to get the tint value corresponding to that
    /// index. However, most blocks do not have a tint value defined (in which
    /// case white is used). Furthermore, no vanilla block currently uses
    /// multiple tint values, and thus the tint index value is ignored (as long
    /// as it is set to something other than -1); it could be used for modded
    /// blocks that need multiple distinct tint values in the same block though.
    #[serde(default = "default_n1", skip_serializing_if = "if_n1")]
    pub tint_index: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged, rename_all = "lowercase")]
pub enum Group {
    Group {
        name: String,
        origin: Vec3,
        children: Vec<Group>,
    },
    Element(u32),
}

fn default_true() -> bool {
    true
}

fn default_n1() -> i32 {
    -1
}

fn if_false(value: &bool) -> bool {
    !*value
}

fn if_true(value: &bool) -> bool {
    *value
}

fn if_0(value: &u32) -> bool {
    *value == 0
}

fn if_n1(value: &i32) -> bool {
    *value == -1
}
