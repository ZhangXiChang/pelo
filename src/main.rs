use std::{any::Any, time};

use anyhow::{anyhow, Ok, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::*};

trait ChannelComponent<T> {
    fn register_channel(&mut self, channel: Channel<T>);
}
trait EventComponent {
    fn event(&mut self, event: event::Event) -> Result<()>;
}
trait RenderComponent {
    fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;
}

enum Message {
    Quit,
}

struct Channel<T> {
    smsg: Sender<T>,
    rmsg: Receiver<T>,
}
impl<T> Channel<T> {
    fn recv_pool(&mut self) -> Result<Option<T>> {
        match self.rmsg.try_recv() {
            core::result::Result::Ok(msg) => return Ok(Some(msg)),
            Err(err) => match err {
                crossbeam_channel::TryRecvError::Empty => return Ok(None),
                crossbeam_channel::TryRecvError::Disconnected => {
                    return Err(anyhow!("信道断开连接"))
                }
            },
        }
    }
    fn send(&mut self, message: T) -> Result<()> {
        match self.smsg.send(message) {
            core::result::Result::Ok(_) => (),
            Err(msg) => return Err(anyhow!("发送消息失败！消息：{}", msg)),
        }
        Ok(())
    }
}
impl<T> Default for Channel<T> {
    fn default() -> Self {
        let (smsg, rmsg) = unbounded();
        Self { smsg, rmsg }
    }
}
impl<T> Clone for Channel<T> {
    fn clone(&self) -> Self {
        Self {
            smsg: self.smsg.clone(),
            rmsg: self.rmsg.clone(),
        }
    }
}

#[derive(Default)]
struct Menu {
    title: String,
    items: Vec<String>,
    selected_items: ListState,
    channel: Channel<Message>,
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
    fn register_channel(&mut self, channel: Channel<Message>) {
        self.channel = channel;
    }
}
impl EventComponent for Menu {
    fn event(&mut self, event: event::Event) -> Result<()> {
        match event {
            Event::Key(key) => match key.kind {
                KeyEventKind::Press => match key.code {
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
                                1 => self.channel.send(Message::Quit)?,
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

#[derive(Default)]
struct App {
    channel: Channel<Message>,
    components: Vec<Box<dyn Any + Send + Sync>>,
}
impl App {
    // fn set_component(mut self, components: Vec<Box<dyn Any>>) -> Self {
    //     self.components = components;
    //     self
    // }
    fn run(&mut self) -> Result<()> {
        for component in &mut self.components {
            let mut async_loop_channel = self.channel.clone();
            tokio::spawn(async move {
                loop {
                    if event::poll(time::Duration::from_millis(0))? {
                        if let Some(event_component) =
                            component.downcast_mut::<Box<dyn EventComponent>>()
                        {
                            event_component.event(event::read()?)?;
                        }
                        match event::read()? {
                            event::Event::Key(_) => async_loop_channel.send(Message::Quit)?,
                            _ => (),
                        }
                    }
                    if let Some(msg) = async_loop_channel.recv_pool()? {
                        match msg {
                            Message::Quit => break,
                        }
                    }
                }
                Ok(())
            });
        }
        let mut main_loop_channel = self.channel.clone();
        loop {
            if let Some(msg) = main_loop_channel.recv_pool()? {
                match msg {
                    Message::Quit => break,
                }
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    App::default().run()?;
    Ok(())
}
