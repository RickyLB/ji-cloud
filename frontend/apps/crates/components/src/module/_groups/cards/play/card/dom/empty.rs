use dominator::{html, Dom, DomBuilder};
use utils::prelude::*;
use web_sys::HtmlElement;

use super::common::*;

pub enum EmptyKind {
    Question,
    Translucent,
}

impl EmptyKind {
    pub const fn as_str_id(&self) -> &'static str {
        match self {
            Self::Question => "question",
            Self::Translucent => "translucent",
        }
    }
}

pub struct EmptyCardOptions<'a> {
    pub kind: EmptyKind,
    pub theme_id: ThemeId,
    pub size: Size,
    pub active: bool,
    pub slot: Option<&'a str>,
}

impl<'a> EmptyCardOptions<'a> {
    pub fn new(kind: EmptyKind, theme_id: ThemeId, size: Size) -> Self {
        Self {
            kind,
            theme_id,
            size,
            //emulate default
            active: false,
            slot: None,
        }
    }
}

pub fn render_empty_card(options: EmptyCardOptions) -> Dom {
    _render_empty_card(
        options,
        None::<fn(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement>>,
    )
}

pub fn render_empty_card_mixin<F>(options: EmptyCardOptions, mixin: F) -> Dom
where
    F: FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement>,
{
    _render_empty_card(options, Some(mixin))
}

fn _render_empty_card<F>(options: EmptyCardOptions, mixin: Option<F>) -> Dom
where
    F: FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement>,
{
    let EmptyCardOptions {
        kind,
        theme_id,
        size,
        active,
        slot,
    } = options;

    html!("empty-card", {
        .apply_if(slot.is_some(), |dom|
            dom.prop("slot", slot.unwrap_ji())
        )
        .prop("active", active)
        .prop("size", size.as_str_id())
        .prop("theme", theme_id.as_str_id())
        .prop("kind", kind.as_str_id())
        .apply_if(mixin.is_some(), |dom| {
            (mixin.unwrap_ji()) (dom)
        })
    })
}
