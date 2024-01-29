use std::{
    any::Any,
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
    backend::{Backend, CrosstermBackend},
    layout::{Layout, Rect},
    Frame, Terminal,
};

pub trait SystemComponent: Send {
    fn as_widget_layout(&mut self) -> Option<&mut WidgetLayout> {
        None
    }
}
#[allow(unused)]
pub trait WidgetComponent: Send {
    fn public(&mut self) -> Option<&mut dyn Any> {
        None
    }
    fn register_system(&mut self, system: Arc<Mutex<System>>) {}
    fn event(&mut self, event: Event) {}
    fn render(&mut self, frame: &mut Frame, area: Rect) {}
}

pub struct Widget {
    pub component: Arc<Mutex<Box<dyn WidgetComponent>>>,
    pub layout_area_index: usize,
}
impl Widget {
    pub fn new(component: Box<dyn WidgetComponent>, layout_area_index: usize) -> Self {
        Self {
            component: Arc::new(Mutex::new(component)),
            layout_area_index,
        }
    }
}
#[derive(Default)]
pub struct WidgetLayout {
    pub layout: Layout,
    pub widgets: Vec<Widget>,
    pub super_layout_area_index: usize,
    pub sub_layout: Option<Vec<Box<WidgetLayout>>>,
}
impl SystemComponent for WidgetLayout {
    fn as_widget_layout(&mut self) -> Option<&mut WidgetLayout> {
        Some(self)
    }
}

#[derive(Default)]
pub struct SystemInfo {
    pub components: Vec<Box<dyn SystemComponent>>,
}
pub struct System {
    is_run: bool,
    components: Vec<Arc<Mutex<Box<dyn SystemComponent>>>>,
}
#[allow(unused)]
impl System {
    pub fn nwe(info: SystemInfo) -> Self {
        Self {
            is_run: true,
            components: {
                let mut components = vec![];
                for component in info.components {
                    components.push(Arc::new(Mutex::new(component)));
                }
                components
            },
        }
    }
    pub fn run(self) -> Result<()> {
        let system = Arc::new(Mutex::new(self));
        io::stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        let mut components;
        {
            components = system.lock().unwrap().components.clone();
        }
        for component in &mut components {
            if let Some(widget_layout) = component.lock().unwrap().as_widget_layout() {
                for widget in &widget_layout.widgets {
                    widget
                        .component
                        .lock()
                        .unwrap()
                        .register_system(system.clone());
                }
            }
        }
        while system.lock().unwrap().is_run {
            for component in &mut components {
                if let Some(widget_layout) = component.lock().unwrap().as_widget_layout() {
                    terminal.autoresize()?;
                    let layout_area = widget_layout.layout.split(terminal.get_frame().size());
                    for widget in &widget_layout.widgets {
                        let widget_async = widget.component.clone();
                        tokio::spawn(async move {
                            if event::poll(time::Duration::from_millis(0))? {
                                widget_async.lock().unwrap().event(event::read()?);
                            }
                            anyhow::Ok(())
                        });
                        widget.component.lock().unwrap().render(
                            &mut terminal.get_frame(),
                            layout_area[widget.layout_area_index],
                        );
                    }
                    terminal.flush()?;
                    terminal.hide_cursor()?;
                    terminal.swap_buffers();
                    terminal.backend_mut().flush()?;
                }
            }
        }
        disable_raw_mode()?;
        io::stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
    pub fn quit(&mut self) {
        self.is_run = false;
    }
}
