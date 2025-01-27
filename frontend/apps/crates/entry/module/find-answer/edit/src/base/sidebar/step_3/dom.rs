use crate::base::{
    actions::Direction,
    state::{Phase, Question},
};

use super::state::*;
use std::rc::Rc;

use components::{
    audio::input::{AudioInput, AudioInputCallbacks, AudioInputOptions},
    hebrew_buttons::HebrewButtons,
    overlay::handle::OverlayHandle,
    stickers::state::Sticker,
    tabs::{MenuTab, MenuTabKind},
};
use dominator::{clone, html, with_node, Dom};
use futures_signals::{
    map_ref,
    signal::{Mutable, ReadOnlyMutable, Signal, SignalExt},
    signal_vec::SignalVecExt,
};
use js_sys::Reflect;
use shared::domain::module::body::{
    Audio,
    _groups::design::{Sticker as RawSticker, Text, TraceKind},
    find_answer::QuestionField,
};
use utils::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlElement, HtmlInputElement, HtmlTextAreaElement};

fn empty_signal(state: Rc<Step3>) -> impl Signal<Item = bool> {
    state
        .sidebar
        .base
        .questions
        .signal_vec_cloned()
        .len()
        .map(|len| len == 0)
}

fn questions_visible_signal(state: Rc<Step3>) -> impl Signal<Item = (bool, bool)> {
    map_ref! {
        let is_empty = empty_signal(state.clone()),
        let advanced_visible = state.advanced_visible.signal()
            => {
                (*is_empty, *advanced_visible)
            }
    }
}

fn current_question_idx_signal(
    state: Rc<Step3>,
    question_index: impl Signal<Item = Option<usize>>,
) -> impl Signal<Item = bool> {
    map_ref! {
        let current_question = state.sidebar.base.current_question.signal(),
        let question_index = question_index
            => {
            match (current_question, question_index) {
                (Some(current_index), Some(question_index)) => current_index == question_index,
                _ => false,
            }
        }
    }
}

fn question_default_title_signal(
    question: Rc<Question>,
    index: impl Signal<Item = Option<usize>>,
) -> impl Signal<Item = String> {
    map_ref! {
        let title = question.title.signal_cloned(),
        let index = index
            => {
                let title = title.clone().unwrap_or_default();
                if title.is_empty() {
                    format!("Question {}", index.unwrap_ji() + 1)
                } else {
                    title
                }
            }
    }
}

fn current_question_signal(state: Rc<Step3>) -> impl Signal<Item = Option<(usize, Rc<Question>)>> {
    state
        .sidebar
        .base
        .current_question
        .signal_ref(clone!(state => move |idx| {
            idx.map(clone!(state => move |idx| {
                let question = state.sidebar.base.questions.lock_ref().get(idx).unwrap_ji().clone();
                (idx, question)
            }))
        }))
}

pub fn render(state: Rc<Step3>) -> Dom {
    state.sidebar.base.continue_next_fn.set(None);

    html!("module-sidebar-body", {
        .after_removed(clone!(state => move |_| {
            if let QuestionField::Text(field_index) = state.sidebar.base.question_field.get_cloned() {
                let field = state.sidebar.base.stickers.get_as_text(field_index).unwrap_ji();
                field.is_editable.set_neq(true);
            }
        }))
        .future(state.sidebar.base.question_field.signal_cloned().for_each(clone!(state => move |field| {
            if let QuestionField::Text(field_index) = field {
                let field = state.sidebar.base.stickers.get_as_text(field_index).unwrap_ji();
                field.is_editable.set_neq(false);
            }
            async {}
        })))
        .prop("slot", "body")
        .prop_signal("dark", empty_signal(state.clone()).map(|is_empty| !is_empty))
        .child_signal(state.advanced_visible.signal().map(clone!(state => move |visible| {
            if visible {
                Some(render_advanced_modal(state.clone()))
            } else {
                None
            }
        })))
        .child_signal(empty_signal(state.clone()).map(clone!(state => move |is_empty| {
            if is_empty {
                // Make sure that the tab_kind is reset so that we can display the correct Jiggling text.
                state.sidebar.tab_kind.set(None);

                Some(html!("questions-empty", {
                    .style_signal("display", state.advanced_visible.signal().map(|visible| {
                        if visible { Some("none") } else { None }
                    }))
                    .child(html!("div", {
                        .child(html!("button-icon", {
                            .prop("size", "medium")
                            .prop("color", "blue")
                            .prop("icon", "circle-+-blue")
                            .event(clone!(state => move|_: events::Click| {
                                let raw_sticker = RawSticker::Text(Text::from_value(format!(
                                    r#"{{"version":"0.1.0","content":[{{"children":[{{"text":"{}","element":"H2"}}]}}]}}"#,
                                    "Your question box"
                                )));
                                let text_sticker = Sticker::new(state.sidebar.base.stickers.clone(), &raw_sticker);
                                if let Sticker::Text(text) = &text_sticker {
                                    text.can_delete.set(false);
                                }
                                state.sidebar.base.stickers.add_sticker(text_sticker);
                                let index = state.sidebar.base.stickers.list.lock_ref().len() - 1;
                                state.sidebar.base.question_field.set(QuestionField::Text(index));
                                state.sidebar.base.add_default_question();
                                state.sidebar.base.current_question.set(Some(0))
                            }))
                        }))
                    }))
                }))
            } else {
                Some(html!("question-container", {
                    .style_signal("display", state.advanced_visible.signal().map(|visible| {
                        if visible { Some("none") } else { None }
                    }))
                    .children_signal_vec(
                        state.sidebar.base.questions.signal_vec_cloned().enumerate().map(clone!(state => move |(index, question)| {
                            render_question(state.clone(), index, question.clone())
                        }))
                    )
                }))
            }
        })))
        .child_signal(questions_visible_signal(state.clone()).map(clone!(state => move |(is_empty, advanced_visible)| {
            if !is_empty && !advanced_visible {
                Some(html!("button-icon-label", {
                    .prop("slot", "action")
                    .prop("icon", "circle-+-blue")
                    .prop("label", "Add a question")
                    .prop("labelcolor", "blue")
                    .event(clone!(state => move|_: events::Click| {
                        state.sidebar.base.add_default_question();
                        let index = state.sidebar.base.questions.lock_ref().len() - 1;
                        state.sidebar.base.current_question.set(Some(index));
                    }))
                }))
            } else {
                None
            }
        })))
    })
}

pub fn render_advanced_modal(state: Rc<Step3>) -> Dom {
    let current_tab = Mutable::new(MenuTabKind::Correct);

    let current_question_signal = || {
        map_ref! {
            let current_tab = current_tab.clone().signal_cloned(),
            let current_question = current_question_signal(state.clone())
                => {
                    // Advanced modal is only available on a question, so there should _always_
                    // be a current question set at this point.
                    let (index, current_question) = current_question.clone().unwrap_ji();
                    (current_tab.clone(), index, current_question)
                }
        }
    };
    html!("module-sidebar-advanced-modal", {
        .prop("header", "Advanced feedback options")
        .prop("tabbed", true)
        .child(html!("fa-button", {
            .prop("slot", "close")
            .prop("icon", "fa-light fa-xmark")
            .event(clone!(state => move |_: events::Click| {
                state.advanced_visible.set_neq(false);
            }))
        }))
        .child(html!("empty-fragment", {
            .child(html!("p", {
                .style("margin", "0px 10px 10px 10px")
                .child(html!("strong", {
                    .text("Question: ")
                }))
                .text_signal(current_question_signal().map(|(_tab, index, question)| {
                    match question.title.get_cloned() {
                        Some(title) if title.len() > 0 => title,
                        _ => format!("Question {}", index + 1)
                    }
                }))
            }))
            .child(html!("menu-tabs", {
                .children(&mut [
                    render_advanced_tab(current_tab.clone(), MenuTabKind::Correct),
                    render_advanced_tab(current_tab.clone(), MenuTabKind::Incorrect),
                    render_advanced_tab(current_tab.clone(), MenuTabKind::Specific),
                    html!("module-sidebar-body", {
                        .prop("slot", "body")
                        .after_removed(clone!(state => move |_| {
                            // Advanced feedback is only available on the Answer tab of a question, so when we remove
                            // this modal, we need to reset the TraceKind to the Correct variant so that the teacher
                            // can continue adding correct answer traces.
                            state.sidebar.base.phase.set(
                                Phase::new_trace_unchecked(state.sidebar.base.clone(), TraceKind::Correct)
                            );
                        }))
                        .child(html!("empty-fragment", {
                            .style("display", "block")
                            .style("margin", "10px 0 20px 0")
                            .child_signal(current_question_signal().map(clone!(state => move |(tab, index, question)| {
                                let state = Rc::clone(&state);
                                match tab {
                                    MenuTabKind::Correct => {
                                        state.sidebar.base.phase.set(Phase::Layout);
                                        let audio_input = get_question_audio_input(state.clone(), question.correct_audio.clone(), index, question.clone());
                                        Some(html!("empty-fragment", {
                                            .child(html!("div", {
                                                .style("margin-bottom", "20px")
                                                .text("Tell students the answer is correct. “That's right! This is red.”")
                                            }))
                                            .child(AudioInput::render(audio_input, None))
                                        }))
                                    },
                                    MenuTabKind::Incorrect => {
                                        state.sidebar.base.phase.set(Phase::Layout);
                                        let audio_input = get_question_audio_input(state.clone(), question.incorrect_audio.clone(), index, question.clone());
                                        Some(html!("empty-fragment", {
                                            .child(html!("div", {
                                                .style("margin-bottom", "20px")
                                                .text("Offer hints or encouragement. “Try again! Red is the color of a stop sign.”")
                                            }))
                                            .child(AudioInput::render(audio_input, None))
                                        }))
                                    },
                                    MenuTabKind::Specific => {
                                        state.sidebar.base.phase.set(
                                            Phase::new_trace_unchecked(state.sidebar.base.clone(), TraceKind::Wrong)
                                        );

                                        if let Phase::Trace(traces) = state.sidebar.base.phase.get_cloned() {
                                            Some(html!("empty-fragment", {

                                                .child_signal(traces.selected_index.signal_cloned().map(clone!(traces => move |index| {
                                                    let input = match index {
                                                        Some(index) => match traces.get(index) {
                                                            Some(trace) if trace.kind == TraceKind::Wrong => {
                                                                let opts = AudioInputOptions::new(Some(traces.audio_signal(index)));

                                                                let callbacks = AudioInputCallbacks::new(
                                                                    Some(clone!(traces, index => move |audio:Audio| {
                                                                        traces.set_audio(index, Some(audio));
                                                                    })),
                                                                    Some(clone!(traces, index => move || {
                                                                        traces.set_audio(index, None);
                                                                    })),
                                                                );

                                                                Some(html!("empty-fragment", {
                                                                    .child(html!("div", {
                                                                        .style("margin-bottom", "20px")
                                                                        .text("Explain why this answer is incorrect. “Oops! This is yellow. You're looking for red.”")
                                                                    }))
                                                                    .child(AudioInput::render(AudioInput::new(opts, callbacks), None))
                                                                }))
                                                            },
                                                            _ => None,
                                                        },
                                                        None => None,
                                                    };

                                                    input.or_else(|| {
                                                        Some(html!("sidebar-empty", {
                                                            .prop("label", "Trace an incorrect area to add specific feedback")
                                                            .prop("imagePath", "module/_common/edit/sidebar/illustration-trace-area.svg")
                                                        }))
                                                    })
                                                })))
                                            }))
                                        } else {
                                            // We're setting the phase directly above this match statement, so this should
                                            // never be reachable.
                                            unreachable!()
                                        }
                                    }
                                    _ => None
                                }
                            })))
                        }))
                    })
                ])
            }))
        }))
    })
}

fn render_advanced_tab(current_tab: Mutable<MenuTabKind>, tab_kind: MenuTabKind) -> Dom {
    MenuTab::render(
        MenuTab::new(
            tab_kind,
            false,
            true,
            clone!(current_tab => move || current_tab.signal_ref(clone!(tab_kind => move |curr| {
                *curr == tab_kind
            }))),
            clone!(current_tab, tab_kind => move || {
                current_tab.set(tab_kind);
            }),
        ),
        Some("tabs"),
    )
}

fn close_kebab_menu(elem: &HtmlElement) {
    let _ = Reflect::set(elem, &"visible".into(), &false.into());
}

pub fn render_question(
    state: Rc<Step3>,
    index: ReadOnlyMutable<Option<usize>>,
    question: Rc<Question>,
) -> Dom {
    html!("question-item", {
        .child(html!("fa-button", {
            .prop("slot", "toggle")
            .prop_signal("icon", current_question_idx_signal(state.clone(), index.signal()).map(|is_current| {
                if is_current {
                    "fa-thin fa-angle-down"
                } else {
                    "fa-thin fa-angle-right"
                }
            }))
            .event(clone!(state, index => move|_: events::Click| {
                let question_index = index.get_cloned().unwrap_ji();
                state.sidebar.base.set_current_question(question_index);
            }))
        }))
        .child_signal(question.is_editing_title.signal_cloned().dedupe().map(clone!(state, index, question => move |is_editing_title| {
            if is_editing_title {
                Some(html!("input-wrapper", {
                    .prop("slot", "title")
                    .prop("pad", false)
                    .child(html!("input" => HtmlInputElement, {
                        .prop_signal("value", question.title.signal_cloned().map(|title| {
                            match title {
                                Some(title) => title,
                                None => "".to_string(),
                            }
                        }))
                        .event(clone!(state, index, question => move |evt: events::Change| {
                            let target = evt.dyn_target::<HtmlInputElement>().unwrap_ji();
                            let value = target.value();
                            let value = value.trim();

                            question.title.set(Some(value.into()));
                            state.sidebar.base.save_question(index.get().unwrap_ji(), question.clone());
                        }))
                        .event(clone!(question => move |_evt: events::Blur| {
                            question.is_editing_title.set_neq(false);
                        }))
                        .after_inserted(|elem| {
                            wasm_bindgen_futures::spawn_local(clone!(elem => async move {
                                gloo_timers::future::TimeoutFuture::new(0).await;
                                let _ = elem.focus();
                            }));
                        })
                    }))
                }))
            } else {
                Some(html!("span", {
                    .style("cursor", "pointer")
                    .prop("slot", "title")
                    .text_signal(question_default_title_signal(question.clone(), index.signal_cloned()))
                    .event(clone!(state, index => move|_: events::Click| {
                        let question_index = index.get_cloned().unwrap_ji();
                        state.sidebar.base.set_current_question(question_index);
                    }))
                }))
            }
        })))
        .child_signal(question.is_editing_title.signal_cloned().dedupe().map(clone!(question => move |is_editing| {
            if !is_editing {
                Some(html!("img-ui", {
                    .prop("slot", "edit-btn")
                    .style("cursor", "pointer")
                    .prop("path", "core/inputs/pencil-blue-darker.svg")
                    .event(clone!(question => move|_: events::Click| {
                        question.is_editing_title.set_neq(!question.is_editing_title.get());
                    }))
                }))
            } else {
                None
            }
        })))
        .child(html!("menu-kebab", {
            .with_node!(kebab_elem => {
                .prop("slot", "menu")
                .child(html!("menu-line", {
                    .prop("icon", "edit")
                    .prop("customLabel", "Rename question")
                    .event(clone!(question, kebab_elem => move |_: events::Click| {
                        question.is_editing_title.set_neq(true);
                        close_kebab_menu(&kebab_elem);
                    }))
                }))
                .child_signal(index.signal_ref(clone!(state, index, kebab_elem => move |current_index| {
                    match current_index {
                        Some(current_index) if *current_index > 0 => {
                            Some(html!("menu-line", {
                                .prop("icon", "move-up")
                                .event(clone!(state, index, kebab_elem => move |_: events::Click| {
                                    state.sidebar.base.move_question(index.get().unwrap_ji(), Direction::Up);
                                    close_kebab_menu(&kebab_elem);
                                }))
                            }))
                        }
                        _ => None
                    }
                })))
                .child_signal(index.signal_ref(clone!(state, index, kebab_elem => move |current_index| {
                    match current_index {
                        Some(current_index) if *current_index < state.sidebar.base.questions.lock_ref().len() - 1 => {
                            Some(html!("menu-line", {
                                .prop("icon", "move-down")
                                .event(clone!(state, index, kebab_elem => move |_: events::Click| {
                                    state.sidebar.base.move_question(index.get().unwrap_ji(), Direction::Down);
                                    close_kebab_menu(&kebab_elem);
                                }))
                            }))
                        }
                        _ => None
                    }
                })))
                .child(html!("menu-line", {
                    .prop("icon", "delete")
                    .prop("customLabel", "Delete question")
                    .event(clone!(question, kebab_elem => move |_: events::Click| {
                        question.confirm_delete.set_neq(true);
                        close_kebab_menu(&kebab_elem);
                    }))
                }))
            })
        }))
        .child_signal(current_question_idx_signal(state.clone(), index.signal()).map(clone!(state => move |is_current| {
            if is_current {
                Some(html!("empty-fragment", {
                    .after_removed(clone!(state => move |_| {
                        // Make sure that we're always in the initial phase when rendering a question body
                        state.sidebar.base.phase.set(Phase::Layout);
                        // Reset the tabs when a question is minimized
                        state.sidebar.tab_kind.set(None);
                    }))
                    .style("display", "contents")
                    .child_signal(
                        state.sidebar.tab_kind.signal_cloned().map(clone!(state => move |kind| {
                            match kind {
                                Some(_) => {
                                    Some(html!("menu-tabs", {
                                        .future(state.sidebar.tab_kind.signal_cloned().for_each(clone!(state => move |kind| {
                                            state.sidebar.base.phase.set(
                                                match kind {
                                                    Some(MenuTabKind::Answer) => Phase::new_trace_unchecked(state.sidebar.base.clone(), TraceKind::Correct),
                                                    _ => Phase::Layout,
                                                }
                                            );

                                            async {}
                                        })))
                                        .style("margin-top", "12px")
                                        .children(&mut [
                                            render_tab(MenuTabKind::Question, state.sidebar.tab_kind.clone()),
                                            render_tab(MenuTabKind::Answer, state.sidebar.tab_kind.clone()),
                                            html!("module-sidebar-body", {
                                                .prop("slot", "body")
                                                .style("overflow", "inherit")
                                                .child_signal(
                                                    //based on the selected tab kind, create and render the tab state
                                                    state.sidebar.tab_kind.signal_cloned()
                                                        .map(clone!(state => move |tab| {
                                                            tab.map(|tab| {
                                                                render_tab_body(state.clone(), tab.into())
                                                            })
                                                        }))
                                                )
                                                .child_signal(
                                                    state.sidebar.tab_kind.signal_cloned()
                                                        .map(clone!(state => move |tab| {
                                                            match tab {
                                                                Some(MenuTabKind::Answer) => {
                                                                    Some(html!("button-rect", {
                                                                        .prop("slot", "action")
                                                                        .prop("color", "blue")
                                                                        .prop("kind", "text")
                                                                        .text("Advanced feedback options")
                                                                        .event(clone!(state => move|_: events::Click| {
                                                                            state.advanced_visible.set_neq(true);
                                                                        }))
                                                                    }))
                                                                },
                                                                _ => None,
                                                            }
                                                        }))

                                                )
                                            })
                                        ])
                                    }))
                                }
                                None => {
                                    // If the current tab isn't set, then set
                                    // it.
                                    state.sidebar.tab_kind.set_neq(Some(MenuTabKind::Question));
                                    None
                                }
                            }
                        }))
                    )
                }))
            } else {
                None
            }
        })))
        .child_signal(question.confirm_delete.signal_cloned().map(clone!(state, index, question => move |confirm_delete| {
            if confirm_delete {
                Some(html!("empty-fragment", {
                    .apply(OverlayHandle::lifecycle(clone!(state, index, question => move || {
                        html!("modal-confirm", {
                            .prop("dangerous", true)
                            .prop("title", "Warning")
                            .prop("content", "Are you sure you want to delete this question?")
                            .prop("cancel_text", "Don't delete")
                            .prop("confirm_text", "Yes, delete")
                            .prop("confirmIcon", "core/menus/delete-white.svg")
                            .event(clone!(question => move |_evt: events::CustomCancel| question.confirm_delete.set_neq(false)))
                            .event(clone!(state, index, question => move |_evt: events::CustomConfirm| {
                                question.confirm_delete.set_neq(false);
                                state.sidebar.base.delete_question(index.get().unwrap_ji());
                            }))
                        })
                    })))
                }))
            } else {
                None
            }
        })))
    })
}

fn render_tab(tab_kind: MenuTabKind, selected_tab: Mutable<Option<MenuTabKind>>) -> Dom {
    MenuTab::render(
        MenuTab::new(
            tab_kind,
            false,
            true,
            clone!(selected_tab => move || selected_tab.signal_ref(clone!(tab_kind => move |curr| {
                match curr {
                    Some(curr) => *curr == tab_kind,
                    None => false
                }
            }))),
            clone!(tab_kind => move || {
                selected_tab.set_neq(Some(tab_kind));
            }),
        ),
        Some("tabs"),
    )
}

fn get_question_audio_input(
    state: Rc<Step3>,
    audio: Mutable<Option<Audio>>,
    question_index: usize,
    question: Rc<Question>,
) -> Rc<AudioInput> {
    let opts = AudioInputOptions::new(Some(audio.signal_cloned()));

    let callbacks = AudioInputCallbacks::new(
        Some(clone!(state, audio, question => move |add_audio: Audio| {
            *audio.lock_mut() = Some(add_audio);
            state.sidebar.base.save_question(question_index, question.clone());
        })),
        Some(clone!(state, audio, question => move || {
            *audio.lock_mut() = None;
            state.sidebar.base.save_question(question_index, question.clone());
        })),
    );

    AudioInput::new(opts, callbacks)
}

fn set_question_sticker_text(state: Rc<Step3>, value: String) {
    if let QuestionField::Text(field_index) = state.sidebar.base.question_field.get_cloned() {
        let text = state
            .sidebar
            .base
            .stickers
            .get_as_text(field_index)
            .unwrap_ji();

        if state
            .sidebar
            .base
            .question_sticker_text
            .get_cloned()
            .is_none()
        {
            state
                .sidebar
                .base
                .question_sticker_text
                .set(text.get_text_value());
        }

        // Weird bug: If the value is an empty string, the sticker's value will be updated,
        // but future updates will not work correctly. Adding in some whitespace resolves this.
        let value = if value.is_empty() { " ".into() } else { value };

        Reflect::set(
            &text.renderer_ref.get_cloned().unwrap_ji(),
            &JsValue::from_str("textValue"),
            &JsValue::from_str(&value),
        )
        .unwrap_ji();

        Reflect::set(
            &text.measurer_ref.get_cloned().unwrap_ji(),
            &JsValue::from_str("textValue"),
            &JsValue::from_str(&value),
        )
        .unwrap_ji();
    }
}

fn render_tab_body(state: Rc<Step3>, tab: Tab) -> Dom {
    match tab {
        Tab::Question => {
            html!("div", {
                .style("padding-top", "18px")
                .child_signal(current_question_signal(state.clone()).map(clone!(state => move |question| {
                    question.map(clone!(state => move |(index, question)| {
                        let audio_input = get_question_audio_input(state.clone(), question.question_audio.clone(), index, question.clone());

                        spawn_local(clone!(state, question => async move {
                            // Since the Question tab is always displayed first, we can set the text here
                            set_question_sticker_text(state, question.question_text.get_cloned().unwrap_or_default());
                        }));

                        html!("empty-fragment", {
                            .child(html!("input-wrapper", {
                                .prop("label", crate::strings::step_3::STR_LABEL)
                                .child(HebrewButtons::reveal().render(Some("hebrew-inputs")))
                                .child(html!("textarea" => HtmlTextAreaElement, {
                                    .attr("dir", "auto")
                                    .prop_signal("value", question.question_text.signal_cloned().map(|text| text.unwrap_or_default()))
                                    .prop("placeholder", crate::strings::step_3::STR_PLACEHOLDER)
                                    .prop("rows", 1)
                                    .event(clone!(state => move |evt: events::Input| {
                                        let target = evt.dyn_target::<HtmlTextAreaElement>().unwrap_ji();
                                        set_question_sticker_text(state.clone(), target.value());
                                    }))
                                    .event(move |evt: events::Change| {
                                        let target = evt.dyn_target::<HtmlTextAreaElement>().unwrap_ji();
                                        let value = target.value();
                                        let value = value.trim();

                                        question.question_text.set(Some(value.into()));

                                        if question.title.get_cloned().is_none() {
                                            question.title.set(Some(value.into()))
                                        }


                                        state.sidebar.base.save_question(index, question.clone());
                                    })
                                }))
                            }))
                            .child(html!("empty-fragment", {
                                .style("display", "block")
                                .style("margin", "20px 0")
                                .child(AudioInput::render(audio_input, None))
                            }))
                        })
                    }))
                })))
            })
        }
        Tab::Answer => html!("empty-fragment", {
            .child(html!("sidebar-empty", {
                .prop("label", "Trace all correct answers")
                .prop("imagePath", "module/_common/edit/sidebar/illustration-trace-area.svg")
            }))
        }),
    }
}
