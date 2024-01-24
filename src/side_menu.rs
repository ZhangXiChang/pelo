use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::*};

use crate::system::*;

pub struct SideMenu {
    title: String,
    title_style: Modifier,
    pub items: Vec<String>,
    items_state: ListState,
    system: Option<Arc<Mutex<System>>>,
}
impl SideMenu {
    pub fn new() -> Self {
        Self {
            title: "副菜单".to_string(),
            title_style: Modifier::default(),
            items: vec![],
            items_state: ListState::default(),
            system: None,
        }
    }
    fn select_last_item(&mut self) {
        if let Some(item_index) = self.items_state.selected() {
            if item_index > 0 {
                self.items_state.select(Some(item_index - 1));
            }
        }
    }
    fn select_next_item(&mut self) {
        if let Some(item_index) = self.items_state.selected() {
            if item_index < self.items.len() - 1 {
                self.items_state.select(Some(item_index + 1));
            }
        }
    }
}
impl SystemComponent for SideMenu {
    fn public(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }
    fn register_system(&mut self, system: Arc<Mutex<System>>) {
        self.system = Some(system);
    }
    fn event(&mut self, event: Event) {
        match event {
            Event::Key(key) => match key.kind {
                KeyEventKind::Press => match key.code {
                    KeyCode::Up => self.select_last_item(),
                    KeyCode::Down => self.select_next_item(),
                    KeyCode::Enter => {
                        if let Some(selected) = self.items_state.selected() {
                            match selected {
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }
    }
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            List::new(self.items.clone())
                .block(
                    Block::new()
                        .borders(Borders::ALL)
                        .title(self.title.clone().add_modifier(self.title_style)),
                )
                .highlight_style(Style::new().add_modifier(Modifier::BOLD))
                .highlight_symbol(">> "),
            area,
            &mut self.items_state,
        )
    }
}
