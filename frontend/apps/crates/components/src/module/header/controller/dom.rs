use dominator::{Dom, html, clone};
use futures_signals::signal_vec::SignalVecExt;
use std::rc::Rc;
use utils::events;
use futures_signals::signal::SignalExt;
use crate::module::history::state::HistoryState;

pub struct ControllerDom {
}

//TODO - move on_undoredo into HistoryState itself
impl ControllerDom {
    pub fn render<T, ON_CHANGE, ON_UNDOREDO, ON_PREVIEW>(history: Rc<HistoryState<T, ON_CHANGE, ON_UNDOREDO>>, on_preview: ON_PREVIEW) -> Dom 
    where
        T: Clone + 'static,
        ON_CHANGE: Fn(Option<T>) + 'static,
        ON_UNDOREDO: Fn(Option<T>) + 'static,
        ON_PREVIEW: Fn() + 'static,
    {
        html!("module-header-controller", {
            .property("slot", "controller")
            .property_signal("undoable", history.undoable())
            .property_signal("redoable", history.redoable())
            .event(clone!(history => move |evt:events::CustomString| {
                match evt.value().as_ref() {
                    "undo" => {
                        history.undo();
                    },
                    "redo" => {
                        history.redo();
                    },
                    "preview" => {
                        on_preview();
                    }
                    _ => {}
                };
            }))
        })
    }
}
