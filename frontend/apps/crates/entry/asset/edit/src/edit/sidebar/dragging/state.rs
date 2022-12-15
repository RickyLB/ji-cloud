use crate::edit::sidebar::spot::state::SpotState;
use std::rc::Rc;
use utils::drag::Drag;

//Instead of getting the anchor from the mouse point
//These values keep it at a consistent, predesiged place
const ANCHOR_X: f64 = 20.0;
const ANCHOR_Y: f64 = 100.0;

pub struct State {
    pub module: Rc<SpotState>,
    pub inner: Drag,
}

impl State {
    pub fn new(module: Rc<SpotState>, x: i32, y: i32) -> Self {
        Self {
            module,
            inner: Drag::new(x, y, ANCHOR_X, ANCHOR_Y, false),
        }
    }
}
