// ! Multiplex system for Zellij-style TUI

mod pane;
mod layout;
mod tabs;

pub use pane::{Pane, PaneId, ChatPane, ToolPane};
pub use layout::Layout;
pub use tabs::{Tab, TabBar};
