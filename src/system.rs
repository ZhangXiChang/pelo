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
    fn event(&mut self, event: &Event) {}
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
    pub sub_layout: Vec<Box<WidgetLayout>>,
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
        let mut terminal = None;
        for component in &components {
            if let Some(widget_layout) = component.lock().unwrap().as_widget_layout() {
                io::stdout().execute(EnterAlternateScreen)?;
                enable_raw_mode()?;
                terminal = Some(Terminal::new(CrosstermBackend::new(io::stdout()))?);
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
            for component in &components {
                if let Some(widget_layout) = component.lock().unwrap().as_widget_layout() {
                    let mut event = None;
                    if event::poll(time::Duration::from_millis(0))? {
                        event = Some(event::read()?);
                    }
                    terminal.as_mut().unwrap().draw(|frame| {
                        Self::draw_widgets(&event, frame, widget_layout, frame.size());
                    });
                }
            }
        }
        if terminal.is_some() {
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
            if component.lock().unwrap().as_widget_layout().is_some() {
                return Some(component.clone());
            }
        }
        None
    }
    fn draw_widgets(
        event: &Option<Event>,
        frame: &mut Frame,
        widget_layout: &WidgetLayout,
        super_area: Rect,
    ) {
        let layout_area = widget_layout.layout.split(super_area);
        for widget in &widget_layout.widgets {
            let event = event.clone();
            let widget_component = widget.component.clone();
            tokio::spawn(async move {
                if let Some(event) = event {
                    widget_component.lock().unwrap().event(&event);
                }
                anyhow::Ok(())
            });
            widget
                .component
                .lock()
                .unwrap()
                .render(frame, layout_area[widget.layout_area_index]);
        }
        for widget_layout in &widget_layout.sub_layout {
            Self::draw_widgets(
                event,
                frame,
                widget_layout,
                layout_area[widget_layout.super_layout_area_index],
            );
        }
    }
}
