use crate::base::state::*;
use std::rc::Rc;

use futures_signals::signal::Mutable;
use shared::domain::jig::module::body::drag_drop::*;

pub struct Game {
    pub base: Rc<Base>,
    pub phase: Mutable<Phase>,
}

impl Game {
    pub fn new(base: Rc<Base>) -> Rc<Self> {
        let phase = Mutable::new(match base.settings.hint {
            Hint::Highlight => Phase::ShowHints,
            Hint::None => Phase::Playing,
        });

        let _self = Rc::new(Self { base, phase });

        _self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    ShowHints,
    Playing,
}
