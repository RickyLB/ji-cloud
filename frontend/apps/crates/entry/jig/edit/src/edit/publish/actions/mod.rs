use std::{collections::HashMap, pin::Pin, rc::Rc};

use dominator::clone;
use futures::{join, Future};
use shared::{
    api::endpoints::{category, meta, ApiEndpoint},
    domain::{
        asset::{AssetId, PrivacyLevel},
        category::{Category, CategoryId, CategoryResponse, CategoryTreeScope, GetCategoryRequest},
        jig::JigFocus,
        meta::MetadataResponse,
    },
    error::EmptyError,
};
use utils::{
    prelude::{api_with_auth, UnwrapJiExt},
    routes::{CourseEditRoute, JigEditRoute},
};

use crate::edit::publish::editable_assets::EditableAsset;

use super::super::state::AssetEditState;
use super::state::Publish;

mod course_actions;
mod jig_actions;

impl Publish {
    pub async fn load_new(asset_edit_state: Rc<AssetEditState>) -> Self {
        let asset: Pin<Box<dyn Future<Output = Result<EditableAsset, ()>>>> =
            match asset_edit_state.asset_id {
                AssetId::JigId(jig_id) => Box::pin(jig_actions::load_jig(jig_id)),
                AssetId::CourseId(course_id) => Box::pin(course_actions::load_course(course_id)),
            };
        let categories = load_categories();
        let meta = load_metadata();

        let (asset, categories, meta) = join!(asset, categories, meta);

        let asset = asset.unwrap_ji();

        let categories = categories.unwrap_ji();
        let mut category_label_lookup = HashMap::new();
        get_categories_labels(&categories, &mut category_label_lookup, "");

        let meta = meta.unwrap_ji();

        if asset.published_at().is_none() {
            set_default_values(&asset, &meta);
        }

        // ensure the correct jig focus is set
        #[allow(irrefutable_let_patterns)] // TODO: remove once enum has another option
        if let EditableAsset::Jig(jig) = &asset {
            assert_eq!(
                asset_edit_state.jig_focus, jig.jig_focus,
                "Jig focus doesn't match the route"
            );
        };

        Self::new(
            asset,
            categories,
            category_label_lookup,
            meta.age_ranges,
            meta.affiliations,
            meta.resource_types,
            asset_edit_state,
        )
    }

    pub fn navigate_to_cover(&self) {
        match &self.asset {
            EditableAsset::Jig(_) => {
                // navigate to cover if exists otherwise navigate to landing
                let cover_module_id = self.asset.cover().as_ref().map(|m| m.id);

                let route = match cover_module_id {
                    Some(cover_module_id) => JigEditRoute::Module(cover_module_id),
                    None => JigEditRoute::Landing,
                };

                self.asset_edit_state.set_route_jig(route);
            }
            EditableAsset::Course(_) => {
                self.asset_edit_state
                    .set_route_course(CourseEditRoute::Cover);
            }
        };
    }

    // used to show tooltip, can probably be combined with `is_ready_to_publish` somehow
    fn form_invalid(self: &Rc<Self>) -> bool {
        self.asset.display_name().lock_ref().is_empty()
        // || self.jig.description.lock_ref().is_empty()
        // || self.jig.language.lock_ref().is_empty()
        // || self.jig.age_ranges.lock_ref().is_empty()
        // || self.jig.categories.lock_ref().is_empty()
    }

    // used to disable button
    pub fn is_ready_to_publish(self: &Rc<Self>) -> bool {
        match &self.asset {
            EditableAsset::Jig(jig) => match jig.jig_focus {
                JigFocus::Modules => jig.modules.iter().all(|m| m.is_complete),
                JigFocus::Resources => jig.cover.is_some(),
            },
            EditableAsset::Course(course) => course.cover.is_some(),
        }
    }

    pub fn save_asset(self: Rc<Self>) {
        let state = Rc::clone(&self);
        if Rc::clone(&state).form_invalid() {
            state.submission_tried.set(true);
            state.show_missing_info_popup.set(true);
            return;
        };

        state.loader.load(clone!(state => async move {
            match &state.asset {
                EditableAsset::Jig(jig) => {
                    jig_actions::save_and_publish_jig(jig)
                        .await
                        .unwrap_ji();
                    state.asset_edit_state.set_route_jig(JigEditRoute::PostPublish);
                },
                EditableAsset::Course(course) => {
                    course_actions::save_and_publish_course(course)
                        .await
                        .unwrap_ji();
                    state.asset_edit_state.set_route_course(CourseEditRoute::PostPublish);
                }
            };

            state.submission_tried.set(false);
        }));
    }
}

fn get_categories_labels(
    categories: &Vec<Category>,
    lookup: &mut HashMap<CategoryId, String>,
    base_name: &str,
) {
    for category in categories {
        let name = format!("{}{}", base_name, category.name);
        lookup.insert(category.id, name.clone());

        let base_name = name + "/";
        get_categories_labels(&category.children, lookup, &base_name);
    }
}

fn set_default_values(asset: &EditableAsset, meta: &MetadataResponse) {
    let available_affiliations = meta
        .affiliations
        .iter()
        .map(|affiliation| affiliation.id)
        .collect();
    asset.affiliations().replace(available_affiliations);

    let available_ages = meta.age_ranges.iter().map(|age| age.id).collect();
    asset.age_ranges().replace(available_ages);

    asset.privacy_level().replace(PrivacyLevel::default());
}

async fn load_categories() -> Result<Vec<Category>, EmptyError> {
    let req = GetCategoryRequest {
        ids: Vec::new(),
        scope: Some(CategoryTreeScope::Descendants),
    };

    match api_with_auth::<CategoryResponse, EmptyError, GetCategoryRequest>(
        category::Get::PATH,
        category::Get::METHOD,
        Some(req),
    )
    .await
    {
        Ok(resp) => Ok(resp.categories),
        Err(e) => Err(e),
    }
}

pub async fn load_metadata() -> Result<MetadataResponse, EmptyError> {
    api_with_auth::<MetadataResponse, EmptyError, ()>(meta::Get::PATH, meta::Get::METHOD, None)
        .await
}
