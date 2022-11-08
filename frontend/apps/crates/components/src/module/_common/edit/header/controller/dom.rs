use dominator::{clone, html, Dom};

use std::{fmt::Debug, rc::Rc};
use utils::events;

use crate::module::_common::edit::history::state::HistoryState;

#[derive(Debug)]
pub struct ControllerDom {}

//TODO - move on_undoredo into HistoryState itself
impl ControllerDom {
    pub fn render<T, OnChangeFn, OnUndoRedoFn, OnPreviewFn>(
        history: Rc<HistoryState<T, OnChangeFn, OnUndoRedoFn>>,
        on_preview: OnPreviewFn,
    ) -> Dom
    where
        T: Clone + Debug + 'static,
        OnChangeFn: Fn(T) + 'static,
        OnUndoRedoFn: Fn(T) + 'static,
        OnPreviewFn: Fn() + 'static,
    {
        html!("module-header-controller", {
            .prop("slot", "controller")
            .prop_signal("undoable", history.undoable())
            .prop_signal("redoable", history.redoable())
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
