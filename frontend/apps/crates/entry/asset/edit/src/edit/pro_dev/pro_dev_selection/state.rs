use std::rc::Rc;

use components::asset_search_bar::AssetSearchBar;
use dominator_helpers::futures::AsyncLoader;
use futures_signals::{signal::Mutable, signal_vec::MutableVec};
use shared::domain::{asset::Asset, pro_dev::ProDevId};
use utils::drag::Drag;

use crate::edit::AssetEditState;

pub struct ProDevSelection {
    pub pro_dev_id: ProDevId,
    pub search_bar: Rc<AssetSearchBar>,
    pub asset_edit_state: Rc<AssetEditState>,
    pub loader: AsyncLoader,
    pub search_results: MutableVec<Rc<JigResponse>>,
    pub next_page: Mutable<u32>,
    pub active_query: Mutable<String>,
    pub total_unit_count: Mutable<u32>,
    pub drag: Mutable<Option<Rc<Drag<Asset>>>>,
}

impl ProDevSelection {
    pub fn new(pro_dev_id: ProDevId, asset_edit_state: &Rc<AssetEditState>) -> Rc<Self> {
        Rc::new(Self {
            pro_dev_id,
            search_bar: AssetSearchBar::new(),
            asset_edit_state: Rc::clone(asset_edit_state),
            loader: AsyncLoader::new(),
            search_results: Default::default(),
            next_page: Default::default(),
            active_query: Default::default(),
            total_jig_count: Default::default(),
            drag: Default::default(),
        })
    }
}
