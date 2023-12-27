use log::info;
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};

#[allow(dead_code)]
enum LogRecordMode {
    Console,
    File,
    All,
}

pub struct LogRecord {
    log_mode: LogRecordMode,
    log_file_path: Option<String>,
    log_level: log::LevelFilter,
}
impl LogRecord {
    pub fn new() -> Self {
        Self {
            log_mode: LogRecordMode::File,
            log_file_path: Some("./logs/latest.log".to_string()),
            log_level: log::LevelFilter::Info,
        }
    }
    pub fn enable(&mut self) {
        let log_config;
        let pattern_encoder = Box::new(PatternEncoder::new("[{d(%Y-%m-%d %H:%M:%S)}][{l}]:{m}{n}"));
        match self.log_mode {
            LogRecordMode::Console => {
                match log4rs::Config::builder()
                    .appender(
                        Appender::builder().build(
                            "console_log",
                            Box::new(
                                ConsoleAppender::builder()
                                    .encoder(pattern_encoder.clone())
                                    .build(),
                            ),
                        ),
                    )
                    .build(
                        Root::builder()
                            .appender("console_log")
                            .build(self.log_level),
                    ) {
                    Ok(c) => log_config = c,
                    Err(e) => {
                        panic!("日志配置构建失败，返回的错误信息：{}", e);
                    }
                }
            }
            LogRecordMode::File => {
                if let Some(log_file_path) = &self.log_file_path {
                    match FileAppender::builder()
                        .encoder(pattern_encoder.clone())
                        .append(false)
                        .build(log_file_path)
                    {
                        Ok(fa) => {
                            match log4rs::Config::builder()
                                .appender(Appender::builder().build("file_log", Box::new(fa)))
                                .build(Root::builder().appender("file_log").build(self.log_level))
                            {
                                Ok(c) => log_config = c,
                                Err(e) => {
                                    panic!("日志配置构建失败，返回的错误信息：{}", e);
                                }
                            }
                        }
                        Err(e) => {
                            panic!("日志输出器构建失败，返回的错误信息：{}", e);
                        }
                    }
                } else {
                    panic!("没有设置日志文件路径");
                }
            }
            LogRecordMode::All => {
                if let Some(log_file_path) = &self.log_file_path {
                    match FileAppender::builder()
                        .encoder(pattern_encoder.clone())
                        .append(false)
                        .build(log_file_path)
                    {
                        Ok(fa) => {
                            match log4rs::Config::builder()
                                .appender(
                                    Appender::builder().build(
                                        "console_log",
                                        Box::new(
                                            ConsoleAppender::builder()
                                                .encoder(pattern_encoder.clone())
                                                .build(),
                                        ),
                                    ),
                                )
                                .appender(Appender::builder().build("file_log", Box::new(fa)))
                                .build(
                                    Root::builder()
                                        .appender("console_log")
                                        .appender("file_log")
                                        .build(self.log_level),
                                ) {
                                Ok(c) => log_config = c,
                                Err(e) => {
                                    panic!("日志配置构建失败，返回的错误信息：{}", e);
                                }
                            }
                        }
                        Err(e) => {
                            panic!("日志输出器构建失败，返回的错误信息：{}", e);
                        }
                    }
                } else {
                    panic!("没有设置日志文件路径");
                }
            }
        }
        match log4rs::init_config(log_config) {
            Ok(_) => info!("初始化日志系统成功"),
            Err(e) => {
                panic!("日志初始化失败，返回的错误信息：{}", e);
            }
        }
    }
}
