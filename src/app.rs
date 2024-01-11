use std::{any::Any, thread, time};

use anyhow::{anyhow, Ok, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};
use crossterm::event;
use ratatui::prelude::*;

pub trait ChannelComponent<T> {
    fn register_send_channel(&mut self, send_channel: Sender<T>);
    fn message(&mut self, msg: T) -> Result<()>;
}
pub trait EventComponent {
    fn event(&mut self, event: event::Event) -> Result<()>;
}
pub trait RenderComponent {
    fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;
}

pub enum Message {
    Quit,
}

struct Channel<T> {
    send_channel: Sender<T>,
    recv_channel: Receiver<T>,
}
impl<T> Channel<T> {
    fn clone_send_channel(&mut self) -> Sender<T> {
        self.send_channel.clone()
    }
    fn recv_pool(&mut self) -> Result<Option<T>> {
        match self.recv_channel.try_recv() {
            core::result::Result::Ok(msg) => return Ok(Some(msg)),
            Err(err) => match err {
                crossbeam_channel::TryRecvError::Empty => return Ok(None),
                crossbeam_channel::TryRecvError::Disconnected => {
                    return Err(anyhow!("信道断开连接"))
                }
            },
        }
    }
}
impl<T> Default for Channel<T> {
    fn default() -> Self {
        let (send_channel, recv_channel) = unbounded();
        Self {
            send_channel,
            recv_channel,
        }
    }
}
impl<T> Clone for Channel<T> {
    fn clone(&self) -> Self {
        Self {
            send_channel: self.send_channel.clone(),
            recv_channel: self.recv_channel.clone(),
        }
    }
}

#[derive(Default)]
pub struct App {
    channel: Channel<Message>,
    components: Vec<Box<dyn Any + Send + Sync>>,
    is_quit: bool,
}
impl App {
    pub fn set_components(mut self, components: Vec<Box<dyn Any + Send + Sync>>) -> Self {
        self.components = components;
        self
    }
    pub fn run(mut self) -> Result<()> {
        for mut component in self.components {
            if let Some(channel_component) =
                component.downcast_mut::<Box<dyn ChannelComponent<Message>>>()
            {
                channel_component.register_send_channel(self.channel.clone_send_channel());
            }
            let mut async_loop_channel = self.channel.clone();
            thread::spawn(move || {
                while !self.is_quit {
                    if event::poll(time::Duration::from_millis(0))? {
                        if let Some(event_component) =
                            component.downcast_mut::<Box<dyn EventComponent>>()
                        {
                            event_component.event(event::read()?)?;
                        }
                    }
                    if let Some(msg) = async_loop_channel.recv_pool()? {
                        if let Some(channel_component) =
                            component.downcast_mut::<Box<dyn ChannelComponent<Message>>>()
                        {
                            channel_component.message(msg)?;
                        }
                    }
                    // if let Some(render_component) =
                    //     component.downcast_mut::<Box<dyn RenderComponent>>()
                    // {
                    //     render_component.render()?;
                    // }
                }
                Ok(())
            });
        }
        let mut main_loop_channel = self.channel.clone();
        while !self.is_quit {
            if let Some(msg) = main_loop_channel.recv_pool()? {
                match msg {
                    Message::Quit => self.is_quit = true,
                }
            }
        }
        Ok(())
    }
}
