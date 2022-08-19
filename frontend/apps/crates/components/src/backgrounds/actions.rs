use super::state::*;
use futures_signals::signal::Mutable;
use shared::domain::module::body::Background;

pub enum Layer {
    One,
    Two,
}

impl Backgrounds {
    pub fn delete_layer(&self, layer: Layer) {
        self.get_layer(layer).set(None);
        self.call_change();
    }

    pub fn set_layer(&self, layer: Layer, bg: Option<Background>) {
        self.get_layer(layer).set(bg);
        self.call_change();
    }

    //helper
    fn get_layer(&self, layer: Layer) -> &Mutable<Option<Background>> {
        match layer {
            Layer::One => &self.layer_1,
            Layer::Two => &self.layer_2,
        }
    }

    // Internal - saving/history is done on the module level
    fn call_change(&self) {
        if let Some(on_change) = self.callbacks.on_change.as_ref() {
            (on_change)(self.to_raw());
        }
    }
}
