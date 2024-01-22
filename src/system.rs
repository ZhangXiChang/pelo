use std::{
    io,
    sync::{Arc, Mutex},
    time,
};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListState},
    Frame, Terminal,
};

trait SystemComponent {
    fn register_system(&mut self, system: Arc<Mutex<System>>);
}
trait RenderComponent {
    fn render(&mut self, frame: &mut Frame, area: Rect);
}
trait EventComponent {
    fn event(&mut self, event: Event);
}

#[derive(Default)]
struct MenuInfo<'a> {
    title: &'a str,
    items: Vec<&'a str>,
    items_selected: Option<usize>,
}
struct Menu<'a> {
    title: &'a str,
    items: Vec<&'a str>,
    items_state: ListState,
    system: Option<Arc<Mutex<System>>>,
}
impl<'a> Menu<'a> {
    fn new(info: MenuInfo<'a>) -> Self {
        Self {
            title: info.title,
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
}
impl RenderComponent for Menu<'_> {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            List::new(self.items.clone())
                .block(Block::new().borders(Borders::ALL).title(self.title))
                .highlight_style(Style::new().add_modifier(Modifier::BOLD))
                .highlight_symbol(">> "),
            area,
            &mut self.items_state,
        )
    }
}
impl EventComponent for Menu<'_> {
    fn event(&mut self, event: Event) {
        match event {
            Event::Key(key) => match key.kind {
                KeyEventKind::Press => match key.code {
                    KeyCode::Esc => {
                        if let Some(system) = &mut self.system {
                            system.lock().unwrap().is_run = false;
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
}

pub struct SystemInfo {
    is_run: bool,
}
impl Default for SystemInfo {
    fn default() -> Self {
        Self { is_run: true }
    }
}
pub struct System {
    is_run: bool,
}
impl System {
    pub fn nwe(info: SystemInfo) -> Self {
        Self {
            is_run: info.is_run,
        }
    }
    pub fn run(self) -> Result<()> {
        let system = Arc::new(Mutex::new(self));
        io::stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        let widgets = vec![Arc::new(Mutex::new(Menu::new(MenuInfo {
            title: "主菜单",
            items: vec!["开始", "结束"],
            items_selected: Some(0),
        })))];
        for widget in widgets.clone() {
            widget.lock().unwrap().register_system(system.clone());
        }
        while system.lock().unwrap().is_run {
            let mut event = None;
            if event::poll(time::Duration::from_millis(0))? {
                event = Some(event::read()?);
            }
            if let Some(event) = event {
                for widget in widgets.clone() {
                    let event = event.clone();
                    tokio::spawn(async move {
                        widget.lock().unwrap().event(event);
                        anyhow::Ok(())
                    });
                }
            }
            terminal.draw(|frame| {
                for widget in widgets.clone() {
                    widget.lock().unwrap().render(frame, frame.size());
                }
            })?;
        }
        disable_raw_mode()?;
        io::stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
}
