use dominator::{html, Dom, clone, with_node};
use crate::data::{
    actions,
    state::*,
    raw,
};
use std::rc::Rc;
use utils::{routes::*, events, settings::SETTINGS, iframe::IframeInit};
use wasm_bindgen::prelude::*;
use futures_signals::{
    map_ref,
    signal::SignalExt
};
use shared::domain::jig::ModuleKind;
use dominator_helpers::events::Message;

pub struct PreviewDom {}
impl PreviewDom {
    pub fn render(state:Rc<State>) -> Dom {
        let url = {
            let route:String = Route::Module(ModuleRoute::Play(
                    ModuleKind::Memory, 
                    state.jig_id.clone(), 
                    state.module_id.clone())
                                            ).into();

            let url = unsafe {
                SETTINGS.get_unchecked()
                    .remote_target
                    .spa_iframe(&route)
            };

            format!("{}?iframe_data=true", url)
        };

        log::info!("{}", url);

        //TODO - change to custom element / component
        html!("iframe" => web_sys::HtmlIFrameElement, {
            .property("slot", "main")
            .style("width", "100%")
            .style("height", "100%")
            .property("src", url.clone())
            .with_node!(elem => {
                .global_event(clone!(state, url => move |evt:Message| {

                    if let Ok(_) = evt.try_serde_data::<IframeInit<()>>() {
                        //Iframe is ready and sent us a message, let's send one back!
                        let data = state.history.get_current().game_data.unwrap_throw(); 
                        let msg:IframeInit<raw::GameData> = IframeInit::new(data); 
                        let window = elem.content_window().unwrap_throw();
                        window.post_message(&msg.into(), &url);
                    } else {
                        log::info!("hmmm got other iframe message...");
                    }
                }))
            })
        })
    }
}
