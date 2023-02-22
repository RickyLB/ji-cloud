use dominator::{clone, html, Dom};

use utils::prelude::*;

use super::UnitIframe;
use futures_signals::signal::SignalExt;
use std::rc::Rc;

impl UnitIframe {
    pub fn render(self: Rc<Self>) -> Dom {
        let state = self;
        state.load_module_kind();

        html!("iframe" => web_sys::HtmlIFrameElement, {
            .prop("allow", "autoplay; fullscreen")
            .prop("slot", "main")
            .style("width", "100%")
            .style("height", "100%")
            .style("border", "none")
            .prop_signal("src", state.module_kind.signal().map(clone!(state => move |module_kind| {
                match module_kind {
                    None => String::new(),
                    Some(module_kind) => {
                        let path = Route::Unit(UnitRoute::Edit(module_kind, state.asset_id, state.module_id)).to_string();
                        let url = unsafe {
                            SETTINGS.get_unchecked()
                                .remote_target
                                .spa_iframe(&path)
                        };
                        url
                    },
                }
            })))
        })
    }
}
