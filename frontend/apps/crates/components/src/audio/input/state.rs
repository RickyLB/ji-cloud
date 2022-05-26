use super::callbacks::AudioInputCallbacks;
use super::options::*;
use super::recorder::AudioRecorder;
use dominator::clone;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use shared::domain::module::body::Audio;
use std::cell::RefCell;
use std::rc::Rc;
use utils::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum AudioInputMode {
    Playing(Audio),
    Stopped(Audio),
    Empty,
    Recording,
    Uploading,
    // Paused(AudioId, Timecode) we don't have a design for this but might be useful
}

#[derive(Clone, Copy, PartialEq)]
pub enum AudioInputAddMethod {
    Record,
    Upload,
}

pub struct AudioInput {
    pub callbacks: AudioInputCallbacks,
    pub mode: Mutable<AudioInputMode>,
    pub add_method: Mutable<AudioInputAddMethod>,
    pub recorder: AudioRecorder,
    pub aborter: RefCell<AbortController>,
    ext_audio_handle: RefCell<Option<FutureHandle>>,
}

impl AudioInput {
    pub fn new<S>(opts: AudioInputOptions<S>, callbacks: AudioInputCallbacks) -> Rc<Self>
    where
        S: Signal<Item = Option<Audio>> + 'static,
    {
        let _self = Rc::new(Self {
            callbacks,
            mode: Mutable::new(AudioInputMode::Empty),
            recorder: AudioRecorder::new(),
            add_method: Mutable::new(AudioInputAddMethod::Record),
            aborter: RefCell::new(AbortController::new()),
            ext_audio_handle: RefCell::new(None),
        });

        *_self.ext_audio_handle.borrow_mut() = opts.ext_audio_signal.map(|sig| {
            spawn_handle(sig.for_each(clone!(_self => move |ext_audio| {
                _self.set_audio_ext(ext_audio);
                async {}
            })))
        });

        _self
    }
}
