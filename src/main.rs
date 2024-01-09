use std::time;

use anyhow::{Ok, Result};
use crossbeam_channel::unbounded;
use crossterm::event;

enum Message {
    Quit,
}

#[tokio::main]
async fn main() -> Result<()> {
    let (s_msg, r_msg) = unbounded::<Message>();
    let async_loop_r_msg = r_msg.clone();
    tokio::spawn(async move {
        loop {
            if event::poll(time::Duration::from_millis(0))? {
                match event::read()? {
                    event::Event::Key(_) => s_msg.send(Message::Quit)?,
                    _ => (),
                }
            }
            match async_loop_r_msg.try_recv() {
                core::result::Result::Ok(msg) => match msg {
                    Message::Quit => {
                        drop(async_loop_r_msg);
                        break;
                    }
                },
                Err(err) => match err {
                    crossbeam_channel::TryRecvError::Empty => (),
                    crossbeam_channel::TryRecvError::Disconnected => break,
                },
            }
        }
        Ok(())
    });
    let main_loop_r_msg = r_msg.clone();
    loop {
        match main_loop_r_msg.try_recv() {
            core::result::Result::Ok(msg) => match msg {
                Message::Quit => {
                    drop(main_loop_r_msg);
                    break;
                }
            },
            Err(err) => match err {
                crossbeam_channel::TryRecvError::Empty => (),
                crossbeam_channel::TryRecvError::Disconnected => break,
            },
        }
    }
    Ok(())
}
