use dominator::{clone, html, Dom};
use futures_signals::signal::{Signal, SignalExt};
use std::rc::Rc;
use utils::{
    events,
    languages::{Language, JIG_LANGUAGES},
};

use crate::curation::jig::state::CurationJig;

const STR_LANGUAGE_LABEL: &str = "Language of instructions";
const STR_LANGUAGE_PLACEHOLDER: &str = "Select language";

impl CurationJig {
    pub fn render_languages(self: &Rc<Self>) -> Dom {
        let state = Rc::clone(self);
        html!("input-select", {
            .prop("slot", "language")
            .prop("label", STR_LANGUAGE_LABEL)
            .prop("placeholder", STR_LANGUAGE_PLACEHOLDER)
            .prop_signal("value", language_value_signal(state.clone()))
            // .prop_signal("error", {
            //     (map_ref! {
            //         let submission_tried = state.submission_tried.signal(),
            //         let value = state.jig.language.signal_cloned()
            //             => (*submission_tried, value.clone())
            //     })
            //         .map(|(submission_tried, value)| {
            //             submission_tried && value.is_empty()
            //         })
            // })
            .children(JIG_LANGUAGES.iter().map(clone!(state => move |language| {
                render_language(language, state.clone())
            })))
        })
    }
}

fn render_language(Language(lang_code, land_label): &Language, state: Rc<CurationJig>) -> Dom {
    html!("input-select-option", {
        .text(land_label)
        .prop_signal("selected", state.jig.language.signal_cloned().map(clone!(lang_code => move |selected_lang| {
            lang_code == selected_lang
        })))
        .event(clone!(state, lang_code => move |evt: events::CustomSelectedChange| {
            if evt.selected() {
                state.jig.language.set(lang_code.to_string());
            }
        }))
    })
}

fn language_value_signal(state: Rc<CurationJig>) -> impl Signal<Item = &'static str> {
    state
        .jig
        .language
        .signal_cloned()
        .map(move |selected_language| {
            match JIG_LANGUAGES
                .iter()
                .find(|Language(lang_code, _)| lang_code == &selected_language)
            {
                Some(lang) => lang.1,
                None => "",
            }
        })
}
