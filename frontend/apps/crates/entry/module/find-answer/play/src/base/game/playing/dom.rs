use components::{
    audio::mixer::AUDIO_MIXER,
    module::_common::play::prelude::ModulePlayPhase,
    traces::{
        bubble::TraceBubble,
        show::{TracesShow, TracesShowMode},
    },
};
use dominator::{clone, html, Dom};
use futures_signals::{
    signal::SignalExt,
    signal_vec::{self, SignalVecExt},
};
use gloo_timers::future::TimeoutFuture;
use js_sys::Reflect;
use shared::domain::module::body::{
    find_answer::QuestionField, ModuleAssist, ModuleAssistType, _groups::design::TraceKind,
};
use std::rc::Rc;
use utils::{prelude::*, resize::resize_info_signal};
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;

use super::state::*;

const HINT_TIME: u32 = 500;

pub fn render(state: Rc<PlayState>) -> Dom {
    html!("empty-fragment", {
        // We don't want the cursor to change when the student mouses over an answer trace, or even an incorrect trace.
        // Make the entire playable area use the pointer cursor.
        .style("cursor", "pointer")
        .future(state.incorrect_choice_count.signal_cloned().for_each(clone!(state => move |count| {
            let state = state.clone();
            async move {
                match state.game.base.settings.n_attempts {
                    Some(attempts) if count >= attempts => {
                        state.incorrect_choice_count.set(0);
                        state.show_hint.set_neq(true);
                        TimeoutFuture::new(HINT_TIME).await;
                        state.show_hint.set_neq(false);
                    },
                    _ => {}
                }
            }
        })))
        .future(state.selection_audio.signal_cloned().for_each(clone!(state => move |audio| {
            let state = state.clone();
            async move {
                if let Some(audio) = audio {
                    AUDIO_MIXER.with(move |mixer| {
                        *state.selection_audio_handle.borrow_mut() = Some(mixer.play_on_ended(audio.into(), false, clone!(state => move || {
                            state.selection_audio.set(None);
                            *state.selection_audio_handle.borrow_mut() = None;
                            state.clone().evaluate_end();

                            // Necessary to do this here for cases where an incorrect trace does *not*
                            // have specific audio attached for some reason, and the incorrect audio
                            // plays instead.
                            state.clone().remove_incorrect_highlights();
                        })));
                    });
                }
            }
        })))
        .future(state.game.base.module_phase.signal_cloned().dedupe().for_each(clone!(state => move |phase| {
            // Only play audio and update the text if we're in the playing phase.
            if let ModulePlayPhase::Playing = phase {
                // Play audio if we have any for this question
                if let Some(audio) = &state.question.question_audio {
                    if is_in_iframe() {
                        // If we're in an iframe, send a message to the player to play the audio from
                        // the module assist component. This allows students to replay the question audio.
                        let module_assist = ModuleAssist {
                            text: None,
                            audio: Some(audio.clone()),
                        };
                        let msg = IframeAction::new(ModuleToJigPlayerMessage::ModuleAssist(Some((module_assist, ModuleAssistType::InActivity))));
                        let _ = msg.try_post_message_to_player();
                    } else {
                        // Otherwise, play the audio directly from the activity. It isn't possible to
                        // replay the audio in this case.
                        AUDIO_MIXER.with(|mixer| {
                            mixer.play_oneshot(audio.into());
                        });
                    }
                }

                // Update the question sticker if it is set and the question has text
                if let QuestionField::Text(question_index) = state.game.base.question_field {
                    let (sticker_ref, measurer_ref) = state.game.base.sticker_refs.get(question_index).unwrap_ji();
                    let sticker_ref = sticker_ref.get().unwrap_ji();
                    let measurer_ref = measurer_ref.get().unwrap_ji();

                    // This is weird. If we use "", then subsequent calls to set textValue don't work correctly.
                    let question_text = if state.question.question_text.is_empty() {
                        " "
                    } else {
                        &state.question.question_text
                    };

                    Reflect::set(
                        sticker_ref,
                        &JsValue::from_str("textValue"),
                        &JsValue::from_str(question_text)
                    ).unwrap_ji();

                    Reflect::set(
                        measurer_ref,
                        &JsValue::from_str("textValue"),
                        &JsValue::from_str(question_text)
                    ).unwrap_ji();
                }
            }

            async {}
        })))
        .child_signal(state.game.base.module_phase.signal_cloned().map(clone!(state => move |phase| {
            // Only render the banner if we're in the playing phase
            let should_render = matches!(phase, ModulePlayPhase::Playing)
                && !state.question.question_text.is_empty()
                && state.game.base.question_field.is_dynamic();
            if should_render {
                Some(html!("question-banner", {
                    .text(&state.question.question_text)
                }))
            } else {
                None
            }
        })))
        .child_signal(state.show_hint.signal_cloned().map(clone!(state => move |show| {
            if show {
                Some(TracesShow::render(TracesShow::new(
                    state.traces.iter().filter(|t| t.kind == TraceKind::Correct).map(|t| t.inner.clone()).collect(),
                    TracesShowMode::Cutout,
                    TracesShow::on_select_noop()
                )))
            } else {
                None
            }
        })))
        .child(TracesShow::render(TracesShow::new(
            state.traces
                .iter()
                .map(|t| t.inner.clone())
                .collect(),
            TracesShowMode::HiddenSolidMap(state.selected_set.clone()),
            Some(clone!(state => move |index| {
                if !state.ended.get() {
                    PlayState::select(state.clone(), index);
                }
            }))
        )))
        .event(clone!(state => move |evt: events::Click| {
            // Check for incorrect choice.
            if !state.ended.get() {
                if let Some(target) = evt.target() {
                    let target: JsValue = target.into();
                    let element: HtmlElement = target.into();
                    let tag_name = element.tag_name().to_lowercase();
                    let tag_name = tag_name.as_str();
                    // SVGs created for traces are either path, ellipse or rect. So check that the student hasn't
                    // clicked on any of those types of SVG to determine whether they clicked in the wrong place.
                    if tag_name != "path" && tag_name != "ellipse" && tag_name != "rect" {
                        state.clone().incorrect_choice(clone!(state => move || {
                            if let Some(audio) = &state.question.incorrect_audio {
                                state.selection_audio.set(Some(audio.clone()));
                            }
                        }));
                    }
                }
            }
        }))
        .child(html!("overlay-container", {
            .children_signal_vec(
                resize_info_signal()
                    .switch_signal_vec(clone!(state => move |_resize_info| {
                        signal_vec::always(state.traces.clone())
                            .map_signal(|trace| {
                                trace.phase.signal_cloned()
                            })
                            .map(|phase| {
                                match phase {
                                    PlayPhase::Playing(bubble) => {
                                        Some(TraceBubble::render(bubble))
                                    },
                                    _ => None
                                }
                            })
                            .filter(|x| x.is_some())
                            .map(|x| x.unwrap_ji())
                    }))
            )
        }))

    })
}
