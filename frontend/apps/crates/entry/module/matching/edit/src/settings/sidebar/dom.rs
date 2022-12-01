use dominator::{clone, Dom};
use std::rc::Rc;

use super::state::*;
use futures_signals::signal::{always, SignalExt};

use components::module::_common::edit::settings::prelude::*;

pub fn render(state: Rc<SidebarSettings>) -> Dom {
    render_settings(Rc::new(ModuleSettings {
        lines: vec![
            ModuleSettingsLine::new(
                LineKind::GameDisplay,
                vec![
                    Some(SettingsButton::new_value(
                        SettingsButtonKind::NumChoices,
                        || always(true),
                        SettingsValue::new(
                            state.settings().n_choices.get(),
                            clone!(state => move |value| {
                                state.set_n_choices(value);
                            }),
                        ),
                    )),
                    Some(SettingsButton::new_click(
                        SettingsButtonKind::Swap,
                        clone!(state => move || {
                            state.base.extra.settings.swap.signal()
                        }),
                        clone!(state => move || {
                            state.toggle_swap();
                        }),
                    )),
                ],
            ),
            ModuleSettingsLine::new(
                LineKind::Rounds,
                vec![Some(SettingsButton::new_value(
                    SettingsButtonKind::Rounds,
                    || always(true),
                    SettingsValue::new(
                        state.settings().n_rounds.get(),
                        clone!(state => move |value| {
                            state.set_n_rounds(value);
                        }),
                    ),
                ))],
            ),
            ModuleSettingsLine::new(
                LineKind::TimeLimit,
                vec![
                    Some(SettingsButton::new_click(
                        SettingsButtonKind::TimeLimitOff,
                        clone!(state => move || {
                            state.base.extra.settings.has_time_limit
                                .signal()
                                .map(|flag| !flag)
                        }),
                        clone!(state => move || {
                            state.set_has_time_limit(false);
                        }),
                    )),
                    Some(SettingsButton::new_value_click(
                        SettingsButtonKind::TimeLimit,
                        clone!(state => move || {
                            state.base.extra.settings.has_time_limit
                                .signal()
                        }),
                        SettingsValue::new(
                            state.settings().time_limit.get(),
                            clone!(state => move |value| {
                                state.set_time_limit(value);
                            }),
                        ),
                        clone!(state => move || {
                            state.set_has_time_limit(true);
                        }),
                    )),
                ],
            ),
            // NOTE - not including score until player/jig story is resolved
        ],
    }))
}
