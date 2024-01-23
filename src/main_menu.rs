use std::sync::{Arc, Mutex};

use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::*};

use crate::system::*;

pub struct MainMenu<'a> {
    title: Span<'a>,
    items: Vec<&'a str>,
    items_state: ListState,
    system: Option<Arc<Mutex<System>>>,
}
impl<'a> MainMenu<'a> {
    pub fn new() -> Self {
        Self {
            title: "主菜单".add_modifier(Modifier::REVERSED),
            items: vec!["开始", "结束"],
            items_state: {
                let mut list_state = ListState::default();
                list_state.select(Some(0));
                list_state
            },
            system: None,
        }
    }
    fn select_item(&mut self, selected: Option<usize>) {
        self.items_state.select(selected);
    }
    fn select_last_item(&mut self) {
        if let Some(item_index) = self.items_state.selected() {
            if item_index > 0 {
                self.select_item(Some(item_index - 1));
            }
        }
    }
    fn select_next_item(&mut self) {
        if let Some(item_index) = self.items_state.selected() {
            if item_index < self.items.len() - 1 {
                self.select_item(Some(item_index + 1));
            }
        }
    }
}
impl SystemComponent for MainMenu<'_> {
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
                                0 => (),
                                1 => {
                                    if let Some(system) = &mut self.system {
                                        system.lock().unwrap().quit();
                                    }
                                }
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
                .block(Block::new().borders(Borders::ALL).title(self.title.clone()))
                .highlight_style(Style::new().add_modifier(Modifier::BOLD))
                .highlight_symbol(">> "),
            area,
            &mut self.items_state,
        )
    }
}
