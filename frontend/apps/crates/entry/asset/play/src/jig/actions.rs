use std::rc::Rc;

use super::{
    state::{can_load_liked_status, Instructions, JigPlayer},
    timer::Timer,
};
use awsm_web::audio::{AudioClipOptions, AudioHandle};
use components::{
    audio::mixer::{AudioSourceExt, AUDIO_MIXER},
    module::_common::prelude::ModuleId,
};
use dominator::clone;
use futures_signals::signal::SignalExt;
use shared::{
    api::endpoints::{self, jig},
    domain::{
        asset::DraftOrLive,
        jig::{AudioBackground, JigGetDraftPath, JigGetLivePath, JigLikedPath, JigPlayPath},
        meta::GetMetadataPath,
        module::body::Instructions as ModuleInstructions,
    },
};
use utils::{
    iframe::{IframeAction, JigToModulePlayerMessage, ModuleToJigPlayerMessage},
    prelude::{ApiEndpointExt, SETTINGS},
    routes::{HomeRoute, Route},
    unwrap::UnwrapJiExt,
};
use wasm_bindgen_futures::spawn_local;

const TRACK_MODULE_COUNT: usize = 3;

pub fn toggle_background_audio(state: Rc<JigPlayer>) {
    let bg_audio_handle = state.bg_audio_handle.borrow();

    match &*bg_audio_handle {
        Some(bg_audio_handle) => {
            if state.bg_audio_playing.get() {
                pause_background_audio(&state, bg_audio_handle);
            } else {
                play_background_audio(&state, bg_audio_handle);
            };
        }
        None => {}
    };
}

pub fn play_background_audio(state: &JigPlayer, audio_handle: &AudioHandle) {
    audio_handle.play();
    state.bg_audio_playing.set(true);
}

pub fn pause_background_audio(state: &JigPlayer, audio_handle: &AudioHandle) {
    audio_handle.pause();
    state.bg_audio_playing.set(false);
}

pub fn navigate_forward(state: Rc<JigPlayer>) {
    let active_module = state.active_module.get();

    let module_count = (&*state.jig.lock_ref())
        .as_ref()
        .map(|jig| jig.jig_data.modules.len());

    if let Some(module_count) = module_count {
        let is_done = active_module == module_count - 1;
        // Track that the JIG has been played if
        // - The JIG is not a draft;
        // - The play hasn't been tracked yet;
        // - Either TRACK_MODULE_COUNT count of modules have been played or the JIG is done.
        let should_track = !state.player_options.draft_or_live.is_draft()
            && !*state.play_tracked.borrow()
            && (*state.played_modules.borrow() + 1 == TRACK_MODULE_COUNT || is_done);

        if should_track {
            state.loader.load(clone!(state => async move {
                // We don't need to handle an Ok Result; We can ignore Err, nothing is dependent on the
                // success of this call. The failure should be noted in the server logs.
                let _ = jig::Play::api_no_auth_empty(
                    JigPlayPath(state.jig_id),
                    None,
                ).await;

                // Set the flag to indicate that the play has been tracked for this JIG.
                *state.play_tracked.borrow_mut() = true;
            }))
        }

        if !is_done {
            // Only increment the played count when navigating to the _next_ module.
            let mut played_modules = state.played_modules.borrow_mut();
            *played_modules += 1;

            navigate_to_index(Rc::clone(&state), active_module + 1);
        } else {
            state.done.set(true);
        }
    }
}

pub fn navigate_back(state: Rc<JigPlayer>) {
    let active_module = state.active_module.get();
    if active_module != 0 {
        navigate_to_index(state, active_module - 1);
    }
}

pub fn navigate_to_index(state: Rc<JigPlayer>, index: usize) {
    state.active_module.set(index);
    state.timer.set(None);
    state.done.set(false);
}

pub fn navigate_to_module(state: Rc<JigPlayer>, module_id: &ModuleId) {
    if let Some(jig) = &*state.jig.lock_ref() {
        let index = jig
            .jig_data
            .modules
            .iter()
            .position(|module| &module.id == module_id);

        if let Some(index) = index {
            navigate_to_index(Rc::clone(&state), index);
        }
    }
}

pub fn set_instructions(
    state: Rc<JigPlayer>,
    instructions: Option<ModuleInstructions>,
    persisted: bool,
) {
    // Only set the instructions field if the Instructions has content. Otherwise, leave it at None.
    let instructions = match instructions {
        Some(instructions) if instructions.has_content() => Some(instructions),
        _ => None,
    };

    state.instructions.set(
        instructions.map(|instructions| Instructions::from_instructions(instructions, persisted)),
    );
}

pub fn show_instructions(state: Rc<JigPlayer>, visible: bool) {
    if let Some(instructions) = state.instructions.get_cloned() {
        if !(!instructions.persisted && instructions.text.is_none()) {
            state.instructions_visible.set_neq(visible);
            set_paused(state.clone(), visible);
        }

        if visible {
            play_instructions_audio(state);
        } else {
            if !instructions.persisted {
                send_iframe_message(
                    Rc::clone(&state),
                    JigToModulePlayerMessage::InstructionsDone,
                );
            }
        }
    }
}

pub fn play_instructions_audio(state: Rc<JigPlayer>) {
    if let Some(instructions) = state.instructions.get_cloned() {
        if let Some(audio) = &instructions.audio {
            AUDIO_MIXER.with(clone!(state, instructions => move |mixer| mixer.play_oneshot_on_ended(audio.as_source(), clone!(state => move || {
                if !instructions.persisted {
                    send_iframe_message(Rc::clone(&state), JigToModulePlayerMessage::InstructionsDone);
                }
            }))));
        }
    }
}

pub fn load_data(state: Rc<JigPlayer>) {
    state.loader.load(clone!(state => async move {
        load_resource_types(Rc::clone(&state)).await;
        load_jig(Rc::clone(&state)).await;
    }));
}

async fn load_jig(state: Rc<JigPlayer>) {
    state.loader.load(clone!(state => async move {
        let (jig, jig_liked) = match state.player_options.draft_or_live {
            DraftOrLive::Live => {
                let jig = {
                    jig::GetLive::api_no_auth(JigGetLivePath(state.jig_id), None).await
                };

                // Fetch whether the current user has liked this JIG.
                let jig_liked = {
                    match &jig {
                        // Only fetch liked status if the jig request didn't error, the user is
                        // logged in and the user is not the author of the JIG.
                        Ok(jig) if can_load_liked_status(jig) => {
                            jig::Liked::api_with_auth(JigLikedPath(state.jig_id), None)
                                .await
                                .map_or(false, |r| r.is_liked)
                        },
                        _ => false
                    }
                };

                (jig, jig_liked)
            },
            DraftOrLive::Draft => {
                let jig = {
                    jig::GetDraft::api_no_auth(JigGetDraftPath(state.jig_id), None).await
                };

                (jig, false)
            },
        };

        match jig {
            Ok(jig) => {
                // state.active_module.set(Some(resp.jig.modules[0].clone()));
                state.jig.set(Some(jig));
                state.jig_liked.set(Some(jig_liked));
            },
            Err(_) => {
                todo!();
            },
        }
    }));
}

async fn load_resource_types(state: Rc<JigPlayer>) {
    match endpoints::meta::Get::api_with_auth(GetMetadataPath(), None).await {
        Err(_) => todo!(),
        Ok(meta) => {
            state.resource_types.set(meta.resource_types);
        }
    };
}

fn init_audio(state: &JigPlayer, background_audio: AudioBackground) {
    let handle = AUDIO_MIXER.with(|mixer| {
        mixer.add_source(
            background_audio.as_source(),
            AudioClipOptions {
                auto_play: true,
                is_loop: true,
                on_ended: None::<fn()>,
            },
        )
    });

    let mut bg_audio_handle = state.bg_audio_handle.borrow_mut();
    *bg_audio_handle = Some(handle);
    state.bg_audio_playing.set(true);
}

pub fn start_timer(state: Rc<JigPlayer>, time: u32) {
    let timer = Timer::new(time);

    spawn_local(timer.time.signal().for_each(clone!(state => move|time| {
        if time == 0 {
            send_iframe_message(Rc::clone(&state), JigToModulePlayerMessage::TimerDone);
        }
        async {}
    })));

    state.timer.set(Some(timer));
}

pub fn toggle_paused(state: Rc<JigPlayer>) {
    let paused = !state.paused.get();
    set_paused(state, paused);
}

pub fn set_paused(state: Rc<JigPlayer>, paused: bool) {
    state.paused.set(paused);

    // pause timer if exists
    match &*state.timer.lock_ref() {
        None => {}
        Some(timer) => {
            *timer.paused.borrow_mut() = paused;
        }
    }

    // let iframe know that paused
    let iframe_message = match paused {
        false => JigToModulePlayerMessage::Play,
        true => JigToModulePlayerMessage::Pause,
    };

    send_iframe_message(Rc::clone(&state), iframe_message);
}

pub fn send_iframe_message(state: Rc<JigPlayer>, data: JigToModulePlayerMessage) {
    let iframe_origin: String = Route::Home(HomeRoute::Home).into();
    let iframe_origin = unsafe {
        SETTINGS
            .get_unchecked()
            .remote_target
            .spa_iframe(&iframe_origin)
    };

    match &*state.iframe.borrow() {
        None => {
            // Do nothing - we cannot send a message to an iframe which does not exist yet.
        }
        Some(iframe) => {
            let m = IframeAction::new(data);
            let _ = iframe
                .content_window()
                .unwrap_ji()
                .post_message(&m.into(), &iframe_origin);
        }
    };
}

pub fn on_iframe_message(state: Rc<JigPlayer>, message: ModuleToJigPlayerMessage) {
    match message {
        ModuleToJigPlayerMessage::AddPoints(amount) => {
            let mut points = state.points.lock_mut();
            *points += amount;
        }
        ModuleToJigPlayerMessage::Start(time) => {
            start_player(state, time);
        }
        ModuleToJigPlayerMessage::Next => {
            navigate_forward(state);
        }
        ModuleToJigPlayerMessage::Stop => {
            state.timer.set(None);
        }
        ModuleToJigPlayerMessage::JumpToIndex(index) => {
            navigate_to_index(state, index);
        }
        ModuleToJigPlayerMessage::JumpToId(module_id) => {
            navigate_to_module(state, &module_id);
        }
        ModuleToJigPlayerMessage::Instructions(instructions, persisted) => {
            set_instructions(state, instructions, persisted);
        }
    };
}

fn start_player(state: Rc<JigPlayer>, time: Option<u32>) {
    // If bg audio is not yet set (i.e. first module to be ready) initialize the audio once the jig is started
    if state.bg_audio_handle.borrow().is_none() {
        if let Some(jig) = state.jig.get_cloned() {
            if let Some(audio_background) = jig.jig_data.audio_background {
                init_audio(&state, audio_background);
            }
        }
    }

    // If the background audio is set to play, then start the audio
    if state.bg_audio_playing.get() {
        if let Some(bg_audio_handle) = &*state.bg_audio_handle.borrow() {
            play_background_audio(&state, bg_audio_handle);
        }
    }

    if let Some(time) = time {
        start_timer(Rc::clone(&state), time);
    }

    state.started.set_neq(true);
}

pub fn reload_iframe(state: Rc<JigPlayer>) {
    match &*state.iframe.borrow() {
        None => {}
        Some(iframe) => {
            iframe.set_src(&iframe.src());
            state.timer.set(None);
        }
    };
}
