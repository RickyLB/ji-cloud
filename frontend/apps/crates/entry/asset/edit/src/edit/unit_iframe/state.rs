use dominator_helpers::futures::AsyncLoader;
use futures_signals::signal::Mutable;
use shared::domain::{
    asset::AssetId,
    pro_dev::{unit::{ProDevUnitValue, ProDevUnitId}, ProDevId},
};
use std::rc::Rc;

pub struct UnitIframe {
    pub pro_dev_id: ProDevId,
    pub unit_id: ProDevUnitId,
    pub unit_value: Mutable<Option<ProDevUnitValue>>,
    pub loader: AsyncLoader,
}

impl UnitIframe {
    pub fn new(pro_dev_id: ProDevId, unit_id: ProDevUnitId) -> Rc<Self> {
        Rc::new(Self {
            pro_dev_id,
            unit_id,
            unit_value: Default::default(),
            loader: AsyncLoader::new(),
        })
    }
}
