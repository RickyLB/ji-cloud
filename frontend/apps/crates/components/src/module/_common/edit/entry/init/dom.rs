use super::super::{base::state::*, state::*};
use dominator::{html, Dom};
use shared::domain::module::body::{BodyExt, ModeExt, StepExt};
use std::rc::Rc;

pub fn render<Mode, Step, RawData, Base, Main, Sidebar, Header, Footer, Overlay>(
    _state: Rc<GenericState<Mode, Step, RawData, Base, Main, Sidebar, Header, Footer, Overlay>>,
) -> Dom
where
    Base: BaseExt<Step> + 'static,
    Main: MainExt + 'static,
    Sidebar: SidebarExt + 'static,
    Header: HeaderExt + 'static,
    Footer: FooterExt + 'static,
    Overlay: OverlayExt + 'static,
    Mode: ModeExt + 'static,
    Step: StepExt + 'static,
    RawData: BodyExt<Mode, Step> + 'static,
{
    html!("div", {
        .prop("slot", "main")
    })
}
