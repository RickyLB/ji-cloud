use super::state::*;
use crate::base::styles::{FULL_STAGE, SVG_FILL_TRANSPARENT_CLICK_CLASS};
use crate::config::HINT_TIME_MS;
use dominator::{clone, svg, Dom};
use futures_signals::{
    map_ref,
    signal::{self, SignalExt},
};
use gloo_timers::future::TimeoutFuture;
use std::rc::Rc;
use utils::{
    prelude::*,
    resize::{resize_info_signal, ResizeInfo},
};

use components::traces::svg::{ShapeStyle, ShapeStyleKind, ShapeStyleMode, ShapeStylePlayMode};

impl AskQuestions {
    pub fn render(self: Rc<Self>) -> Dom {
        let state = self;
        svg!("svg", {
            .future(state.phase.signal().for_each(clone!(state => move |phase| {
                clone!(state => async move {
                    if phase == Phase::Hint {
                        TimeoutFuture::new(HINT_TIME_MS).await;
                        state.phase.set_neq(Phase::Play);
                    }
                })
            })))
            .class(&*FULL_STAGE)

            .child(svg!("rect", {
                .attr_signal("width", resize_info_signal().map(|info| {
                    format!("{}px", info.width)
                }))
                .attr_signal("height", resize_info_signal().map(|info| {
                    format!("{}px", info.height)
                }))
                .class(&*SVG_FILL_TRANSPARENT_CLICK_CLASS)
                .event(clone!(state => move |_evt:events::Click| {
                    state.clone().on_bg_click();
                }))
            }))
            .child_signal({
                map_ref!{
                    let resize_info = resize_info_signal(),
                    let phase = state.phase.signal(),
                    let item = state.item.signal_cloned()
                    => move {
                        Some(match phase {
                            Phase::Play => {
                                item.clone().render_svg_playing(state.clone(), resize_info)
                            },

                            Phase::Hint | Phase::WaitingNext => {
                                item.clone().render_svg_hint(state.clone(), resize_info)
                            }
                        })
                    }
                }
            })
        })
    }
}

impl QuestionItem {
    pub fn render_svg_hint(
        self: Rc<Self>,
        _parent: Rc<AskQuestions>,
        resize_info: &ResizeInfo,
    ) -> Dom {
        let state = self;
        state.hotspot.render(
            resize_info,
            || {},
            signal::always(ShapeStyle {
                interactive: false,
                mode: ShapeStyleMode::Play(ShapeStylePlayMode::Hint),
                kind: ShapeStyleKind::General,
            }),
        )
    }

    pub fn render_svg_playing(
        self: Rc<Self>,
        parent: Rc<AskQuestions>,
        resize_info: &ResizeInfo,
    ) -> Dom {
        let state = self;
        state.hotspot.render(
            resize_info,
            clone!(state, parent => move || {
                state.clone().on_correct_click(parent.clone());
            }),
            state.revealed.signal().map(|revealed| ShapeStyle {
                interactive: true,
                mode: if revealed {
                    ShapeStyleMode::Play(ShapeStylePlayMode::Selected)
                } else {
                    ShapeStyleMode::Transparent
                },
                kind: ShapeStyleKind::General,
            }),
        )
    }
}
