use std::rc::Rc;

use components::share_jig::ShareJig;
use dominator_helpers::futures::AsyncLoader;
use shared::domain::jig::JigId;

use super::super::state::State as JigEditState;

pub struct PostPublish {
    pub jig_id: JigId,
    pub loader: AsyncLoader,
    pub share_state: Rc<ShareJig>,
    pub jig_edit_state: Rc<JigEditState>,
}

impl PostPublish {
    pub fn new(jig_id: JigId, jig_edit_state: Rc<JigEditState>) -> Rc<Self> {
        Rc::new(Self {
            jig_id,
            loader: AsyncLoader::new(),
            share_state: ShareJig::new(jig_id),
            jig_edit_state,
        })
    }
}
