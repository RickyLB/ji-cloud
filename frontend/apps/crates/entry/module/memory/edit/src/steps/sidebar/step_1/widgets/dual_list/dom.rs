use dominator::{html, Dom, clone};
use std::rc::Rc;
use utils::prelude::*;
use wasm_bindgen::prelude::*;
use futures_signals::{
    map_ref,
    signal::{Mutable, SignalExt},
    signal_vec::{MutableVec, SignalVec, SignalVecExt},
};
use components::tooltip::types::*;
use super::state::*;
use unicode_segmentation::UnicodeSegmentation;
use crate::data::state::Mode;

pub struct DualListDom {}
impl DualListDom {
    pub fn render(state: Rc<State>) -> Dom { 
        html!("sidebar-widget-dual-list", {
            .children(&mut [

                html!("button-text", {
                    .property("slot", "clear")
                    .text(crate::strings::STR_CLEAR)
                    .event(clone!(state => move |evt:events::Click| {
                        state.clear();
                    }))
                }),
                html!("button-sidebar", {
                    .property("slot", "input-buttons")
                    .property("mode", "keyboard")
                }),
                html!("button-sidebar", {
                    .property("slot", "input-buttons")
                    .property("mode", "dicta")
                }),
                html!("button-sidebar", {
                    .property("slot", "input-buttons")
                    .property("mode", "sefaria")
                }),
                html!("button-rect", {
                    .property("color", "grey")
                    .property("size", "small")
                    .property("iconAfter", "done")
                    .property("slot", "done-btn")
                    .text(crate::strings::STR_DONE)
                    .event(clone!(state => move |evt:events::Click| {
                        match state.derive_list() {
                            Ok(list) => {
                                state.app.replace_dual_list(list);
                            },
                            Err(err) => {
                                log::info!("why not rendering...");
                                state.app.overlay.tooltips.list_error.set(Some(
                                    TooltipData::Error(TooltipError {
                                        elem: state.error_element_ref.borrow().as_ref().unwrap_ji().clone(), 
                                        placement: Placement::Right, 
                                        max_width: Some(185.0),
                                        move_strategy: MoveStrategy::None,
                                        slot: None,
                                        body: crate::strings::error::STR_SINGLE_LIST_NUM_WORDS.to_string(),
                                        on_close: Some(Rc::new(Box::new(clone!(state => move || {
                                            state.app.overlay.tooltips.list_error.set(None); 
                                        })))),
                                    })
                                ));
                            }
                        }
                    }))
                }),
                render_column(state.clone(), ColumnSide::Left),
                render_column(state.clone(), ColumnSide::Right),
            ])
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColumnSide {
    Left,
    Right
}

impl ColumnSide {
    fn side_prop(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
    fn col_index(&self) -> usize { 
        match self {
            Self::Left => 0, 
            Self::Right => 1, 
        }
    }

    fn mutable(&self, state:&State) -> Rc<MutableVec<Mutable<String>>> {
        match self {
            Self::Left => state.left.clone(),
            Self::Right => state.right.clone() 
        }
    }
}

fn render_column(state: Rc<State>, side: ColumnSide) -> Dom {
    let mode = state.app.mode.get().unwrap_ji();

    html!("sidebar-widget-dual-list-column", {
        .property("slot", side.side_prop())
        .property("side", side.side_prop())
        .property("header", crate::strings::STR_HEADER(side, mode)) 
        .children_signal_vec(
            side.mutable(&state).signal_vec_cloned()
                .enumerate()
                .map(clone!(state => move |(index, value)| {
                    //couldn't get it to compile by moving this into an Rc at a higher level
                    //but closures should be cheap in a JS VM anyway :P
                    let constrain_cb = Closure::wrap(Box::new(clone!(state => move |text:String| {
                        state.app.limit_text(crate::config::DUAL_LIST_CHAR_LIMIT, text)
                    })) as Box<dyn FnMut(String) -> String>);

                    Dom::with_state(constrain_cb, clone!(state => move |constrain_cb| {
                        let row = index.get().unwrap_or_default();
                        let mode = state.app.mode.get_cloned().unwrap_ji();
                        let rows = match mode {
                            Mode::Riddles => 2,
                            _ => 1
                        };
                        html!("sidebar-widget-dual-list-input", {
                            .property("rows", rows)
                            .property_signal("value", {
                                map_ref! {
                                    let value = value.signal_cloned(),
                                    let is_placeholder = state.is_placeholder.signal()
                                        => move {
                                            if *is_placeholder {
                                                match crate::config::get_dual_list_init_word(row, side.col_index()) {
                                                    Some(s) => s.to_string(),
                                                    None => "".to_string()
                                                }
                                            } else {
                                                value.clone()
                                            }
                                        }
                                }
                            })
                            .property("constrain", constrain_cb.as_ref())
                            .property_signal("placeholder", state.is_placeholder.signal())
                            .event(clone!(state => move |evt:events::Focus| {
                                //log::info!("got focus!");
                                state.is_placeholder.set_neq(false);
                            }))
                            .event(clone!(state => move |evt:events::CustomInput| {
                                value.set_neq(evt.value());
                            }))
                            .after_inserted(clone!(index, state, side => move |elem| {
                                if side == ColumnSide::Right && row == 2 {
                                    *state.error_element_ref.borrow_mut() = Some(elem);
                                }

                            }))
                        })
                    }))
                }))
        )
    })
}
