// SPDX-License-Identifier: GPL-3.0

use crate::app::Message;
use cosmic::widget::menu;

/// Represents a Action that executes after clicking on the application Menu
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    /// Open the About [`ContextPage`] of the application
    About,
    /// Open the Settings [`ContextPage`] of the application
    Settings,
}

impl menu::action::MenuAction for MenuAction {
    type Message = crate::app::Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::MenuAction(MenuAction::About),
            MenuAction::Settings => Message::MenuAction(MenuAction::Settings),
        }
    }
}
