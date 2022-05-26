use super::state::*;
use dominator::{clone, html, Dom};
use shared::domain::module::body::{BodyExt, ModeExt, StepExt};
use std::rc::Rc;
use utils::prelude::*;

pub fn render<RawData, Mode, Step>(state: Rc<Choose<RawData, Mode, Step>>) -> Vec<Dom>
where
    RawData: BodyExt<Mode, Step> + 'static,
    Mode: ModeExt + 'static,
    Step: StepExt + 'static,
{
    vec![html!("choose-mode", {
        .property("slot", "main")
        .property("module", RawData::kind().as_str())
        .children(
            RawData::choose_mode_list()
                .into_iter()
                .map(|mode| {
                    html!("choose-mode-option", {
                        .property("mode", mode.as_str_id())
                        .property("label", mode.label())
                        .property("module", RawData::kind().as_str())
                        .event(clone!(state => move |_evt:events::Click| {
                            (state.on_mode_change) (mode);
                        }))
                    })
                })
                .collect::<Vec<Dom>>()
        )
    })]
}
