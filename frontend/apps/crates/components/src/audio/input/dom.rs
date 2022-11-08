use super::{
    components::{add_method, delete, main_action, main_content},
    state::*,
};
use dominator::{clone, html, Dom};
use futures_signals::{map_ref, signal::SignalExt};
use std::rc::Rc;
use utils::prelude::*;

/// Note - the external audio signal will only be valid for first render
/// This can be fixed by instead storing an AsyncLoader which reacts to the
/// Signal changes and sets an internal Mutable
impl AudioInput {
    pub fn render(state: Rc<Self>, slot: Option<&str>) -> Dom {
        html!("audio-input", {
            .apply_if(slot.is_some(), move |dom| {
                dom.prop("slot", slot.unwrap_ji())
            })
            .prop_signal("mode", state.mode.signal_cloned().map(get_element_mode))
            .children(&mut [
                add_method::render(state.clone(), AudioInputAddMethod::Record),
                add_method::render(state.clone(), AudioInputAddMethod::Upload),
                delete::render(state.clone()),
            ])
            .children_signal_vec({
                let sig = map_ref! {
                    let mode = state.mode.signal_cloned(),
                    let add_method = state.add_method.signal_cloned()
                        => (mode.clone(), *add_method)
                };

                sig.map(clone!(state => move |(mode, add_method)| {
                    vec![
                        html!("empty-fragment", {
                            .prop("slot", "main-action")
                            .child(main_action::render(state.clone(), mode.clone(), add_method))
                        }),
                        main_content::render(state.clone(), mode, add_method),
                    ]
                }))
                .to_signal_vec()


            })
        })
    }
}

fn get_element_mode(mode: AudioInputMode) -> String {
    match mode {
        AudioInputMode::Recording | AudioInputMode::Uploading => String::from("active"),
        AudioInputMode::Empty => String::from("default"),
        AudioInputMode::Stopped(_) => String::from("success"),
        AudioInputMode::Playing(_) => String::from("done"),
    }
}
