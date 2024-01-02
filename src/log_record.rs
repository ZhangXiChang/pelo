use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    encode::pattern::PatternEncoder,
};

use crate::error::Error;

pub enum LogRecordMode {
    Console,
    File,
}

pub struct LogRecord {
    log_level: log::LevelFilter,
    log_mode: LogRecordMode,
    log_file_path: Option<String>,
}
impl LogRecord {
    pub fn new() -> Self {
        Self {
            log_level: log::LevelFilter::Info,
            log_mode: LogRecordMode::Console,
            log_file_path: Default::default(),
        }
    }
    pub fn log_level(mut self, log_level: log::LevelFilter) -> Self {
        self.log_level = log_level;
        self
    }
    pub fn log_mode(mut self, log_mode: LogRecordMode) -> Self {
        self.log_mode = log_mode;
        self
    }
    pub fn log_file_path(mut self, log_file_path: String) -> Self {
        self.log_file_path = Some(log_file_path);
        self
    }
    pub fn start(self) -> Result<(), Error> {
        let pattern_encoder = Box::new(PatternEncoder::new("[{d(%Y-%m-%d %H:%M:%S)}][{l}]:{m}{n}"));
        let log_config = match self.log_mode {
            LogRecordMode::Console => log4rs::Config::builder()
                .appender(
                    log4rs::config::Appender::builder().build(
                        "console_log",
                        Box::new(
                            ConsoleAppender::builder()
                                .encoder(pattern_encoder.clone())
                                .build(),
                        ),
                    ),
                )
                .build(
                    log4rs::config::Root::builder()
                        .appender("console_log")
                        .build(self.log_level),
                )?,
            LogRecordMode::File => {
                let config;
                if let Some(log_file_path) = self.log_file_path {
                    config = log4rs::Config::builder()
                        .appender(
                            log4rs::config::Appender::builder().build(
                                "file_log",
                                Box::new(
                                    FileAppender::builder()
                                        .encoder(pattern_encoder.clone())
                                        .append(false)
                                        .build(log_file_path)?,
                                ),
                            ),
                        )
                        .build(
                            log4rs::config::Root::builder()
                                .appender("file_log")
                                .build(self.log_level),
                        )?;
                } else {
                    return Err("没有设置日志文件路径".into());
                }
                config
            }
        };
        log4rs::init_config(log_config)?;
        Ok(())
    }
}
