use super::card::state::*;
use crate::module::_groups::cards::{edit::state::*, lookup::Side};
use futures_signals::signal::{ReadOnlyMutable, Mutable};
use std::rc::Rc;

use shared::domain::module::body::_groups::cards::Step;
pub struct MainPair<RawData: RawDataExt, E: ExtraExt> {
    pub base: Rc<CardsBase<RawData, E>>,
    pub step: Step,
    pub index: ReadOnlyMutable<Option<usize>>,
    pub left: Rc<MainCard<RawData, E>>,
    pub right: Rc<MainCard<RawData, E>>,
    pub hebrew_buttons_open: Mutable<bool>,
}

impl<RawData: RawDataExt, E: ExtraExt> MainPair<RawData, E> {
    pub fn new(
        base: Rc<CardsBase<RawData, E>>,
        step: Step,
        index: ReadOnlyMutable<Option<usize>>,
        pair: (Card, Card),
    ) -> Rc<Self> {
        Rc::new(Self {
            base: base.clone(),
            step,
            index: index.clone(),
            left: MainCard::new(
                base.clone(),
                step,
                index.clone(),
                Side::Left,
                pair.0.clone(),
                pair.1.clone(),
            ),
            right: MainCard::new(base, step, index, Side::Right, pair.1.clone(), pair.0),
            hebrew_buttons_open: Mutable::new(false),
        })
    }
}
