use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::*};

use crate::{system::*, SideMenu};

pub struct MainMenu {
    title: String,
    title_style: Modifier,
    items: Vec<String>,
    items_state: ListState,
    system: Option<Arc<Mutex<System>>>,
    focus: bool,
}
impl MainMenu {
    pub fn new() -> Self {
        Self {
            title: "主菜单".to_string(),
            title_style: Modifier::REVERSED,
            items: vec!["开始".to_string(), "结束".to_string()],
            items_state: {
                let mut list_state = ListState::default();
                list_state.select(Some(0));
                list_state
            },
            system: None,
            focus: true,
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
impl WidgetComponent for MainMenu {
    fn public(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }
    fn register_system(&mut self, system: Arc<Mutex<System>>) {
        self.system = Some(system);
    }
    fn event(&mut self, event: &Event) {
        if self.focus {
            match event {
                Event::Key(key) => match key.kind {
                    KeyEventKind::Press => match key.code {
                        KeyCode::Up => self.select_last_item(),
                        KeyCode::Down => self.select_next_item(),
                        KeyCode::Enter => {
                            if let Some(selected) = self.items_state.selected() {
                                match selected {
                                    0 => {
                                        self.focus = false;
                                        self.title_style.remove(Modifier::REVERSED);
                                        if let Some(side_main) = self
                                            .system
                                            .as_ref()
                                            .unwrap()
                                            .lock()
                                            .unwrap()
                                            .query_widget_layout()
                                            .unwrap()
                                            .lock()
                                            .unwrap()
                                            .as_widget_layout()
                                            .unwrap()
                                            .sub_layout[0]
                                            .widgets[1]
                                            .component
                                            .lock()
                                            .unwrap()
                                            .public()
                                            .unwrap()
                                            .downcast_mut::<SideMenu>()
                                        {
                                            side_main.items = vec![
                                                "第一".to_string(),
                                                "第二".to_string(),
                                                "第三".to_string(),
                                            ];
                                            side_main.items_state.select(Some(0));
                                            side_main.title_style |= Modifier::REVERSED;
                                            side_main.focus = true;
                                        }
                                    }
                                    1 => self.system.as_ref().unwrap().lock().unwrap().quit(),
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
