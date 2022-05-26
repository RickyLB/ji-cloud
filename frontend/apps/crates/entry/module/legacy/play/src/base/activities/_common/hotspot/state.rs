use dominator::animation::MutableAnimation;
use futures_signals::signal::Mutable;
use shared::domain::module::body::legacy::activity::Hotspot as RawHotspot;
use std::rc::Rc;

pub struct Hotspot {
    pub raw: RawHotspot,
    pub tooltip_text: Mutable<Option<String>>,
    pub fade_animation: MutableAnimation,
}

impl Hotspot {
    pub fn new(raw: RawHotspot) -> Rc<Self> {
        Rc::new(Self {
            raw,
            tooltip_text: Mutable::new(None),
            fade_animation: MutableAnimation::new(1000.0),
        })
    }
}
