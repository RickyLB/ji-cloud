use std::{ops::Range, rc::Rc};

use dominator::{clone, html, with_node, Dom};
use futures_signals::{
    map_ref,
    signal::{Signal, SignalExt},
    signal_vec::{SignalVec, SignalVecExt},
};
use shared::domain::category::Category;
use utils::{events, unwrap::UnwrapJiExt};
use web_sys::HtmlInputElement;

use crate::overlay::handle::OverlayHandle;

use super::state::CategoriesInput;

impl CategoriesInput {
    pub fn render(self: Rc<Self>, slot: Option<&str>) -> Dom {
        let state = self;

        html!("input" => HtmlInputElement, {
            .with_node!(input_elem => {
                .prop("placeholder", &state.placeholder)
                .apply_if(slot.is_some(), |dom| {
                    dom.prop("slot", slot.unwrap_ji())
                })
                .event(clone!(state, input_elem => move |_:events::Input| {
                    state.input.set(input_elem.value());
                }))
                .event(clone!(state, input_elem => move |_:events::Focus| {
                    input_elem.set_value("");
                    state.focused.set(true);
                }))
                .event(clone!(state, input_elem => move |_:events::Blur| {
                    state.on_focus_out(&input_elem);
                }))
                .future(state.input_value_signal().for_each(clone!(input_elem => move|value| {
                    if let Some(value) = value {
                        input_elem.set_value(&value);
                    };
                    async {}
                })))
                .apply(OverlayHandle::lifecycle(
                    clone!(input_elem => move || {
                        html!("overlay-content", {
                            .prop("target", &input_elem)
                            .prop("targetAnchor", "bm")
                            .prop("contentAnchor", "tm")
                            .prop("styled", true)
                            .child_signal(state.focused.signal().map(clone!(state, input_elem => move|focused| {
                                match focused {
                                    false => None,
                                    true => {
                                        Some(html!("div", {
                                            .style("min-width", format!("{}px", input_elem.scroll_width()))
                                            .style("max-height", "300px")
                                            .event(clone!(state, input_elem => move |_:events::FocusOut| {
                                                state.on_focus_out(&input_elem);
                                            }))
                                            .children_signal_vec(
                                                state.filtered_options_signal().map(clone!(state => move|(category, range)| {
                                                    state.render_option(&category, range)
                                                }))
                                            )
                                            .after_inserted(clone!(state => move |elem| {
                                                *state.overlay_content_elem.borrow_mut() = Some(elem);
                                            }))
                                            .after_removed(clone!(state => move|_|{
                                                *state.overlay_content_elem.borrow_mut() = None;
                                            }))
                                        }))
                                    }
                                }
                            })))
                        })
                    })
                ))
            })
        })
    }

    fn render_option(self: &Rc<Self>, category: &Category, range: Range<usize>) -> Dom {
        let state = self;
        let category_id = category.id;
        html!("input-autocomplete-option", {
            .apply_if(range.start > 0, |dom| {
                dom.text(&category.name[..range.start])
            })
            .child(html!("b", {
                .text(&category.name[range.start..range.end])
            }))
            .apply_if(range.end < category.name.len(), |dom| {
                dom.text(&category.name[range.end..])
            })
            .prop_signal("selected", state.selected_categories.signal_ref(clone!(category_id => move |selected_categories| {
                selected_categories.contains(&category_id)
            })))
            .event(clone!(state => move |_: events::CustomSelectedChange| {
                let mut categories = state.selected_categories.lock_mut();
                if categories.contains(&category_id) {
                    categories.remove(&category_id);
                } else {
                    categories.insert(category_id);
                }
            }))
        })
    }

    fn filtered_options_signal(self: &Rc<Self>) -> impl SignalVec<Item = (Category, Range<usize>)> {
        let state = self;

        state
            .input
            .signal_ref(clone!(state => move |input| {
                state.all_categories.iter().filter_map(|category| {

                    let category_name = category.name.to_lowercase();
                    let input = input.to_lowercase();
                    let contains = category_name.contains(&input);

                    match contains {
                        false => None,
                        true => {
                            let start = category_name.find(&input).unwrap_ji();
                            let range = start..(start + input.len());
                            Some((category.clone(), range))
                        },
                    }

                }).collect()
            }))
            .to_signal_vec()
    }

    // `None` if the input value should not be touched
    fn input_value_signal(self: &Rc<Self>) -> impl Signal<Item = Option<String>> {
        map_ref! {
            let focused = self.focused.signal(),
            let value = self.value.signal_cloned() => move {
                match focused {
                    true => None,
                    false => Some(value.clone()),
                }
            }
        }
    }
}
