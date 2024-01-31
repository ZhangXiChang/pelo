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
    backend::CrosstermBackend,
    layout::{Layout, Rect},
    Frame, Terminal,
};

pub trait SystemComponent: Send {
    fn as_widget_layout(&self) -> Option<&WidgetLayout> {
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
    fn as_widget_layout(&self) -> Option<&WidgetLayout> {
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
        let mut components;
        {
            components = system.lock().unwrap().components.clone();
        }
        let mut widget_component = None;
        let mut terminal = None;
        for component in &components {
            if let Some(_) = component.lock().unwrap().as_widget_layout() {
                widget_component = Some(component);
                io::stdout().execute(EnterAlternateScreen)?;
                enable_raw_mode()?;
                terminal = Some(Terminal::new(CrosstermBackend::new(io::stdout()))?);
                break;
            }
        }
        for component in &components {
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
            if let Some(widget_component) = widget_component {
                if let Some(widget_layout) = widget_component.lock().unwrap().as_widget_layout() {
                    terminal.as_mut().unwrap().draw(|frame| {
                        Self::draw_widgets(frame, widget_layout, frame.size());
                    });
                }
            }
        }
        if let Some(_) = terminal {
            disable_raw_mode()?;
            io::stdout().execute(LeaveAlternateScreen)?;
        }
        Ok(())
    }
    pub fn quit(&mut self) {
        self.is_run = false;
    }
    pub fn query_widget_layout(&mut self) -> Option<Arc<Mutex<Box<dyn SystemComponent>>>> {
        for component in &self.components {
            if let Some(_) = component.lock().unwrap().as_widget_layout() {
                return Some(component.clone());
            }
        }
        None
    }
    fn draw_widgets(frame: &mut Frame, widget_layout: &WidgetLayout, super_area: Rect) {
        for widget in &widget_layout.widgets {
            let widget = widget.component.clone();
            tokio::spawn(async move {
                if event::poll(time::Duration::from_millis(0))? {
                    widget.lock().unwrap().event(event::read()?);
                }
                anyhow::Ok(())
            });
        }
        let layout_area = widget_layout.layout.split(frame.size());
        for widget in &widget_layout.widgets {
            widget
                .component
                .lock()
                .unwrap()
                .render(frame, layout_area[widget.layout_area_index]);
        }
    }
}
