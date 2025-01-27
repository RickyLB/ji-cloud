use std::rc::Rc;

use dominator::{clone, html, Dom};
use futures_signals::{map_ref, signal::Signal, signal_vec::SignalVecExt};
use itertools::Itertools;
use shared::{
    domain::{circle::Circle, user::public_user::PublicUser},
    media::MediaLibrary,
};
use utils::{
    events,
    languages::Language,
    routes::{CommunityCirclesRoute, CommunityMembersRoute, CommunityRoute, Route},
};
use wasm_bindgen::JsValue;

use super::CommunitySearch;

const STR_SEE_MORE: &str = "See more";

impl CommunitySearch {
    pub fn render(self: Rc<Self>) -> Dom {
        let state = self;
        state.search();

        html!("community-search-page", {
            .prop("query", &state.query.q)
            .prop_signal("memberCount", state.member_count.signal())
            .children_signal_vec(state.members.signal_vec_cloned().map(clone!(state => move|member| {
                state.render_member(&member)
            })))
            .child_signal(state.render_see_more_members())
            .prop_signal("circleCount", state.circle_count.signal())
            .children_signal_vec(state.circles.signal_vec_cloned().map(clone!(state => move|circle| {
                state.render_circle(&circle)
            })))
            .child_signal(state.render_see_more_circles())
        })
    }

    fn render_member(self: &Rc<Self>, member: &PublicUser) -> Dom {
        html!("community-list-member", {
            .prop("slot", "members")
            .prop("name", &format!("{} {}", member.given_name, member.family_name))
            // .prop("city", "New York")
            // .prop("state", "NY")
            .apply(|mut dom| {
                if let Some(languages_spoken) = &member.languages_spoken {
                    if languages_spoken.len() > 0 {
                        let languages = languages_spoken.iter().map(|l| Language::code_to_display_name(l)).join(", ");
                        dom = dom.prop("language", languages);
                    };
                };
                dom
            })
            .apply(move |dom| dominator::on_click_go_to_url!(dom, {
                Route::Community(CommunityRoute::Members(CommunityMembersRoute::Member(member.id))).to_string()
            }))
            .child(html!("profile-image", {
                .prop("slot", "img")
                .prop("imageId", {
                    match &member.profile_image {
                        Some(image_id) => JsValue::from_str(&image_id.0.to_string()),
                        None => JsValue::UNDEFINED,
                    }
                })
            }))
        })
    }

    fn render_circle(self: &Rc<Self>, circle: &Circle) -> Dom {
        html!("community-list-circle", {
            .prop("slot", "circles")
            .prop("name", &circle.display_name)
            .prop("memberCount", circle.member_count)
            .prop("description", &circle.description)
            .apply(move |dom| dominator::on_click_go_to_url!(dom, {
                Route::Community(CommunityRoute::Circles(CommunityCirclesRoute::Circle(circle.id))).to_string()
            }))
            .child(html!("img-ji", {
                .prop("slot", "img")
                .prop("lib", MediaLibrary::User.to_str())
                .prop("id", &circle.image.0.to_string())
            }))
            .child(html!("community-list-circle-status", {
                .prop("slot", "status")
                .prop("status", "")
            }))
        })
    }

    fn render_see_more_members(self: &Rc<Self>) -> impl Signal<Item = Option<Dom>> {
        let state = Rc::clone(self);
        map_ref! {
            let member_count = state.member_count.signal(),
            let member_len = state.members.signal_vec_cloned().len() => move {
                if *member_count > *member_len as u32 {
                    Some(html!("button-rect", {
                        .prop("slot", "members-see-more")
                        .prop("color", "blue")
                        .prop_signal("disabled", state.loader.is_loading())
                        .text(STR_SEE_MORE)
                        .event(clone!(state => move |_: events::Click| {
                            state.load_more_members();
                        }))
                    }))
                } else {
                    None
                }
            }
        }
    }

    fn render_see_more_circles(self: &Rc<Self>) -> impl Signal<Item = Option<Dom>> {
        let state = Rc::clone(self);
        map_ref! {
            let circle_count = state.circle_count.signal(),
            let circle_len = state.circles.signal_vec_cloned().len() => move {
                if *circle_count > *circle_len as u32 {
                    Some(html!("button-rect", {
                        .prop("slot", "circles-see-more")
                        .prop("color", "blue")
                        .prop_signal("disabled", state.loader.is_loading())
                        .text(STR_SEE_MORE)
                        .event(clone!(state => move |_: events::Click| {
                            state.load_more_circles();
                        }))
                    }))
                } else {
                    None
                }
            }
        }
    }
}
