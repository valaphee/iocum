use std::collections::HashMap;

use glam::Vec3;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, EnumMap};

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    /// List of characteristics of a block that are applicable to all
    /// permutations of the block. The description MUST contain an identifier;
    /// the other fields are optional. View the other fields in Block
    /// Description.
    pub description: Description,

    /// List of all components that are used in this block. View the list of
    /// components in Block Components List. Block trigger components can also
    /// be specified in the components section, but they require the Holiday
    /// Creator Features experimental toggle to be ON in order for them to work.
    /// But you can view the list of block trigger components in Block Trigger
    /// List.
    #[serde_as(as = "EnumMap")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<Component>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permutations: Vec<Permutation>,
}

/// The block "description" is a section inside the "minecraft:block" section of
/// a custom block's behavior resource_pack JSON file that contains a list of
/// characteristics of a block that are applicable to all permutations of the
/// block. The description MUST contain an identifier to identify the block by;
/// the other fields are optional.
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Description {
    /// The identifier for this block. The name must include a namespace and
    /// must not use the Minecraft namespace unless overriding a Vanilla block.
    pub identifier: String,

    /// Map of key/value pairs that maps the property name (key) to an array of
    /// all possible values for that property (value). Learn how to use block
    /// properties in Block Properties and Permutations.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, Property>,

    /// Specifies the menu category and group for the block, which determine
    /// where this block is placed in the inventory and crafting table container
    /// screens. If this field is omitted, the block will not appear in the
    /// inventory or crafting table container screens.
    #[serde(default, skip_serializing_if = "MenuCategory::is_default")]
    pub menu_category: MenuCategory,

    /// Block traits are designed to be a shortcut for creators to use Vanilla
    /// block states without needing to define and manage a series of events or
    /// triggers on custom blocks. While custom states and permutations can be
    /// used to set multiple variations of the same block (whether it's on/off,
    /// is flammable, etc.), states that are exposed through traits allow you to
    /// access the inherent data certain Vanilla blocks hold.
    #[serde_as(as = "EnumMap")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<Trait>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Property {
    Bool(Vec<bool>),
    Int(Vec<u32>),
    IntRange { values: Range },
    Enum(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Range {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MenuCategory {
    /// Determines which category this block will be placed under in the
    /// inventory and crafting table container screens. Options are
    /// "construction", "nature", "equipment", "items", and "none". If omitted
    /// or "none" is specified, the block will not appear in the inventory or
    /// crafting table container screens.
    pub category: Category,

    /// Specifies the language file key that maps to which
    /// expandable/collapsible group this block will be a part of within a
    /// category. If this field is omitted, or there is no group whose name
    /// matches the loc string, this block will be placed standalone in the
    /// given category.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

impl MenuCategory {
    fn is_default(&self) -> bool {
        matches!(self.category, Category::None)
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Construction,
    Nature,
    Equipment,
    Items,
    #[default]
    None,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Trait {
    /// placement_direction can add states containing information about the
    /// player's rotation when the block is placed. For example, if a block
    /// using placement_direction is placed while the player is facing south,
    /// the state value will be "south". This will allow a data-driven block to
    /// replicate the rotation behavior of a furnace, pumpkin, or terracotta
    /// block. Note that while the block contains information, permutations will
    /// need to be configured to determine how the block is placed/looks/acts.
    /// There are two states that can be included with this trait:
    /// - minecraft:cardinal_direction is a 4-value state containing the
    ///   cardinal-facing direction of the player when the block was placed. THe
    ///   values for this state are 'north', 'south', 'east', and 'west'.
    /// - minecraft:facing_direction is a 6-value state containing the overall
    ///   direction of the player when the block was placed. The values for this
    ///   state are 'down', 'up', 'north', 'south', 'east', and 'west'.
    /// This trait can also be configured with a 'y_rotation_offset' where an
    /// axis-aligned angle may be specified via degree (e.g. 90.0). This causes
    /// the state within the trait to store a rotated value. In other words,
    /// with a rotation offset of 90.0, a block placed when the player is facing
    /// south would have a state of 'east'. This rotation offset only applies to
    /// the horizontal state values (north, south, east, west).
    #[serde(rename = "minecraft:placement_direction")]
    PlacementDirection {
        enabled_states: Vec<String>,
        y_rotation_offset: f32,
    },
    /// placement_position contains information about where the player placed
    /// the block. This allows a block to replicate the upside-down placement of
    /// slabs and stairs, as well as the attachment behavior of torches and
    /// vines. There are two states that can be included with this trait:
    /// - minecraft:block_face is a 6-value state representing the face on which
    ///   the block was placed. Values for this state are 'up', 'down', 'north',
    ///   'south', 'east', and 'west'. For example, if a block using the
    ///   minecraft:block_face is placed on the south face of a neighboring
    ///   block, the state value will be 'south'.
    /// - minecraft:vertical_half is a 2-value state specifying whether a block
    ///   was placed in the top or bottom half of a block. The values for this
    ///   state are 'bottom' and 'top'. For blocks using
    ///   minecraft:vertical_half, if the block is placed on the 'down' face of
    ///   a block or above the midline of a horizontal face, the state value
    ///   will be 'top'.
    #[serde(rename = "minecraft:placement_position")]
    PlacementPosition { enabled_states: Vec<String> },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Component {
    /// Defines the area of the block that collides with entities. If set to
    /// true, default values are used. If set to false, the block's collision
    /// with entities is disabled. If this component is omitted, default values
    /// are used. Experimental toggles required: Holiday Creator Features (in
    /// format versions before 1.19.50).
    #[serde(rename = "minecraft:collision_box")]
    CollisionBox {
        /// Minimal position of the bounds of the collision box. origin is
        /// specified as [x, y, z] and must be in the range (-8, 0, -8) to (8,
        /// 16, 8), inclusive.
        origin: Vec3,

        /// Size of each side of the collision box. Size is specified as [x, y,
        /// z]. origin + size must be in the range (-8, 0, -8) to (8, 16, 8),
        /// inclusive.
        size: Vec3,
    },

    /// Describes the destructible by explosion properties for this block. If
    /// set to true, the block will have the default explosion resistance. If
    /// set to false, this block is indestructible by explosion. If the
    /// component is omitted, the block will have the default explosion
    /// resistance.
    #[serde(rename = "minecraft:destructible_by_explosion")]
    DestructibleByExplosion {
        /// Describes how resistant the block is to explosion. Greater values
        /// mean the block is less likely to break when near an explosion (or
        /// has higher resistance to explosions). The scale will be different
        /// for different explosion power levels. A negative value or 0 means it
        /// will easily explode; larger numbers increase level of resistance.
        explosion_resistance: f32,
    },

    /// Describes the destructible by mining properties for this block. If set
    /// to true, the block will take the default number of seconds to destroy.
    /// If set to false, this block is indestructible by mining. If the
    /// component is omitted, the block will take the default number of seconds
    /// to destroy.
    #[serde(rename = "minecraft:destructible_by_mining")]
    DestructibleByMining {
        /// Sets the number of seconds it takes to destroy the block with base
        /// equipment. Greater numbers result in greater mining times.
        seconds_to_destroy: f32,
    },

    /// Describes the flammable properties for this block. If set to true,
    /// default values are used. If set to false, or if this component is
    /// omitted, the block will not be able to catch on fire naturally from
    /// neighbors, but it can still be directly ignited.
    #[serde(rename = "minecraft:flammable")]
    Flammable {
        /// A modifier affecting the chance that this block will catch flame
        /// when next to a fire. Values are greater than or equal to 0, with a
        /// higher number meaning more likely to catch on fire. For a
        /// catch_chance_modifier greater than 0, the fire will continue to burn
        /// until the block is destroyed (or it will burn forever if the
        /// "destroy_chance_modifier" is 0). If the catch_chance_modifier is 0,
        /// and the block is directly ignited, the fire will eventually burn out
        /// without destroying the block (or it will have a chance to be
        /// destroyed if destroy_chance_modifier is greater than 0). The default
        /// value of 5 is the same as that of Planks.
        catch_chance_modifier: u32,

        /// A modifier affecting the chance that this block will be destroyed by
        /// flames when on fire. Values are greater than or equal to 0, with a
        /// higher number meaning more likely to be destroyed by fire. For a
        /// destroy_chance_modifier of 0, the block will never be destroyed by
        /// fire, and the fire will burn forever if the catch_chance_modifier is
        /// greater than 0. The default value of 20 is the same as that of
        /// Planks.
        destroy_change_modifier: u32,
    },

    /// Describes the friction for this block in a range of (0.0-0.9). Friction
    /// affects an entity's movement speed when it travels on the block. Greater
    /// value results in more friction.
    #[serde(rename = "minecraft:friction")]
    Friction(f32),

    /// The description identifier of the geometry file to use to render this
    /// block. This identifier must match an existing geometry identifier in any
    /// of the currently loaded resource packs. Experimental toggles required:
    /// Holiday Creator Features (in format versions before 1.19.40).
    #[serde(rename = "minecraft:geometry")]
    Geometry {
        identifier: String,

        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        bone_visibility: HashMap<String, bool>,
    },

    /// The amount that light will be dampened when it passes through the block,
    /// in a range (0-15). Higher value means the light will be dampened more.
    #[serde(rename = "minecraft:light_dampening")]
    LightDampening(u8),

    /// The amount of light this block will emit in a range (0-15). Higher value
    /// means more light will be emitted.
    #[serde(rename = "minecraft:light_emission")]
    LightEmission(u8),

    /// Sets the color of the block when rendered to a map. The color is
    /// represented as a hex value in the format "#RRGGBB". May also be
    /// expressed as an array of [R, G, B] from 0 to 255. If this component is
    /// omitted, the block will not show up on the map.
    #[serde(rename = "minecraft:map_color")]
    MapColor(String),

    /// The material instances for a block. Maps face or material_instance names
    /// in a geometry file to an actual material instance. You can assign a
    /// material instance object to any of these faces: "up", "down", "north",
    /// "south", "east", "west", or "*". You can also give an instance the name
    /// of your choosing such as "my_instance", and then assign it to a face by
    /// doing "north":"my_instance". Experimental toggles required: Holiday
    /// Creator Features (in format versions before 1.19.40).
    #[serde(rename = "minecraft:material_instances")]
    MaterialInstances(HashMap<String, MaterialInstance>),

    /// Defines the area of the block that is selected by the player's cursor.
    /// If set to true, default values are used. If set to false, this block is
    /// not selectable by the player's cursor. If this component is omitted,
    /// default values are used. Experimental toggles required: Holiday Creator
    /// Features (in format versions before 1.19.60).
    #[serde(rename = "minecraft:selection_box")]
    SelectionBox { origin: Vec3, size: Vec3 },

    /// The block's translation around the center of the cube in degrees. The
    /// rotation order is [x, y, z]. Angles need to be in multiples of 90.
    /// Experimental toggles required: Holiday Creator Features (in format
    /// versions before 1.19.80).
    #[serde(rename = "minecraft:transformation")]
    Transformation {
        translation: Vec3,
        scale: Vec3,
        rotation: Vec3,
    },

    /// Specifies that a unit cube is to be used with tessellation.
    #[serde(rename = "minecraft:unit_cube")]
    UnitCube {},
}

/// minecraft:material_instances is a JSON Object component that specifies the
/// material instances for a block. The object contains a map that maps face or
/// material_instance names in a geometry file to an actual material instance.
/// You can assign a material instance object to any of these faces: up, down,
/// north, south, east, west, or *. You can also give an instance the name of
/// your choosing such as my_instance, and then assign it to a face by doing
/// "north": "my_instance".
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaterialInstance {
    /// Should this material have ambient occlusion applied when lighting? If
    /// true, shadows will be created around and underneath the block.
    #[serde(default = "default_true", skip_serializing_if = "if_true")]
    pub ambient_occlusion: bool,

    /// Should this material be dimmed by the direction it's facing?
    #[serde(default = "default_true", skip_serializing_if = "if_true")]
    pub face_dimming: bool,

    /// The render method to use.
    #[serde(default, skip_serializing_if = "RenderMethod::if_default")]
    pub render_method: RenderMethod,

    /// Texture name for the material.
    pub texture: String,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMethod {
    /// Used for a regular block texture without an alpha layer. Does not allow
    /// for transparency or translucency.
    #[default]
    Opaque,

    /// Used for completely disabling backface culling.
    DoubleSided,

    /// Used for a block like stained glass. Allows for transparency and
    /// translucency (slightly transparent textures).
    Blend,

    /// Used for a block like the vanilla (unstained) glass. Does not allow for
    /// translucency, only fully opaque or fully transparent textures. Also
    /// disables backface culling.
    AlphaTest,
}

impl RenderMethod {
    fn if_default(&self) -> bool {
        matches!(self, RenderMethod::Opaque)
    }
}

/// Consider block permutations as variations of the same block, while block
/// properties are the flags that can be changed and queried in order to
/// determine which permutation a block should use. Block permutations and
/// properties go hand in hand, so let's look at how they are used together.
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Permutation {
    /// A Molang expression that evaluates to true or false to determine if this
    /// permutation should be used. For permutation conditions you are limited
    /// to using one Molang query: "query.block_property()".
    pub condition: String,

    /// List of all components that are used in this permutation.
    #[serde_as(as = "EnumMap")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<Component>,
}

fn default_true() -> bool {
    true
}

fn if_true(value: &bool) -> bool {
    *value
}
