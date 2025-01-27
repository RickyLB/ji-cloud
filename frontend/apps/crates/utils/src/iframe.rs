use crate::{keyboard::KeyEvent, unwrap::UnwrapJiExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use shared::domain::{
    jig::player::{ModuleConfig, Seconds},
    module::{
        body::{ModuleAssist, ModuleAssistType},
        LiteModule, ModuleId,
    },
};
use std::cell::Cell;
use wasm_bindgen::prelude::*;

pub const IFRAME_DATA_PARAM: &str = "iframe_data";

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum IframeTarget {
    Top,
    Parent,
}

thread_local! {
    pub(super) static PLAYER_TARGET:Cell<IframeTarget> = Cell::new(IframeTarget::Parent);
    pub(super) static EDITOR_TARGET:Cell<IframeTarget> = Cell::new(IframeTarget::Top);
}

pub fn get_player_target() -> IframeTarget {
    PLAYER_TARGET.with(|p| p.get())
}
pub fn get_editor_target() -> IframeTarget {
    EDITOR_TARGET.with(|p| p.get())
}

// call this to change the target for msg.try_post_message_player()
// by default it is Top
pub fn set_player_target(target: IframeTarget) {
    PLAYER_TARGET.with(move |p| p.set(target))
}
pub fn set_editor_target(target: IframeTarget) {
    EDITOR_TARGET.with(move |p| p.set(target))
}

#[wasm_bindgen(
    inline_js = "export function is_in_iframe() { return window && window.parent && window.location !== window.parent.location; }"
)]
extern "C" {
    pub fn is_in_iframe() -> bool;
}

pub trait IframeMessageExt {
    fn try_post_message_to_top(&self) -> Result<(), JsValue>;

    fn try_post_message_to_parent(&self) -> Result<(), JsValue>;

    fn try_post_message_to_player(&self) -> Result<(), JsValue>;

    fn try_post_message_to_editor(&self) -> Result<(), JsValue>;
}

macro_rules! impl_iframe_msg_ext {
    ($s:ident) => {
        impl<T: Serialize> IframeMessageExt for $s<T> {
            fn try_post_message_to_top(&self) -> Result<(), JsValue>
            where
                Self: Into<JsValue>,
            {
                let window = web_sys::window().unwrap_ji();
                let top = window.top()?.unwrap_ji();

                top.post_message(&self.into(), "*")
            }
            fn try_post_message_to_parent(&self) -> Result<(), JsValue>
            where
                Self: Into<JsValue>,
            {
                let window = web_sys::window().unwrap_ji();
                let parent = window.parent()?.unwrap_ji();

                parent.post_message(&self.into(), "*")
            }

            fn try_post_message_to_player(&self) -> Result<(), JsValue>
            where
                Self: Into<JsValue>,
            {
                match get_player_target() {
                    IframeTarget::Top => self.try_post_message_to_top(),
                    IframeTarget::Parent => self.try_post_message_to_parent(),
                }
            }

            fn try_post_message_to_editor(&self) -> Result<(), JsValue>
            where
                Self: Into<JsValue>,
            {
                match get_editor_target() {
                    IframeTarget::Top => self.try_post_message_to_top(),
                    IframeTarget::Parent => self.try_post_message_to_parent(),
                }
            }
        }
    };
}

impl_iframe_msg_ext!(IframeInit);
impl_iframe_msg_ext!(IframeAction);

/// Init is used for bootstrapping and passing initial loaded data
#[derive(Serialize, Deserialize, Debug)]
pub struct IframeInit<T> {
    pub data: T,
}

impl<T> IframeInit<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl IframeInit<EmptyMessage> {
    pub fn empty() -> IframeInit<EmptyMessage> {
        IframeInit {
            data: EmptyMessage {},
        }
    }
}

/// Needed to avoid accidentally creating a null over the wire
/// which would be (de)serialized to anything
#[derive(Serialize, Deserialize, Debug)]
pub struct EmptyMessage {}

impl<T: Serialize> From<IframeInit<T>> for JsValue {
    fn from(msg: IframeInit<T>) -> Self {
        (&msg).into()
    }
}

impl<T: Serialize> From<&IframeInit<T>> for JsValue {
    fn from(msg: &IframeInit<T>) -> Self {
        serde_wasm_bindgen::to_value(msg).unwrap_ji()
    }
}

impl<T: DeserializeOwned> From<JsValue> for IframeInit<T> {
    fn from(msg: JsValue) -> Self {
        serde_wasm_bindgen::from_value(msg).unwrap_ji()
    }
}

pub fn should_get_iframe_data() -> bool {
    crate::routes::is_param_bool(IFRAME_DATA_PARAM)
}

/// Action is used for passing runtime messages
#[derive(Serialize, Deserialize, Debug)]
pub struct IframeAction<T> {
    pub data: T,
}

impl<T> IframeAction<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[wasm_bindgen(inline_js = "export function temp_log(val) { console.log(val); }")]
extern "C" {
    fn temp_log(val: &JsValue);
}

impl<T: Serialize> From<IframeAction<T>> for JsValue {
    fn from(msg: IframeAction<T>) -> Self {
        (&msg).into()
    }
}

impl<T: Serialize> From<&IframeAction<T>> for JsValue {
    fn from(msg: &IframeAction<T>) -> Self {
        serde_wasm_bindgen::to_value(msg).unwrap_ji()
    }
}

impl<T: DeserializeOwned> From<JsValue> for IframeAction<T> {
    fn from(msg: JsValue) -> Self {
        serde_wasm_bindgen::from_value(msg).unwrap_ji()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum JigToModulePlayerMessage {
    TimerDone,
    // remove play and pause? might need for video
    Play,
    Pause,
    /// Sent if the player is configured to forward navigation events to
    /// the module
    Previous,
    /// Sent if the player is configured to forward navigation events to
    /// the module
    Next,
    ModuleAssistDone(ModuleAssistType),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ModuleToJigPlayerMessage {
    AddPoints(u32),
    Start(ModuleConfig),
    ResetTimer(Seconds),
    PauseTimer,
    UnpauseTimer,
    Previous,
    Next,
    JumpToIndex(usize),
    JumpToId(ModuleId),
    /// Useful for acting on key press events such as arrow keys for navigating between activities
    KeyEvent(KeyEvent),

    /// Optional module assistance, and it's type
    ModuleAssist(Option<(ModuleAssist, ModuleAssistType)>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ModuleToJigEditorMessage {
    AppendModule(LiteModule),
    Next,
    /// Whenever a modules completion status changes, i.e. meets the minimum required content
    Complete(ModuleId, bool),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ModuleToAssetEditorMessage {
    Publish,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AssetPlayerToPlayerPopup {
    Close,
    // used for course player to hide close button when jig is playing since the jig has it's own button
    CloseButtonShown(bool),
}
