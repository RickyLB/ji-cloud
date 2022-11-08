use super::{actions, state::*};
use crate::images::meta::state::{MutableImage, State as MetaState};
use dominator::{clone, html, Dom};
use std::rc::Rc;
use utils::events;

use components::image::tag::ImageTag;
use shared::domain::meta::MetadataResponse;
use strum::IntoEnumIterator;

pub struct GeneralDom {}

impl GeneralDom {
    pub fn render(
        meta_state: Rc<MetaState>,
        image: Rc<MutableImage>,
        metadata: Rc<MetadataResponse>,
    ) -> Dom {
        let state = Rc::new(State::new(meta_state, image, metadata));

        html!("image-meta-section-general", {
            .children(state.metadata.image_styles.iter().map(|style| {
                let id = style.id;
                html!("input-checkbox", {
                    .prop("slot", "styles")
                    .prop("label", &style.display_name)
                    .prop_signal("checked", state.style_selected(id))
                    .event(clone!(state, id => move |evt:events::CustomToggle| {
                        actions::toggle_style(state.clone(), id, evt.value());
                    }))
                })
            }))
            .children(state.metadata.age_ranges.iter().map(|age_range| {
                let id = age_range.id;
                html!("input-checkbox", {
                    .prop("slot", "age_ranges")
                    .prop("label", &age_range.display_name)
                    .prop_signal("checked", state.age_range_selected(id))
                    .event(clone!(state, id => move |evt:events::CustomToggle| {
                        actions::toggle_age_range(state.clone(), id, evt.value());
                    }))
                })
            }))
            .children(state.metadata.affiliations.iter().map(|affiliation| {
                let id = affiliation.id;
                html!("input-checkbox", {
                    .prop("slot", "affiliations")
                    .prop("label", &affiliation.display_name)
                    .prop_signal("checked", state.affiliation_selected(id))
                    .event(clone!(state, id => move |evt:events::CustomToggle| {
                        actions::toggle_affiliation(state.clone(), id, evt.value());
                    }))
                })
            }))
            .children(ImageTag::iter().map(|tag| {
                html!("input-checkbox", {
                    .prop("slot", "tags")
                    .prop("label", tag.display_name())
                    .prop_signal("checked", state.tag_selected(tag.as_index()))
                    .event(clone!(state, tag => move |evt:events::CustomToggle| {
                        actions::toggle_tag(state.clone(), tag.as_index(), evt.value());
                    }))
                })
            }))
        })
    }
}
