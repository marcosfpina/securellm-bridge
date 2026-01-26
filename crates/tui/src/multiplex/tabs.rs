// ! Tab system for multiplex sessions

use super::{Layout, Pane, PaneId};
use std::collections::HashMap;

pub struct Tab {
    pub name: String,
    pub layout: Layout,
    pub panes: HashMap<PaneId, Pane>,
    pub focused_pane: Option<PaneId>,
}

impl Tab {
    pub fn new(name: impl Into<String>) -> Self {
        // Create initial chat pane
        let chat_pane = Pane::Chat(super::pane::ChatPane::new());
        let pane_id = chat_pane.id();
        
        let mut panes = HashMap::new();
        panes.insert(pane_id, chat_pane);
        
        Self {
            name: name.into(),
            layout: Layout::single(pane_id),
            panes,
            focused_pane: Some(pane_id),
        }
    }
    
    pub fn split_horizontal(&mut self) {
        let new_pane = Pane::Chat(super::pane::ChatPane::new());
        let new_id = new_pane.id();
        
        self.panes.insert(new_id, new_pane);
        self.layout = self.layout.clone().split_horizontal(new_id, 50);
        self.focused_pane = Some(new_id);
    }
    
    pub fn split_vertical(&mut self) {
        let new_pane = Pane::Chat(super::pane::ChatPane::new());
        let new_id = new_pane.id();
        
        self.panes.insert(new_id, new_pane);
        self.layout = self.layout.clone().split_vertical(new_id, 50);
        self.focused_pane = Some(new_id);
    }
    
    pub fn get_focused_pane(&self) -> Option<&Pane> {
        self.focused_pane.and_then(|id| self.panes.get(&id))
    }
    
    pub fn get_focused_pane_mut(&mut self) -> Option<&mut Pane> {
        self.focused_pane.and_then(|id| self.panes.get_mut(&id))
    }
    
    /// Navigate to next pane (for Tab key cycling)
    pub fn focus_next(&mut self) {
        let ids = self.layout.pane_ids();
        if ids.is_empty() {
            return;
        }
        
        if let Some(current) = self.focused_pane {
            if let Some(pos) = ids.iter().position(|id| *id == current) {
                let next_pos = (pos + 1) % ids.len();
                self.focused_pane = Some(ids[next_pos]);
            }
        }
    }
}

pub struct TabBar {
    pub tabs: Vec<Tab>,
    pub active_index: usize,
}

impl TabBar {
    pub fn new() -> Self {
        Self {
            tabs: vec![Tab::new("Main")],
            active_index: 0,
        }
    }
    
    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_index]
    }
    
    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_index]
    }
    
    pub fn new_tab(&mut self, name: impl Into<String>) {
        self.tabs.push(Tab::new(name));
        self.active_index = self.tabs.len() - 1;
    }
    
    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_index = (self.active_index + 1) % self.tabs.len();
        }
    }
    
    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_index = if self.active_index == 0 {
                self.tabs.len() - 1
            } else {
                self.active_index - 1
            };
        }
    }
    
    pub fn close_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_index);
            if self.active_index >= self.tabs.len() {
                self.active_index = self.tabs.len() - 1;
            }
        }
    }
}

impl Default for TabBar {
    fn default() -> Self {
        Self::new()
    }
}
