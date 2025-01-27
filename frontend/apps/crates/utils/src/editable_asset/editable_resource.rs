use std::collections::HashSet;
use std::iter::FromIterator;
use std::rc::Rc;

use chrono::{DateTime, Utc};
use futures_signals::signal::Mutable;
use futures_signals::signal_vec::MutableVec;
use shared::domain::additional_resource::AdditionalResource;
use shared::domain::asset::PrivacyLevel;
use shared::domain::meta::AffiliationId;
use shared::domain::{
    category::CategoryId,
    meta::AgeRangeId,
    module::LiteModule,
    resource::{ResourceId, ResourceResponse, ResourceUpdateDraftDataRequest},
};

#[derive(Clone)]
pub struct EditableResource {
    pub id: ResourceId,
    // cover and modules only for read
    pub cover: Mutable<Option<LiteModule>>,
    pub published_at: Mutable<Option<DateTime<Utc>>>,
    pub display_name: Mutable<String>,
    pub description: Mutable<String>,
    pub age_ranges: Mutable<HashSet<AgeRangeId>>,
    pub language: Mutable<String>,
    pub categories: Mutable<HashSet<CategoryId>>,
    pub affiliations: Mutable<HashSet<AffiliationId>>,
    pub additional_resources: Rc<MutableVec<AdditionalResource>>,
    pub privacy_level: Mutable<PrivacyLevel>,
}

impl From<ResourceResponse> for EditableResource {
    fn from(resource: ResourceResponse) -> Self {
        Self {
            id: resource.id,
            cover: Mutable::new(resource.resource_data.cover),
            display_name: Mutable::new(resource.resource_data.display_name),
            description: Mutable::new(resource.resource_data.description.clone()),
            age_ranges: Mutable::new(HashSet::from_iter(resource.resource_data.age_ranges)),
            language: Mutable::new(resource.resource_data.language),
            categories: Mutable::new(HashSet::from_iter(resource.resource_data.categories)),
            affiliations: Mutable::new(HashSet::from_iter(resource.resource_data.affiliations)),
            additional_resources: Rc::new(MutableVec::new_with_values(
                resource.resource_data.additional_resources,
            )),
            privacy_level: Mutable::new(resource.resource_data.privacy_level),
            published_at: Mutable::new(resource.published_at),
        }
    }
}

impl From<ResourceId> for EditableResource {
    fn from(resource_id: ResourceId) -> Self {
        Self {
            id: resource_id,
            cover: Default::default(),
            display_name: Default::default(),
            description: Default::default(),
            age_ranges: Default::default(),
            language: Default::default(),
            categories: Default::default(),
            affiliations: Default::default(),
            additional_resources: Default::default(),
            privacy_level: Default::default(),
            published_at: Default::default(),
        }
    }
}

impl EditableResource {
    pub fn fill_from_resource(&self, resource: ResourceResponse) {
        self.cover.set(resource.resource_data.cover);
        self.display_name.set(resource.resource_data.display_name);
        self.description
            .set(resource.resource_data.description.clone());
        self.age_ranges
            .set(HashSet::from_iter(resource.resource_data.age_ranges));
        self.language.set(resource.resource_data.language);
        self.categories
            .set(HashSet::from_iter(resource.resource_data.categories));
        self.affiliations
            .set(HashSet::from_iter(resource.resource_data.affiliations));
        self.additional_resources
            .lock_mut()
            .replace_cloned(resource.resource_data.additional_resources);
        self.privacy_level.set(resource.resource_data.privacy_level);
        self.published_at.set(resource.published_at);
    }

    pub fn deep_clone(&self) -> Self {
        Self {
            id: self.id,
            cover: Mutable::new(self.cover.get_cloned()),
            published_at: Mutable::new(self.published_at.get()),
            display_name: Mutable::new(self.display_name.get_cloned()),
            description: Mutable::new(self.description.get_cloned()),
            age_ranges: Mutable::new(self.age_ranges.get_cloned()),
            language: Mutable::new(self.language.get_cloned()),
            categories: Mutable::new(self.categories.get_cloned()),
            affiliations: Mutable::new(self.affiliations.get_cloned()),
            additional_resources: Rc::new(MutableVec::new_with_values(
                self.additional_resources.lock_ref().to_vec(),
            )),
            privacy_level: Mutable::new(self.privacy_level.get()),
        }
    }

    pub fn to_resource_update_request(&self) -> ResourceUpdateDraftDataRequest {
        // don't include additional_resources here since they're handled in separately
        ResourceUpdateDraftDataRequest {
            display_name: Some(self.display_name.get_cloned()),
            description: Some(self.description.get_cloned()),
            age_ranges: Some(self.age_ranges.get_cloned().into_iter().collect()),
            language: Some(self.language.get_cloned()),
            categories: Some(self.categories.get_cloned().into_iter().collect()),
            affiliations: Some(self.affiliations.get_cloned().into_iter().collect()),
            privacy_level: Some(self.privacy_level.get()),
            ..Default::default()
        }
    }
}
