use futures_signals::{
    signal_vec::MutableVec,
    signal::{Mutable, Signal, SignalExt}
};
use std::{
    rc::Rc,
    cell::RefCell,
    collections::HashMap
};
use super::{
    module::state::{Module, State as ModuleState},
    dragging::state::State as DragState
};
use utils::{drag::Drag, math::PointI32};
use dominator_helpers::{futures::AsyncLoader, signals::OptionSignal};
use shared::domain::jig::{Jig, LiteModule, JigId, module::ModuleId, ModuleKind};
use web_sys::DomRect;
use wasm_bindgen::prelude::*;
use chrono::{DateTime, Utc};

pub struct State {
    pub jig: Jig,
    pub module_id: Mutable<Option<ModuleId>>,
    pub name: Mutable<String>,
    pub publish_at: Mutable<Option<DateTime<Utc>>>,
    pub modules: MutableVec<Rc<Module>>,
    pub collapsed: Mutable<bool>,
    pub settings_shown: Mutable<bool>,
    pub drag: Mutable<Option<Rc<DragState>>>,
    pub drag_target_index: Mutable<Option<usize>>,
    pub loader: AsyncLoader,
}

impl State {
    pub fn new(jig: Jig, module_id: Mutable<Option<ModuleId>>) -> Self {
        Self {
            module_id,
            name: Mutable::new(jig.display_name.clone()),
            publish_at: Mutable::new(jig.publish_at.clone()),
            modules: MutableVec::new_with_values(
                jig.modules
                    .iter()
                    .map(|module| Rc::new(module.clone().into()))
                    .collect()
            ),
            collapsed: Mutable::new(false),
            settings_shown: Mutable::new(false),
            drag: Mutable::new(None),
            drag_target_index: Mutable::new(None),
            loader: AsyncLoader::new(),
            jig,
        }

    }

    //There's probably a way of making this simpler
    //But in any case, the signature is what matters :P
    pub fn drag_target_pos_signal(&self) -> impl Signal<Item = Option<PointI32>> {
        self.drag.signal_cloned().map(|drag| {
            OptionSignal::new(
                drag.map(|drag| drag.inner.pos_signal())
            )
        })
        .flatten()
        .map(|x| x.and_then(|x| x))
    }

    /*
    pub fn bounding_boxes(&self) -> Vec<(usize, DomRect)> {
        self.drag_targets
            .borrow()
            .iter()
            .map(|(index, module)| {
                //This must exist since it's added before the module 
                //is added to drag_targets
                let elem = module.elem.borrow();
                let elem = elem.as_ref().unwrap_throw();
                let rect = elem.get_bounding_client_rect();
                (*index, rect)
            })
            .collect()
    }
    */
}

