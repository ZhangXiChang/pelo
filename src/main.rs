use std::io;

use anyhow::Result;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use log4rs::{append::file::FileAppender, encode::pattern::PatternEncoder};
use ratatui::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    //初始化日志系统
    log4rs::init_config(
        log4rs::Config::builder()
            .appender(
                log4rs::config::Appender::builder().build(
                    "file_log",
                    Box::new(
                        FileAppender::builder()
                            .encoder(Box::new(PatternEncoder::new(
                                "[{d(%Y-%m-%d %H:%M:%S)}][{l}]:{m}{n}",
                            )))
                            .append(false)
                            .build("./logs/error.log")?,
                    ),
                ),
            )
            .build(
                log4rs::config::Root::builder()
                    .appender("file_log")
                    .build(log::LevelFilter::Error),
            )?,
    )?;
    //初始化终端
    io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut _terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    //恢复终端
    io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
