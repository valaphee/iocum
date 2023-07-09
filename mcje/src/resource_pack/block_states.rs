use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockStates {
    /// Holds the names of all the variants of the block.
    Variants(HashMap<String, Variant>),

    /// Used instead of variants to combine models based on block state
    /// attributes.
    Multipart(Vec<Case>),
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Variant(#[serde_as(as = "OneOrMany<_>")] pub Vec<Model>);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Model {
    /// Specifies the path to the model file of the block, in form of a resource
    /// location.
    pub model: String,

    /// Rotation of the model on the x-axis in increments of 90 degrees.
    #[serde(default, skip_serializing_if = "if_0")]
    pub x: u32,

    /// Rotation of the model on the y-axis in increments of 90 degrees.
    #[serde(default, skip_serializing_if = "if_0")]
    pub y: u32,

    /// Can be true or false (default). Locks the rotation of the texture of a
    /// block, if set to true. This way the texture does not rotate with the
    /// block when using the x and y-tags above.
    #[serde(default, skip_serializing_if = "if_false")]
    pub uv_lock: bool,

    /// Sets the probability of the model for being used in the game, defaults
    /// to 1 (=100%). If more than one model is used for the same variant, the
    /// probability is calculated by dividing the individual model's weight by
    /// the sum of the weights of all models. (For example, if three models are
    /// used with weights 1, 1, and 2, then their combined weight would be 4
    /// (1+1+2). The probability of each model being used would then be
    /// determined by dividing each weight by 4: 1/4, 1/4 and 2/4, or 25%, 25%
    /// and 50%, respectively.)
    #[serde(default = "default_1", skip_serializing_if = "if_1")]
    pub weight: u32,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Case {
    /// A list of cases that have to be met for the model to be applied. If
    /// unset, the model always applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<When>,

    /// Determines the model(s) to apply and its properties. There can be one
    /// model or an array of models. If set to an array, the model is chosen
    /// randomly from the options given, with each option being specified in
    /// separate subsidiary -tags.
    #[serde_as(as = "OneOrMany<_>")]
    pub apply: Vec<Model>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum When {
    One(HashMap<String, String>),
    Many(HashMap<String, Vec<HashMap<String, String>>>),
}

fn default_1() -> u32 {
    1
}

fn if_false(value: &bool) -> bool {
    !*value
}

fn if_0(value: &u32) -> bool {
    *value == 0
}

fn if_1(value: &u32) -> bool {
    *value == 1
}
