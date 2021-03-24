use super::raw;
use std::{
    rc::Rc,
    cell::RefCell
};
use futures_signals::{
    signal::{Mutable, SignalExt, Signal},
    signal_vec::{MutableVec, SignalVecExt, SignalVec},
};
use rand::prelude::*;
use shared::{
    domain::image::ImageId,
    media::MediaLibrary
};
use crate::{
    index::state::LocalState as IndexState,
    player::card::state::{State as CardState, Side},
};

pub struct State {
    pub index: Rc<IndexState>,
    pub mode: GameMode,
    pub pair_lookup: Vec<usize>,
    pub original_pairs: Vec<(raw::Card, raw::Card)>,
    pub cards: Vec<Rc<CardState>>,
    pub theme: String,
    pub flip_state: Mutable<FlipState>,
    pub found_pairs: RefCell<Vec<(usize, usize)>>, 
}

impl State {
    pub fn new(index: Rc<IndexState>, raw_data:raw::GameData) -> Self {
        let mode:GameMode = raw_data.mode.into();

        let n_cards = raw_data.pairs.len() * 2;
        let mut pair_lookup:Vec<usize> = vec![0;n_cards]; 
        let mut cards = { 
            let pairs = &raw_data.pairs;

            let n_cards = pairs.len() * 2;
            let mut cards:Vec<Rc<CardState>> = Vec::with_capacity(n_cards);
            let mut index:usize = 0;

            for (card_1, card_2) in pairs.iter() {
                let id_1 = index; 
                let id_2 = index + 1;
                index = id_2 + 1;

                cards.push(Rc::new(CardState::new(card_1, id_1, id_2, Side::Left)));
                cards.push(Rc::new(CardState::new(card_2, id_2, id_1, Side::Right)));
            }

            cards
        };

        for card in cards.iter() {
            pair_lookup[card.id] = card.other_id;
        }

        let mut rng = thread_rng();

        if crate::debug::settings().shuffle {
            cards.shuffle(&mut rng);
        }

        Self {
            index,
            mode,
            pair_lookup,
            original_pairs: raw_data.pairs,
            cards,
            theme: raw_data.theme,
            flip_state: Mutable::new(FlipState::None), 
            found_pairs: RefCell::new(Vec::new()),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum GameMode {
    Duplicate,
    WordsAndImages
}

impl From<raw::Mode> for GameMode {
    fn from(raw_mode:raw::Mode) -> Self {
        match raw_mode {
            raw::Mode::Duplicate => Self::Duplicate,
            raw::Mode::WordsAndImages => Self::WordsAndImages,
        }
    }
}

impl From<GameMode> for raw::Mode {
    fn from(game_mode:GameMode) -> raw::Mode {
        match game_mode {
            GameMode::Duplicate => raw::Mode::Duplicate,
            GameMode::WordsAndImages => raw::Mode::WordsAndImages,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FlipState {
    None,
    One(usize),
    Two((usize, usize)),
}
