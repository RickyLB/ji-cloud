use super::base::state::*;
use components::module::_common::play::prelude::*;
use shared::domain::{
    jig::JigId,
    module::{
        body::drag_drop::{Mode, ModuleData as RawData, Step},
        ModuleId,
    },
};
use std::rc::Rc;

pub type AppState = GenericState<RawData, Mode, Step, Base>;

pub fn create_state(jig_id: JigId, module_id: ModuleId) -> Rc<AppState> {
    crate::debug::init(jig_id, module_id);

    let mut opts = StateOpts::new(jig_id, module_id);
    opts.force_raw = crate::debug::settings().data.clone();
    opts.skip_load_jig = crate::debug::settings().skip_load_jig;
    opts.skip_play = crate::debug::settings().skip_play;

    AppState::new(opts, Base::new)
}
