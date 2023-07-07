use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub enum Block {
    /// Holds the names of all the variants of the block.
    Variants(HashMap<String, Vec<Model>>),

    /// Used instead of variants to combine models based on block state
    /// attributes.
    Multipart(Vec<Case>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    /// Specifies the path to the model file of the block, in form of a resource
    /// location.
    pub model: String,

    /// Rotation of the model on the x-axis in increments of 90 degrees.
    pub x: u32,

    /// Rotation of the model on the y-axis in increments of 90 degrees.
    pub y: u32,

    /// Can be true or false (default). Locks the rotation of the texture of a
    /// block, if set to true. This way the texture does not rotate with the
    /// block when using the x and y-tags above.
    pub uvlock: bool,

    /// Sets the probability of the model for being used in the game, defaults
    /// to 1 (=100%). If more than one model is used for the same variant, the
    /// probability is calculated by dividing the individual model's weight by
    /// the sum of the weights of all models. (For example, if three models are
    /// used with weights 1, 1, and 2, then their combined weight would be 4
    /// (1+1+2). The probability of each model being used would then be
    /// determined by dividing each weight by 4: 1/4, 1/4 and 2/4, or 25%, 25%
    /// and 50%, respectively.)
    pub weight: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Case {
    /// A list of cases that have to be met for the model to be applied. If
    /// unset, the model always applies.
    pub when: HashMap<String, Value>,

    /// Determines the model(s) to apply and its properties. There can be one
    /// model or an array of models. If set to an array, the model is chosen
    /// randomly from the options given, with each option being specified in
    /// separate subsidiary -tags.
    pub apply: Vec<Model>,
}
