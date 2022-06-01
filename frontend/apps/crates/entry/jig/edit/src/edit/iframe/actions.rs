use shared::{
    api::endpoints::{module::*, ApiEndpoint},
    domain::{module::*, asset::AssetType},
    error::EmptyError,
};
use std::cell::RefCell;
use std::rc::Rc;
use utils::prelude::*;

pub async fn load_module_kind(
    module_id: ModuleId,
    module_kind: Rc<RefCell<Option<ModuleKind>>>,
) {
    //TODO - API to just get module kind, so no need to load entire body here
    let path = GetDraft::PATH
        .replace("{asset_type}",AssetType::Jig.as_str())
        .replace("{module_id}", &module_id.0.to_string());

    match api_with_auth::<ModuleResponse, EmptyError, ()>(&path, GetDraft::METHOD, None).await {
        Ok(resp) => {
            *module_kind.borrow_mut() = Some(resp.module.body.kind());
        }
        Err(_) => {}
    }
}
