use std::rc::Rc;

use dominator::{clone, html, link, Dom};
use futures_signals::signal::SignalExt;
use shared::{
    domain::{circle::Circle, user::public_user::PublicUser},
    media::MediaLibrary,
};
use utils::routes::{CommunityCirclesRoute, CommunityMembersRoute, CommunityRoute, Route};
use wasm_bindgen::JsValue;

use super::CommunityLanding;

const STR_SEE_MORE: &str = "See more";

impl CommunityLanding {
    pub fn render(self: Rc<Self>) -> Dom {
        let state = self;
        state.load_data();

        html!("community-landing", {
            .children_signal_vec(state.top_members.signal_ref(clone!(state => move |top_members| {
                match top_members {
                    None => vec![html!("progress", {
                        .prop("slot", "members")
                    })],
                    Some(top_members) => {
                        top_members.iter().map(|member| {
                            state.render_member(member)
                        }).collect()
                    },
                }
            })).to_signal_vec())
            .child(html!("button-rect", {
                .prop("slot", "members-link")
                .prop("color", "blue")
                .text(STR_SEE_MORE)
                .apply(move |dom| dominator::on_click_go_to_url!(dom, {
                    Route::Community(CommunityRoute::Members(CommunityMembersRoute::List)).to_string()
                }))
            }))
            .children_signal_vec(state.top_circles.signal_ref(clone!(state => move |top_circles| {
                match top_circles {
                    None => vec![html!("progress", {
                        .prop("slot", "circles")
                    })],
                    Some(top_circles) => {
                        top_circles.iter().map(|circle| {
                            state.render_circle(circle)
                        }).collect()
                    },
                }
            })).to_signal_vec())
            .child(html!("button-rect", {
                .prop("slot", "circles-link")
                .prop("color", "blue")
                .text(STR_SEE_MORE)
                .apply(move |dom| dominator::on_click_go_to_url!(dom, {
                    Route::Community(CommunityRoute::Circles(CommunityCirclesRoute::List)).to_string()
                }))
            }))
        })
    }

    fn render_member(self: &Rc<Self>, member: &PublicUser) -> Dom {
        link!(Route::Community(CommunityRoute::Members(CommunityMembersRoute::Member(member.id))).to_string(), {
            .prop("slot", "members")
            .child(html!("profile-image", {
                .style("height", "50px")
                .style("width", "50px")
                .style("overflow", "hidden")
                .style("border-radius", "50%")
                .prop("slot", "profile-image")
                .prop("imageId", {
                    match &member.profile_image {
                        Some(image_id) => JsValue::from_str(&image_id.0.to_string()),
                        None => JsValue::UNDEFINED,
                    }
                })
            }))
            .text(&format!("{} {}", member.given_name, member.family_name))
        })
    }

    fn render_circle(self: &Rc<Self>, circle: &Circle) -> Dom {
        link!(Route::Community(CommunityRoute::Circles(CommunityCirclesRoute::Circle(circle.id))).to_string(), {
            .prop("slot", "circles")
            .prop("title", &circle.display_name)
            .child(html!("img-ji", {
                .style("height", "70px")
                .style("width", "70px")
                .style("box-shadow", "0 0 8px 0 rgba(0, 0, 0, 0.06)")
                .style("border", "solid 1px var(--light-gray-1)")
                .style("border-radius", "50%")
                .style("overflow", "hidden")
                .prop("lib", MediaLibrary::User.to_str())
                .prop("id", &circle.image.0.to_string())
            }))
            .child(html!("span", {
                .style("white-space", "nowrap")
                .style("overflow", "hidden")
                .style("text-overflow", "ellipsis")
                .style("max-width", "100%")
                .text(&circle.display_name)
            }))
        })
    }
}
