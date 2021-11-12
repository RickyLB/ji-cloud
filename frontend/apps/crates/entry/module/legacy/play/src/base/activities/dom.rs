use super::{
    ask_questions::AskQuestions, say_something::SaySomething, soundboard::Soundboard, video::Video,
};
use crate::base::state::Base;
use dominator::{html, Dom};
use shared::domain::jig::module::body::legacy::activity::Activity;
use std::rc::Rc;

impl Base {
    pub fn render_activity(self: Rc<Self>) -> Dom {
        match self.slide.activity.clone() {
            Some(activity) => match activity {
                Activity::SaySomething(activity) => {
                    SaySomething::new(self.clone(), activity).render()
                }
                Activity::Soundboard(activity) => Soundboard::new(self.clone(), activity).render(),
                Activity::Video(activity) => Video::new(self.clone(), activity).render(),
                Activity::AskQuestions(activity) => {
                    AskQuestions::new(self.clone(), activity).render()
                }
                // _ => html!("empty-fragment"),
            },
            None => html!("empty-fragment"),
        }
    }
}
