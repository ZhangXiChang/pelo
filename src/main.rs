use std::time;

use anyhow::{anyhow, Ok, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};
use crossterm::event;

enum Message {
    Quit,
}

struct Channel<T> {
    smsg: Sender<T>,
    rmsg: Receiver<T>,
}
impl<T> Channel<T> {
    fn new() -> Self {
        let (smsg, rmsg) = unbounded();
        Self { smsg, rmsg }
    }
    fn recv_pool(&mut self) -> Result<Option<T>> {
        let message;
        match self.rmsg.try_recv() {
            core::result::Result::Ok(msg) => message = Some(msg),
            Err(err) => match err {
                crossbeam_channel::TryRecvError::Empty => message = None,
                crossbeam_channel::TryRecvError::Disconnected => {
                    return Err(anyhow!("信道断开连接"))
                }
            },
        }
        Ok(message)
    }
    fn send(&mut self, message: T) -> Result<()> {
        match self.smsg.send(message) {
            core::result::Result::Ok(_) => (),
            Err(msg) => return Err(anyhow!("发送消息失败！消息：{}", msg)),
        }
        Ok(())
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

#[tokio::main]
async fn main() -> Result<()> {
    let channel = Channel::new();
    let mut async_loop_channel = channel.clone();
    tokio::spawn(async move {
        loop {
            if event::poll(time::Duration::from_millis(0))? {
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
    let mut main_loop_r_msg = channel.clone();
    loop {
        if let Some(msg) = main_loop_r_msg.recv_pool()? {
            match msg {
                Message::Quit => break,
            }
        }
    }
    Ok(())
}
