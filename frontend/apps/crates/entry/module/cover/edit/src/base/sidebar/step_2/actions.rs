use crate::base::state::*;
use dominator::clone;
use shared::domain::module::body::Audio;

impl Base {
    pub fn set_instructions_audio(&self, audio: Option<Audio>) {
        self.instructions
            .replace_with(clone!(audio => move |instructions| {
                let mut instructions = instructions.clone();
                instructions.audio = audio;
                instructions
            }));

        self.history.push_modify(move |raw| {
            if let Some(content) = &mut raw.content {
                content.base.instructions.audio = audio;
            }
        });
    }
}
