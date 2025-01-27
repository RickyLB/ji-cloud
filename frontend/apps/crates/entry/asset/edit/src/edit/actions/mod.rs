use std::rc::Rc;

use super::state::AssetEditState;
use dominator::clone;
use shared::domain::asset::{Asset, AssetId};
use utils::{
    prelude::ModuleToAssetEditorMessage,
    routes::{AssetEditRoute, AssetRoute, CourseEditRoute, JigEditRoute, ResourceEditRoute, Route},
    storage,
    unwrap::UnwrapJiExt,
};
use wasm_bindgen_futures::spawn_local;

mod course_actions;
mod jig_actions;
mod resource_actions;

impl AssetEditState {
    pub fn load_data(self: &Rc<Self>) {
        let state = self;
        spawn_local(clone!(state => async move {
            let asset = state.load_asset().await.unwrap_ji();
            match &asset {
                Asset::Jig(jig) => {
                    state.get_jig_spots(jig);
                },
                Asset::Course(course) => {
                    state.get_course_spots(course).await;
                },
                Asset::Resource(_) => {
                    // do nothing, resource doesn't have the sidebar
                },
                Asset::ProDev(_) => todo!(),
            };
            state.asset.fill_from_asset(asset);
        }));
    }

    async fn load_asset(self: &Rc<Self>) -> anyhow::Result<Asset> {
        let asset: Asset = match self.asset_id {
            AssetId::JigId(jig_id) => jig_actions::load_jig(jig_id).await?.into(),
            AssetId::ResourceId(resource_id) => {
                resource_actions::load_resource(resource_id).await?.into()
            }
            AssetId::CourseId(course_id) => course_actions::load_course(course_id).await?.into(),
            AssetId::ProDevId(_) => todo!(),
        };
        Ok(asset)
    }

    pub fn set_permanently_closed(&self) {
        let _ = storage::get_local_storage()
            .unwrap_ji()
            .set_item("onboarding", "hidden");
        self.show_onboarding.set_neq(false);
    }

    pub fn on_iframe_message(&self, message: ModuleToAssetEditorMessage) {
        match message {
            ModuleToAssetEditorMessage::Publish => {
                self.navigate_to_publish();
            }
        }
    }

    pub fn navigate_to_publish(&self) {
        match self.asset_id {
            AssetId::JigId(jig_id) => {
                self.set_route_jig(JigEditRoute::Publish);
                Route::push_state(Route::Asset(AssetRoute::Edit(AssetEditRoute::Jig(
                    jig_id,
                    JigEditRoute::Publish,
                ))));
            }
            AssetId::CourseId(course_id) => {
                self.set_route_course(CourseEditRoute::Publish);
                Route::push_state(Route::Asset(AssetRoute::Edit(AssetEditRoute::Course(
                    course_id,
                    CourseEditRoute::Publish,
                ))));
            }
            AssetId::ResourceId(resource_id) => {
                self.set_route_resource(ResourceEditRoute::Landing);
                Route::push_state(Route::Asset(AssetRoute::Edit(AssetEditRoute::Resource(
                    resource_id,
                    ResourceEditRoute::Landing,
                ))));
            }
            AssetId::ProDevId(_) => todo!(),
        }
    }
}
