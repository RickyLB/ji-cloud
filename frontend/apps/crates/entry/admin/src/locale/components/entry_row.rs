use crate::locale::actions::{AsStringExt, EnumOptionsExt};
use crate::locale::state::{Column, DisplayableEntry, Section, State};
use dominator::{clone, events, html, with_node, Dom};
use futures_signals::signal::Mutable;
use futures_signals::signal_vec::SignalVecExt;
use shared::domain::locale::EntryStatus;
use std::clone::Clone;
use std::rc::Rc;
use url::Url;
use utils::unwrap::UnwrapJiExt;
use uuid::Uuid;
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};

#[derive(Clone)]
pub struct EntryRow {}

impl EntryRow {
    fn url_option_string(url: &Option<Url>) -> String {
        if url.is_some() {
            url.clone().unwrap_ji().to_string()
        } else {
            String::new()
        }
    }

    fn save_entry(state: Rc<State>, entry: DisplayableEntry) {
        state.saving_loader.load(clone!(state => async move {
            state.save_entry(&entry).await;
        }))
    }

    pub fn render(entry: Rc<Mutable<DisplayableEntry>>, state: Rc<State>) -> Dom {
        html!("locale-row", {
            .prop("slot", "rows")
            .children_signal_vec(state.visible_columns.signal_vec_cloned()
                .map(clone!(state, entry => move |column| {
                    let entry_ref = entry.lock_ref();
                    match column {
                        Column::ID => {
                            html!("locale-cell", {
                                .child(html!("input" =>  HtmlInputElement, {
                                    .with_node!(elem => {
                                        .prop("value", entry_ref.id)
                                        .attr("readonly", "")
                                        .event(clone!(state, entry => move |_: events::Input| {
                                            let value: u32 = elem.value().parse().unwrap_ji();
                                            let mut entry = entry.lock_mut();
                                            entry.id = value;
                                            Self::save_entry(state.clone(), entry.clone());
                                        }))
                                    })
                                }))
                            })
                        },
                        Column::Section => {
                            html!("locale-cell", {
                                .child(html!("input" => HtmlInputElement, {
                                    .with_node!(elem => {
                                        .apply_if(entry_ref.section.is_some(), |dom| {
                                            dom.prop("value", &entry_ref.section.clone().unwrap_ji())
                                        })
                                        .attr("list", "sections")
                                        .event(clone!(state, entry => move |_: events::Input| {
                                            let value: Section = elem.value();
                                            let mut entry = entry.lock_mut();
                                            entry.section = Some(value);
                                            Self::save_entry(state.clone(), entry.clone());
                                        }))
                                        .event(clone!(state => move |_: events::Change| {
                                            state.regenerate_section_options();
                                        }))
                                    })
                                }))
                            })
                        },
                        Column::ItemKind => {
                            html!("locale-cell", {
                                .child(html!("select" => HtmlSelectElement, {
                                    .with_node!(elem => {
                                        .prop("value", {
                                            match entry_ref.item_kind_id {
                                                Some(item_kind_id) => item_kind_id.to_string(),
                                                None => String::new(),
                                            }
                                        })
                                        .event(clone!(state, entry => move |_: events::Input| {
                                            let value: String = elem.value();
                                            let value = match Uuid::parse_str(&value) {
                                                Ok(uuid) => Some(uuid),
                                                Err(_) => None,
                                            };
                                            let mut entry = entry.lock_mut();
                                            entry.item_kind_id = value;
                                            Self::save_entry(state.clone(), entry.clone());
                                        }))
                                        .child(html!("option", {
                                            .prop("value", String::new())
                                            .prop("selected", entry_ref.item_kind_id.is_none())
                                        }))
                                        .children(
                                            state.item_kind_options
                                                .iter()
                                                .map(|item_kind| {
                                                    html!("option", {
                                                        .prop("text", &item_kind.name)
                                                        .prop("value", &item_kind.id.to_string())
                                                        .prop("selected", entry_ref.item_kind_id.is_some() && entry_ref.item_kind_id.unwrap_ji() == item_kind.id)
                                                    })
                                                })
                                        )
                                    })
                                }))
                            })
                        },
                        Column::English => {
                            html!("locale-cell", {
                                .child(html!("textarea" => HtmlTextAreaElement, {
                                    .with_node!(elem => {
                                        .text(&entry_ref.english)
                                        .event(clone!(state, entry => move |_: events::Input| {
                                            let value: String = elem.value();
                                            let mut entry = entry.lock_mut();
                                            entry.english = value;
                                            Self::save_entry(state.clone(), entry.clone());
                                        }))
                                    })
                                }))
                            })
                        },
                        Column::Hebrew => {
                            html!("locale-cell", {
                                .child(html!("textarea" => HtmlTextAreaElement, {
                                    .with_node!(elem => {
                                        .text(&entry_ref.hebrew)
                                        .event(clone!(state, entry => move |_: events::Input| {
                                            let value: String = elem.value();
                                            let mut entry = entry.lock_mut();
                                            entry.hebrew = value;
                                            Self::save_entry(state.clone(), entry.clone());
                                        }))
                                    })
                                }))
                            })
                        },
                        Column::Status => {
                            html!("locale-cell", {
                                .child(html!("select" => HtmlSelectElement, {
                                    .with_node!(elem => {
                                        .event(clone!(state, entry => move |_event: events::Change| {
                                            let value: String = elem.value();
                                            let mut entry = entry.lock_mut();
                                            entry.status = EntryStatus::from_str(&value); //.unwrap_ji();
                                            Self::save_entry(state.clone(), entry.clone());
                                        }))
                                    })
                                    .children(
                                        EntryStatus::options()
                                            .iter()
                                            .map(|o| {
                                                html!("option", {
                                                    .prop("text", o.to_string())
                                                    .prop("value", o.to_string())
                                                    .prop("selected", o == &entry_ref.status)
                                                })
                                            })
                                    )
                                }))
                            })
                        },
                        Column::ZeplinReference => {
                            html!("locale-cell", {
                                .child(html!("locale-hover-link", {
                                    .prop_signal("link", entry_ref.zeplin_reference.signal_ref(Self::url_option_string))
                                    .child(html!("input" => HtmlInputElement, {
                                        .with_node!(elem => {
                                            .prop("type", "url")
                                            .apply_if(entry_ref.zeplin_reference.lock_ref().is_some(), |dom| {
                                                dom.prop("value", &entry_ref.zeplin_reference.lock_ref().clone().unwrap_ji().to_string())
                                            })
                                            .event(clone!(state, entry => move |_: events::Input| {
                                                let value: String = elem.value();
                                                let value = Url::parse(&value);

                                                let zeplin_reference = &entry.lock_ref().zeplin_reference;
                                                match value {
                                                    Ok(value) => zeplin_reference.set(Some(value)),
                                                    Err(_) => zeplin_reference.set(None),
                                                };
                                                Self::save_entry(state.clone(), entry.lock_ref().clone());
                                            }))
                                        })
                                    }))
                                }))
                            })
                        },
                        Column::Comments => {
                            html!("locale-cell", {
                                .child(html!("input" => HtmlInputElement, {
                                    .with_node!(elem => {
                                        .prop("value", &entry_ref.comments)
                                        .event(clone!(state, entry => move |_: events::Input| {
                                            let value: String = elem.value();
                                            let mut entry = entry.lock_mut();
                                            entry.comments = value;
                                            Self::save_entry(state.clone(), entry.clone());
                                        }))
                                    })
                                }))
                            })
                        },
                        Column::App => {
                            html!("locale-cell", {
                                .child(html!("input", {
                                    .attr("type", "checkbox")
                                    .prop("checked", entry_ref.in_app)
                                    .event(clone!(state, entry => move |event: events::Change| {
                                        let value: bool = event.checked().unwrap_ji();
                                        let mut entry = entry.lock_mut();
                                        entry.in_app = value;
                                        Self::save_entry(state.clone(), entry.clone());
                                    }))
                                }))
                            })
                        },
                        Column::Element => {
                            html!("locale-cell", {
                                .child(html!("input", {
                                    .attr("type", "checkbox")
                                    .prop("checked", entry_ref.in_element)
                                    .event(clone!(state, entry => move |event: events::Change| {
                                        let value: bool = event.checked().unwrap_ji();
                                        let mut entry = entry.lock_mut();
                                        entry.in_element = value;
                                        Self::save_entry(state.clone(), entry.clone());
                                    }))
                                }))
                            })
                        },
                        Column::Mock => {
                            html!("locale-cell", {
                                .child(html!("input", {
                                    .attr("type", "checkbox")
                                    .prop("checked", entry_ref.in_mock)
                                    .event(clone!(state, entry => move |event: events::Change| {
                                        let value: bool = event.checked().unwrap_ji();
                                        let mut entry = entry.lock_mut();
                                        entry.in_mock = value;
                                        Self::save_entry(state.clone(), entry.clone());
                                    }))
                                }))
                            })
                        },
                        Column::Actions => {
                            html!("locale-cell", {
                                .child(
                                    html!("locale-actions-wrapper", {
                                        .children(&mut [
                                            html!("button-rect", {
                                                .prop("slot", "first")
                                                .prop("kind", "text")
                                                .text("Clone")
                                                .event(clone!(state, entry => move |_event: events::Click| {
                                                    state.loader.load(clone!(state, entry => async move {
                                                        state.clone_entry(&entry.lock_ref()).await;
                                                    }))
                                                }))
                                            }),
                                            html!("button-rect", {
                                                .prop("slot", "second")
                                                .prop("kind", "text")
                                                .text("Delete")
                                                .event(clone!(state, entry => move |_event: events::Click| {
                                                    state.loader.load(clone!(state, entry => async move {
                                                        state.remove_entry(entry.lock_ref().id).await;
                                                    }))
                                                }))
                                            }),
                                        ])
                                    })
                                )
                            })
                        }
                        Column::Bundle => {
                            html!("locale-cell", {
                                .child(html!("select" => HtmlSelectElement, {
                                    .with_node!(elem => {
                                        .event(clone!(state, entry => move |_: events::Input| {
                                            let value: String = elem.value();
                                            let value = Uuid::parse_str(&value).unwrap_ji();
                                            let mut entry = entry.lock_mut();
                                            entry.bundle_id = value;
                                            Self::save_entry(state.clone(), entry.clone());
                                        }))
                                        .children(
                                            state.bundles
                                                .lock_ref()
                                                .iter()
                                                .map(|(bundle, _)| {
                                                    html!("option", {
                                                        .prop("text", &bundle.name)
                                                        .prop("value", &bundle.id.to_string())
                                                        .prop("selected", entry_ref.bundle_id == bundle.id)
                                                    })
                                                })
                                        )
                                    })
                                }))
                            })
                        },
                    }
                }))
            )
        })
    }
}
