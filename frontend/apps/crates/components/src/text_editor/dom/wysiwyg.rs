use crate::text_editor::font_css_converter::font_from_css;
use dominator::{clone, html, Dom};
use futures_signals::signal::SignalExt;
use std::rc::Rc;
use utils::prelude::*;

use super::super::{
    state::TextEditor,
    wysiwyg_types::{ControlsChange, WysiwygControlsChange},
};

impl TextEditor {
    pub fn render_wysiwyg(self: &Rc<Self>) -> Dom {
        let state = self;
        html!("wysiwyg-base", {
            .property_signal("theme", state.theme_id.signal_cloned().map(|theme_id| {
                theme_id.as_str_id()
            }))
            .after_inserted(clone!(state => move |wysiwyg_ref| {
                state.set_wysiwyg_ref(wysiwyg_ref);
            }))
            .after_removed(clone!(state => move |_| {
                state.clear_wysiwyg_ref();
            }))
            .event(clone!(state => move |e: WysiwygControlsChange| {
                let value = e.value();
                // log::info!("{:?}", &value);
                let mut controls = state.controls.lock_mut();
                match value {
                    ControlsChange::Font(font) => controls.font = font_from_css(&font),
                    ControlsChange::Element(element) => controls.element = element,
                    ControlsChange::Weight(weight) => controls.weight = weight,
                    ControlsChange::Align(align) => controls.align = align,
                    ControlsChange::FontSize(font_size) => controls.font_size = font_size,
                    ControlsChange::Color(color) => controls.color = color,
                    ControlsChange::HighlightColor(highlight_color) => controls.highlight_color = highlight_color,
                    ControlsChange::BoxColor(box_color) => controls.box_color = box_color,
                    ControlsChange::Direction(direction) => controls.direction = direction,
                    ControlsChange::Italic(italic) => controls.italic = italic,
                    ControlsChange::Underline(underline) => controls.underline = underline,
                };
            }))
            .event(clone!(state => move |e: events::CustomChange| {
                let value = e.value();
                if let Some(on_change) = &state.callbacks.on_change.as_ref() {
                    on_change(&value);
                }
            }))
            .event(clone!(state => move |_: events::CustomBlur| {
                if let Some(on_blur) = &state.callbacks.on_blur.as_ref() {
                    on_blur();
                }
            }))
        })
    }
}
