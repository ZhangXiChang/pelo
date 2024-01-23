use std::{
    io,
    sync::{Arc, Mutex},
    time,
};

use anyhow::Result;
use crossterm::{
    event::{self, Event},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame, Terminal,
};

#[allow(unused)]
pub trait SystemComponent: Send {
    fn register_system(&mut self, system: Arc<Mutex<System>>) {}
    fn event(&mut self, event: Event) {}
    fn render(&mut self, frame: &mut Frame, area: Rect) {}
}

pub struct SystemInfo {
    pub system_components: Vec<Box<dyn SystemComponent>>,
}
impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            system_components: Default::default(),
        }
    }
}
pub struct System {
    is_run: bool,
    system_components: Vec<Arc<Mutex<Box<dyn SystemComponent>>>>,
}
impl System {
    pub fn nwe(info: SystemInfo) -> Self {
        Self {
            is_run: true,
            system_components: {
                let mut system_components = vec![];
                for system_component in info.system_components {
                    system_components.push(Arc::new(Mutex::new(system_component)));
                }
                system_components
            },
        }
    }
    pub fn run(self) -> Result<()> {
        let system = Arc::new(Mutex::new(self));
        io::stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        for system_component in system.lock().unwrap().system_components.clone() {
            system_component
                .lock()
                .unwrap()
                .register_system(system.clone());
        }
        while system.lock().unwrap().is_run {
            let mut event = None;
            if event::poll(time::Duration::from_millis(0))? {
                event = Some(event::read()?);
            }
            if let Some(event) = event {
                for system_component in system.lock().unwrap().system_components.clone() {
                    let event = event.clone();
                    tokio::spawn(async move {
                        system_component.lock().unwrap().event(event);
                        anyhow::Ok(())
                    });
                }
            }
            terminal.draw(|frame| {
                let root_layout = Layout::new(
                    Direction::Horizontal,
                    [Constraint::Length(21), Constraint::Min(0)],
                )
                .split(frame.size());
                let system_components = system.lock().unwrap().system_components.clone();
                for i in 0..system_components.len() {
                    system_components[i]
                        .lock()
                        .unwrap()
                        .render(frame, root_layout[i]);
                }
            })?;
        }
        disable_raw_mode()?;
        io::stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
    pub fn quit(&mut self) {
        self.is_run = false;
    }
}
