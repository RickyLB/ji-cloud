use futures_signals::signal::Mutable;

use crate::base::state::Base;
use std::{cell::RefCell, rc::Rc, sync::atomic::AtomicBool};
use web_sys::Element;

use shared::domain::module::body::legacy::design::{
    HideToggle, Sticker as RawSticker, StickerKind as RawStickerKind,
};

pub struct ImagePlayer {
    pub base: Rc<Base>,
    pub raw: RawSticker,
    pub size: Mutable<Option<(f64, f64)>>,
    pub controller: Controller,
}

impl ImagePlayer {
    pub fn new(base: Rc<Base>, raw: RawSticker) -> Rc<Self> {
        let size = Mutable::new(raw.override_size);
        let controller = Controller::new(base.clone(), &raw);

        Rc::new(Self {
            base,
            raw,
            controller,
            size,
        })
    }

    pub fn get_text(&self) -> Option<&str> {
        match &self.raw.kind {
            RawStickerKind::Text(s) => s.as_ref().map(|s| s.as_ref()),
            _ => None,
        }
    }
}

pub struct Controller {
    pub base: Rc<Base>,
    pub elem: RefCell<Option<Element>>,
    // directly set from raw.hide
    pub hidden: Mutable<bool>,
    // starts false (changed via ux)
    pub has_toggled_once: AtomicBool,
    // set from raw.hide_toggle
    pub hide_toggle: Option<HideToggle>,
    pub audio_filename: Option<String>,
    pub interactive: bool,
}

impl Controller {
    pub fn new(base: Rc<Base>, raw: &RawSticker) -> Self {
        let interactive = raw.hide_toggle.is_some() || raw.audio_filename.is_some();

        Self {
            base,
            elem: RefCell::new(None),
            hidden: Mutable::new(raw.hide),
            has_toggled_once: AtomicBool::new(false),
            hide_toggle: raw.hide_toggle,
            audio_filename: raw.audio_filename.clone(),
            interactive,
        }
    }
}
