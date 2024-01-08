use anyhow::{Ok, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};

#[allow(unused)]
trait MessageHandling {
    fn message_handling(&mut self, msg: (Sender<Message>, Receiver<Message>)) -> Result<()> {
        Ok(())
    }
}

#[allow(unused)]
enum Message {
    Quit,
}

struct App {
    message: (Sender<Message>, Receiver<Message>),
    systems: Vec<Box<dyn MessageHandling>>,
}
impl App {
    fn new() -> Self {
        Self {
            message: unbounded(),
            systems: vec![],
        }
    }
    fn run(&mut self) -> Result<()> {
        for system in &mut self.systems {
            system.message_handling(self.message.clone())?;
        }
        self.message_handling(self.message.clone())?;
        Ok(())
    }
}
impl MessageHandling for App {
    fn message_handling(&mut self, msg: (Sender<Message>, Receiver<Message>)) -> Result<()> {
        match msg.1.recv()? {
            Message::Quit => drop(msg),
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    App::new().run()?;
    Ok(())
}
