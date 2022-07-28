use crate::home::search_results::search_results_section::SearchResultsSection;
use dominator::{html, Dom};
use futures_signals::{map_ref, signal::SignalExt};
use std::rc::Rc;

use super::state::SearchResults;

impl SearchResults {
    pub fn render(self: &Rc<Self>) -> Dom {
        let state = self;

        let search_results_signal = |section: Rc<SearchResultsSection>| {
            // Ensure that jigs and resources are rendered until all requests have completed
            let should_render_signal = map_ref! {
                let loading = state.loading.signal_cloned(),
                let total = section.total.signal_cloned()
                    => {
                        !*loading && *total > 0
                    }
            };

            // Map the search results render call outside of map_ref so that we don't capture
            // `section` inside the closure.
            should_render_signal.map(move |should_render| {
                if should_render {
                    Some(section.render())
                } else {
                    None
                }
            })
        };

        html!("home-search-results", {
            .property_signal("loading", state.loading.signal())
            .property_signal("jigCount", state.jigs.total.signal())
            .property_signal("resourceCount", state.resources.total.signal())
            .property_signal("courseCount", state.courses.total.signal())
            .property("query", &state.query)
            .child_signal(search_results_signal(Rc::clone(&state.jigs)))
            .child_signal(search_results_signal(Rc::clone(&state.resources)))
            .child_signal(search_results_signal(Rc::clone(&state.courses)))
        })
    }
}
