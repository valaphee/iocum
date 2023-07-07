use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    /// Loads a different model from the given path, in form of a resource
    /// location. If both "parent" and "elements" are set, the "elements" tag
    /// overrides the "elements" tag from the previous model.
    pub parent: String,

    /// Whether to use ambient occlusion (true - default), or not (false).
    /// Note:only works on Parent file
    pub ambient_occlusion: bool,

    /// Holds the different places where item models are displayed.
    pub display: HashMap<String, Display>,

    /// Holds the textures of the model, in form of a resource location or can
    /// be another texture variable.
    pub textures: HashMap<String, String>,

    /// Contains all the elements of the model. They can have only cubic forms.
    /// If both "parent" and "elements" are set, the "elements" tag overrides
    /// the "elements" tag from the previous model.
    pub elements: Vec<Element>,

    pub groups: Vec<Group>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Display {
    /// Specifies the rotation of the model according to the scheme [x, y, z].
    pub rotation: [f32; 3],

    /// Specifies the position of the model according to the scheme [x, y, z].
    /// The values are clamped between -80 and 80.
    pub translation: [f32; 3],

    /// Specifies the scale of the model according to the scheme [x, y, z]. If
    /// the value is greater than 4, it is displayed as 4.
    pub scale: [f32; 3],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Element {
    /// Start point of a cuboid according to the scheme [x, y, z]. Values must
    /// be between -16 and 32.
    pub from: [f32; 3],

    /// Stop point of a cuboid according to the scheme [x, y, z]. Values must be
    /// between -16 and 32.
    pub to: [f32; 3],

    /// Defines the rotation of an element.
    pub rotation: Rotation,

    /// Defines if shadows are rendered (true - default), not (false).
    pub shade: bool,

    /// Holds all the faces of the cuboid. If a face is left out, it does not
    /// render.
    pub faces: HashMap<Face, Uv>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rotation {
    /// Sets the center of the rotation according to the scheme [x, y, z].
    pub origin: [f32; 3],

    /// Specifies the direction of rotation, can be "x", "y" or "z".
    pub axis: Axis,

    /// Specifies the angle of rotation. Can be 45 through -45 degrees in 22.5
    /// degree increments.
    pub angle: f32,

    /// Specifies whether or not to scale the faces across the whole block. Can
    /// be true or false. Defaults to false.
    pub rescale: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Face {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Uv {
    /// Defines the area of the texture to use according to the scheme [x1, y1,
    /// x2, y2]. The texture behavior is inconsistent if UV extends below 0 or
    /// above 16. If the numbers of x1 and x2 are swapped (e.g. from 0, 0, 16,
    /// 16 to 16, 0, 0, 16), the texture flips. UV is optional, and if not
    /// supplied it automatically generates based on the element's position.
    pub uv: [f32; 4],

    /// Specifies the texture in form of the texture variable prepended with a
    /// #.
    pub texture: String,

    /// Specifies whether a face does not need to be rendered when there is a
    /// block touching it in the specified position. The position can be: down,
    /// up, north, south, west, or east. It also determines the side of the
    /// block to use the light level from for lighting the face, and if unset,
    /// defaults to the side.
    pub cull_face: Face,

    /// Rotates the texture by the specified number of degrees. Can be 0, 90,
    /// 180, or 270. Defaults to 0. Rotation does not affect which part of the
    /// texture is used. Instead, it amounts to permutation of the selected
    /// texture vertexes (selected implicitly, or explicitly though uv).
    pub rotation: i32,

    /// Determines whether to tint the texture using a hardcoded tint index. The
    /// default value, -1, indicates not to use the tint. Any other number is
    /// provided to BlockColors to get the tint value corresponding to that
    /// index. However, most blocks do not have a tint value defined (in which
    /// case white is used). Furthermore, no vanilla block currently uses
    /// multiple tint values, and thus the tint index value is ignored (as long
    /// as it is set to something other than -1); it could be used for modded
    /// blocks that need multiple distinct tint values in the same block though.
    pub tint_index: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub origin: [f32; 3],
    pub children: Vec<u32>,
}
