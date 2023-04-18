use std::rc::Rc;

use js_sys::Reflect;
use shared::domain::{module::body::_groups::design::VideoHost, pro_dev::unit::Video};
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;

use crate::edit::pro_dev::unit_editor::UnitValue;

use super::state::AddVideo;

impl AddVideo {
    pub fn save(self: &Rc<Self>, host: VideoHost) {
        let state = Rc::clone(self);

        self.loader.load(async move {
            let video = Video {
                host,
                start_at: None,
                end_at: None,
            };

            state
                .add_unit_value_state
                .unit_editor_state
                .value
                .set(UnitValue::Video(Some(video.clone())));

            state
                .add_unit_value_state
                .unit_editor_state
                .changed
                .set(true);
        })
    }
}

pub fn set_error(elem: &HtmlElement, error: bool) {
    let _ = Reflect::set(
        elem,
        &JsValue::from_str("error"),
        &JsValue::from_bool(error),
    );
}
