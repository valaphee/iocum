use std::collections::HashMap;

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Geometry {
    pub description: Description,

    /// Bones define the 'skeleton' of the mob: the parts that can be animated,
    /// and to which geometry and other bones are attached.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bones: Vec<Bone>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Description {
    /// Entity definition and Client Block definition files refer to this
    /// geometry via this identifier.
    pub identifier: String,

    /// Width of the visibility bounding box (in model space units).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_bounds_width: Option<f32>,

    /// Height of the visible bounding box (in model space units).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_bounds_height: Option<f32>,

    /// Offset of the visibility bounding box from the entity location point (in
    /// model space units).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_bounds_offset: Option<Vec3>,

    /// Assumed width in texels of the texture that will be bound to this
    /// geometry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture_width: Option<u32>,

    /// Assumed height in texels of the texture that will be bound to this
    /// geometry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture_height: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bone {
    /// Animation files refer to this bone via this identifier.
    pub name: String,

    /// Bone that this bone is relative to.  If the parent bone moves, this bone
    /// will move along with it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,

    /// The bone pivots around this point (in model space units).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pivot: Option<Vec3>,

    /// This is the initial rotation of the bone around the pivot, pre-animation
    /// (in degrees, x-then-y-then-z order).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Vec3>,

    /// Mirrors the UV's of the unrotated cubes along the x axis, also causes
    /// the east/west faces to get flipped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirror: Option<bool>,

    /// Grow this box by this additive amount in all directions (in model space
    /// units).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inflate: Option<f32>,

    /// This is the list of cubes associated with this bone.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cubes: Vec<Cube>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cube {
    /// This point declares the unrotated lower corner of cube (smallest x/y/z
    /// value in model space units).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Vec3>,

    /// The cube extends this amount relative to its origin (in model space
    /// units).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<Vec3>,

    /// The cube is rotated by this amount (in degrees, x-then-y-then-z order)
    /// around the pivot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Vec3>,

    /// If this field is specified, rotation of this cube occurs around this
    /// point, otherwise its rotation is around the center of the box.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pivot: Option<Vec3>,

    /// Grow this box by this additive amount in all directions (in model space
    /// units), this field overrides the bone's inflate field for this cube
    /// only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inflate: Option<f32>,

    /// Mirrors this cube about the unrotated x axis (effectively flipping the
    /// east / west faces), overriding the bone's 'mirror' setting for this
    /// cube.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirror: Option<bool>,

    /// This is an alternate per-face uv mapping which specifies each face of
    /// the cube.  Omitting a face will cause that face to not get drawn.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub uv: HashMap<FaceKey, Face>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FaceKey {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Face {
    /// Specifies the uv origin for the face. For this face, it is the
    /// upper-left corner, when looking at the face with y being up.
    pub uv: Vec2,

    /// The face maps this many texels from the uv origin. If not specified, the
    /// box dimensions are used instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uv_size: Option<Vec2>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material_instance: Option<String>,
}
