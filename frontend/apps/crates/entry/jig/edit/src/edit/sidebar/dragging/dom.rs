use dominator::{html, Dom};

use std::rc::Rc;

use crate::edit::sidebar::state::State as SidebarState;
use futures_signals::signal::SignalExt;

use super::super::spot::state::State as ItemState;

pub struct DraggingDom {}

impl DraggingDom {
    pub fn render(sidebar: Rc<SidebarState>) -> Dom {
        html!("empty-fragment", {
            .child_signal(
                sidebar
                    .drag
                    .signal_cloned()
                    .map(|drag| {

                        //In order to avoid re-rendering the entire overlay on every movement
                        //We always return it when a drag is initialized
                        //Even if it hasn't passed the threshhold for actual dragging yet
                        //And tie the display style to the actual dragging state
                        drag.map(|state| {
                            let module = &state.module;

                            html!("jig-edit-sidebar-module", {
                                .style_signal("display", state.inner.active_signal().map(|active| {
                                    if active { "block" } else { "none" }
                                }))
                                .style("position", "fixed")
                                .style("top", "0")
                                .style("left", "0")
                                .style("z-index", "9999")
                                .style_signal("transform", state.inner.transform_signal())
                                .property("selected", true)
                                .property("index", module.index as u32)
                                .property("module", module.kind_str())
                                .property("dragging", true)
                                .child(html!("jig-edit-sidebar-module-window", {
                                    .property("slot", "window")
                                    .property_signal("state", ItemState::window_state_signal(Rc::clone(module)))
                                }))
                            })
                        })
                    })
            )
        })
    }
}
