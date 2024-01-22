use std::sync::{Arc, Mutex};

use anyhow::Result;
use ratatui::{prelude::*, widgets::*};

mod system;
use system::*;

#[derive(Default)]
struct MenuInfo<'a> {
    title: &'a str,
    items: Vec<&'a str>,
    items_selected: Option<usize>,
}
struct Menu<'a> {
    title: Span<'a>,
    items: Vec<&'a str>,
    items_state: ListState,
    system: Option<Arc<Mutex<System>>>,
}
impl<'a> Menu<'a> {
    fn new(info: MenuInfo<'a>) -> Self {
        Self {
            title: {
                let mut title = info.title.into();
                if info.items_selected != None {
                    title = info.title.add_modifier(Modifier::REVERSED);
                }
                title
            },
            items: info.items,
            items_state: {
                let mut list_state = ListState::default();
                list_state.select(info.items_selected);
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
impl SystemComponent for Menu<'_> {
    fn register_system(&mut self, system: Arc<Mutex<System>>) {
        self.system = Some(system);
    }
    fn event(&mut self, event: Event) {
        match event {
            Event::Key(key) => match key.kind {
                KeyEventKind::Press => match key.code {
                    KeyCode::Esc => {
                        if let Some(system) = &mut self.system {
                            system.lock().unwrap().quit();
                        }
                    }
                    KeyCode::Up => self.select_last_item(),
                    KeyCode::Down => self.select_next_item(),
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

#[tokio::main]
async fn main() -> Result<()> {
    System::nwe(SystemInfo {
        system_components: vec![
            Box::new(Menu::new(MenuInfo {
                title: "主菜单",
                items: vec!["开始", "结束"],
                items_selected: Some(0),
            })),
            // Box::new(Menu::new(MenuInfo {
            //     title: "副菜单",
            //     ..Default::default()
            // })),
        ],
    })
    .run()?;
    Ok(())
}
