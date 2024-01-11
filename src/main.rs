mod app;

use anyhow::{Ok, Result};
use app::*;
use crossbeam_channel::Sender;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::*};

#[derive(Default)]
struct Menu {
    title: String,
    items: Vec<String>,
    selected_items: ListState,
    msg: Option<Sender<Message>>,
}
impl Menu {
    fn set_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }
    fn set_items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }
    fn set_def_selected_items(mut self, selected_items: usize) -> Self {
        self.selected_items.select(Some(selected_items));
        self
    }
}
impl ChannelComponent<Message> for Menu {
    fn register_send_channel(&mut self, send_channel: Sender<Message>) {
        self.msg = Some(send_channel);
    }
    fn message(&mut self, msg: Message) -> Result<()> {
        match msg {
            _ => (),
        }
        Ok(())
    }
}
impl EventComponent for Menu {
    fn event(&mut self, event: event::Event) -> Result<()> {
        match event {
            Event::Key(key) => match key.kind {
                KeyEventKind::Press => match key.code {
                    KeyCode::Esc => {
                        if let Some(msg) = &self.msg {
                            msg.send(Message::Quit)?
                        }
                    }
                    KeyCode::Up => {
                        if let Some(i) = self.selected_items.selected() {
                            if i > 0 {
                                self.selected_items.select(Some(i - 1));
                            }
                        } else {
                            self.selected_items.select(Some(0));
                        }
                    }
                    KeyCode::Down => {
                        if let Some(i) = self.selected_items.selected() {
                            if i < self.items.len() - 1 {
                                self.selected_items.select(Some(i + 1));
                            } else {
                                self.selected_items.select(Some(0));
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(i) = self.selected_items.selected() {
                            match i {
                                0 => (),
                                1 => {
                                    if let Some(msg) = &self.msg {
                                        msg.send(Message::Quit)?
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
        Ok(())
    }
}
impl RenderComponent for Menu {
    fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_stateful_widget(
            List::new(self.items.clone())
                .block(Block::new().borders(Borders::ALL).title(self.title.clone()))
                .highlight_style(Style::new().add_modifier(Modifier::BOLD))
                .highlight_symbol(">> "),
            area,
            &mut self.selected_items,
        );
        Ok(())
    }
}

fn main() -> Result<()> {
    App::default()
        .set_components(vec![Box::new(Menu::default())])
        .run()?;
    Ok(())
}
