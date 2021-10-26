use crate::domain::{audio::AudioId, image::ImageId};

pub use super::*;
use serde::{Deserialize, Serialize};
use serde_repr::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Design {
    /// Background layer
    pub bgs: Vec<String>,

    /// Stickers layer
    pub stickers: Vec<Sticker>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sticker {
    pub filename: String,
    pub transform_matrix: [f64; 16],
    /// hide and hide_toggle are mapped from the top sections 
    /// in "Houdini": HideOnTap, ShowOnTap, and ToggleOnTap
    /// start out hidden
    pub hide: bool,
    /// toggle hidden state
    #[serde(skip_serializing_if="Option::is_none")]
    pub hide_toggle: Option<HideToggle>,

    /// animation options are mapped from the bottom animation section
    #[serde(skip_serializing_if="Option::is_none")]
    pub animation: Option<Animation>,
    // associated audio
    #[serde(skip_serializing_if="Option::is_none")]
    pub audio_filename: Option<String>,

    /// override the size
    #[serde(skip_serializing_if="Option::is_none")]
    pub override_size: Option<(f64, f64)>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum HideToggle {
    /// only let the toggle fire once
    Once,
    /// let the toggle fire indefinitely
    Always,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Animation {
    /// do not let the animation loop
    pub once: bool,
    /// wait for tap before playing
    pub tap: bool,
}
