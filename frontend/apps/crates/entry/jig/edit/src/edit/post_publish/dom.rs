use dominator::{clone, html, Dom};
use futures_signals::signal::SignalExt;
use shared::domain::jig::JigId;
use utils::{
    events,
    jig::JigPlayerOptions,
    routes::{AssetRoute, Route},
};

use super::{super::state::State as JigEditState, actions, state::*};
use std::rc::Rc;

pub fn render(jig_id: JigId, jig_edit_state: Rc<JigEditState>) -> Dom {
    let state = Rc::new(State::new(jig_id, jig_edit_state));

    html!("post-publish", {
        .property("slot", "main")
        .property("jigFocus", state.jig_edit_state.jig_focus.as_str())
        .apply(clone!(state => move |dom| {
            match state.jig_edit_state.jig_focus {
                shared::domain::jig::JigFocus::Modules => {
                    dom.children(
                        render_modules_focused_actions(&state)
                    )
                },
                shared::domain::jig::JigFocus::Resources => {
                    dom.children(
                        render_resources_focused_actions(&state)
                    )
                },
            }
        }))
    })
}

fn render_modules_focused_actions(state: &Rc<State>) -> Vec<Dom> {
    let share_anchor = html!("post-publish-action", {
        .property("kind", "share")
        .property_signal("active", state.share_state.active_popup.signal_cloned().map(|active| active.is_some()))
    });

    vec![
        Rc::clone(&state.share_state).render(share_anchor, Some("actions")),
        html!("post-publish-action", {
            .property("slot", "actions")
            .property("kind", "new-jig")
            .event(clone!(state => move |_: events::Click| {
                actions::create_jig(Rc::clone(&state));
            }))
        }),
        html!("post-publish-action", {
            .property("kind", "play-jig")
            .property("slot", "actions")
            .event(clone!(state => move |_: events::Click| {
                state.jig_edit_state.play_jig.set(Some(JigPlayerOptions::default()));
            }))
        }),
    ]
}

fn render_resources_focused_actions(state: &Rc<State>) -> Vec<Dom> {
    vec![
        html!("post-publish-action", {
            .property("slot", "actions")
            .property("kind", "new-resource")
            .event(clone!(state => move |_: events::Click| {
                actions::create_jig(Rc::clone(&state));
            }))
        }),
        html!("post-publish-action", {
            .property("kind", "view-resources")
            .property("slot", "actions")
            .event(|_: events::Click| {
                Route::Asset(AssetRoute::ResourceGallery).redirect();
            })
        }),
    ]
}
