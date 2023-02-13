use dominator::clone;
use shared::{
    api::endpoints::pro_dev::unit::*,
    domain::{pro_dev::unit::*, pro_dev::unit::GetProDevUnitDraftPath},
};
use std::rc::Rc;
use utils::prelude::*;

use super::UnitIframe;

impl UnitIframe {
    pub fn load_unit_kind(self: &Rc<Self>) {
        let state = self;
        state.loader.load(clone!(state => async move {

            // let asset_type = AssetType::from(&state.asset_id);

            match GetDraft::api_with_auth(GetProDevUnitDraftPath(state.pro_dev_id, state.unit_id.clone()), None).await {
                Ok(resp) => {
                    state.module_kind.set(Some(resp.module.body.kind()));
                }
                Err(_) => {}
            }

        }));
    }
}
