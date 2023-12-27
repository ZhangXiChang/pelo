use std::{fs, io, process::exit, time};

use crossterm::{
    event::{self, poll, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use log::{error, info, warn, LevelFilter};
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};
use ratatui::{
    prelude::*,
    widgets::{canvas::Canvas, *},
};

enum MenuState {
    MainMenu,
    SideMenu,
}
enum MainMenuState {
    RootMenu,
    DeckSelectMenu,
}
enum LogMode {
    Console,
    File,
    All,
}

struct ImagePoints {
    width: f64,
    height: f64,
    points: Vec<(f64, f64)>,
}

pub struct Log {
    log_mode: LogMode,
    log_file_path: Option<String>,
    log_level: LevelFilter,
}
impl Log {
    pub fn new() -> Self {
        let _ = LogMode::Console;
        let _ = LogMode::File;
        let _ = LogMode::All;
        Self {
            log_mode: LogMode::File,
            log_file_path: Some("./logs/latest.log".to_string()),
            log_level: LevelFilter::Info,
        }
    }
    pub fn enable(&mut self) {
        let log_config;
        let pattern_encoder = Box::new(PatternEncoder::new("[{d(%Y-%m-%d %H:%M:%S)}][{l}]:{m}{n}"));
        match self.log_mode {
            LogMode::Console => {
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
            LogMode::File => {
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
            LogMode::All => {
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

pub struct App<'a> {
    is_run: bool,
    menu_state: MenuState,
    main_menu_options_state: ListState,
    main_menu_options: Vec<ListItem<'a>>,
    main_menu_state: MainMenuState,
    side_menu_options_state: ListState,
    side_menu_options: Vec<ListItem<'a>>,
    kbn_points: Option<ImagePoints>,
    deck_file_name_list: Vec<String>,
}
impl<'a> App<'a> {
    pub fn new() -> Self {
        match io::stdout().execute(EnterAlternateScreen) {
            Ok(_) => info!("执行切换到终端备用屏幕的命令成功"),
            Err(e) => {
                error!("执行切换到终端备用屏幕的命令失败，返回的错误信息：{}", e);
                exit(-1);
            }
        }
        match enable_raw_mode() {
            Ok(_) => info!("开启终端原始模式成功"),
            Err(e) => {
                error!("开启终端原始模式失败，返回的错误信息：{}", e);
                exit(-1);
            }
        }
        Self {
            is_run: true,
            menu_state: MenuState::MainMenu,
            main_menu_options_state: ListState::default(),
            main_menu_options: vec![],
            main_menu_state: MainMenuState::RootMenu,
            side_menu_options_state: ListState::default(),
            side_menu_options: vec![],
            kbn_points: None,
            deck_file_name_list: vec![],
        }
    }
    pub fn run(&mut self) {
        match Terminal::new(CrosstermBackend::new(io::stdout())) {
            Ok(mut t) => {
                info!("实例化终端UI绘制对象成功");
                self.main_menu_options_state.select(Some(0));
                self.side_menu_options_state.select(Some(0));
                self.kbn_points = match image::open("./assets/texture/kbn.png") {
                    Ok(img) => {
                        let gimg = img.flipv().to_luma8();
                        let mut points = vec![];
                        for (x, y, pixel) in gimg.enumerate_pixels() {
                            if pixel[0] > 128 {
                                points.push((x as f64, y as f64));
                            }
                        }
                        Some(ImagePoints {
                            width: gimg.width() as f64,
                            height: gimg.height() as f64,
                            points,
                        })
                    }
                    Err(e) => {
                        warn!("没有找到看板娘文件，返回的错误信息：{}", e);
                        None
                    }
                };
                while self.is_run {
                    self.main_menu_options = match self.main_menu_state {
                        MainMenuState::RootMenu => vec![
                            ListItem::new("让我康康你的卡组"),
                            ListItem::new("退出牌佬助手"),
                        ],
                        MainMenuState::DeckSelectMenu => vec![ListItem::new("返回")],
                    };
                    self.deck_file_name_list = match fs::read_dir("./assets/deck") {
                        Ok(entrys) => {
                            let mut file_name_list = vec![];
                            for entry_res in entrys {
                                match entry_res {
                                    Ok(de) => match fs::metadata(de.path()) {
                                        Ok(m) => {
                                            if m.is_file() {
                                                if let Some(file_name) = de.file_name().to_str() {
                                                    if let Some(i) = file_name.rfind(".ydk") {
                                                        file_name_list
                                                            .push(file_name[0..i].to_string());
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            warn!("读取目录条目元数据失败，返回的错误信息：{}", e);
                                        }
                                    },
                                    Err(e) => {
                                        warn!("读取目录条目失败，返回的错误信息：{}", e);
                                    }
                                }
                            }
                            file_name_list
                        }
                        Err(e) => {
                            warn!("读取目录失败，返回的错误信息：{}", e);
                            vec![]
                        }
                    };
                    self.side_menu_options = match self.main_menu_state {
                        MainMenuState::DeckSelectMenu => {
                            let mut options = vec![];
                            for file_name in self.deck_file_name_list.clone() {
                                options.push(ListItem::new(file_name));
                            }
                            options
                        }
                        _ => vec![],
                    };
                    match self.input_process() {
                        Err(e) => {
                            error!("输入处理出错，返回的错误信息：{}", e);
                            exit(-1);
                        }
                        _ => (),
                    }
                    match t.draw(|frame| self.draw(frame)) {
                        Err(e) => {
                            error!("绘制终端UI失败，返回的错误信息：{}", e);
                            exit(-1);
                        }
                        _ => (),
                    }
                }
            }
            Err(e) => {
                error!("实例化终端UI绘制对象失败，返回的错误信息：{}", e);
                exit(-1);
            }
        }
    }
    fn input_process(&mut self) -> io::Result<()> {
        if poll(time::Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key) => match key.kind {
                    KeyEventKind::Press => match key.code {
                        KeyCode::Up => match self.menu_state {
                            MenuState::MainMenu => {
                                Self::step_list_state(
                                    &self.main_menu_options,
                                    &mut self.main_menu_options_state,
                                    -1,
                                );
                            }
                            MenuState::SideMenu => {
                                Self::step_list_state(
                                    &self.side_menu_options,
                                    &mut self.side_menu_options_state,
                                    -1,
                                );
                            }
                        },
                        KeyCode::Down => match self.menu_state {
                            MenuState::MainMenu => {
                                Self::step_list_state(
                                    &self.main_menu_options,
                                    &mut self.main_menu_options_state,
                                    1,
                                );
                            }
                            MenuState::SideMenu => {
                                Self::step_list_state(
                                    &self.side_menu_options,
                                    &mut self.side_menu_options_state,
                                    1,
                                );
                            }
                        },
                        KeyCode::Tab => match self.menu_state {
                            MenuState::MainMenu => {
                                self.menu_state = MenuState::SideMenu;
                            }
                            MenuState::SideMenu => {
                                self.menu_state = MenuState::MainMenu;
                            }
                        },
                        KeyCode::Enter => match self.menu_state {
                            MenuState::MainMenu => match self.main_menu_state {
                                MainMenuState::RootMenu => {
                                    if let Some(i) = self.main_menu_options_state.selected() {
                                        match i {
                                            0 => {
                                                self.main_menu_state =
                                                    MainMenuState::DeckSelectMenu;
                                                self.menu_state = MenuState::SideMenu;
                                            }
                                            1 => {
                                                self.is_run = false;
                                            }
                                            _ => (),
                                        }
                                    }
                                }
                                MainMenuState::DeckSelectMenu => {
                                    if let Some(i) = self.main_menu_options_state.selected() {
                                        match i {
                                            0 => {
                                                self.main_menu_state = MainMenuState::RootMenu;
                                            }
                                            _ => (),
                                        }
                                    }
                                }
                            },
                            MenuState::SideMenu => {
                                let _ = self.deck_file_name_list
                                    [self.side_menu_options_state.selected().unwrap()];
                            }
                        },
                        _ => (),
                    },
                    _ => (),
                },
                _ => (),
            }
        }
        Ok(())
    }
    fn draw(&mut self, frame: &mut Frame) {
        let main_layout = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Min(0)],
        )
        .split(frame.size());
        {
            frame.render_widget(
                Paragraph::new("牌佬助手")
                    .block(Block::new().borders(Borders::ALL))
                    .alignment(Alignment::Center)
                    .add_modifier(Modifier::BOLD),
                main_layout[0],
            );
            let content_layout = Layout::new(
                Direction::Horizontal,
                [
                    Constraint::Length(21),
                    Constraint::Min(0),
                    Constraint::Max(main_layout[1].height * 2),
                ],
            )
            .split(main_layout[1]);
            {
                frame.render_stateful_widget(
                    List::new(self.main_menu_options.clone())
                        .block(Block::new().borders(Borders::ALL).title("主菜单"))
                        .highlight_style(Style::new().add_modifier(Modifier::BOLD))
                        .highlight_symbol(">> "),
                    content_layout[0],
                    &mut self.main_menu_options_state,
                );
                frame.render_stateful_widget(
                    List::new(self.side_menu_options.clone())
                        .block(Block::new().borders(Borders::ALL).title("副菜单"))
                        .highlight_style(Style::new().add_modifier(Modifier::BOLD))
                        .highlight_symbol(">> "),
                    content_layout[1],
                    &mut self.side_menu_options_state,
                );
                if let Some(image_points) = &self.kbn_points {
                    frame.render_widget(
                        Canvas::default()
                            .block(Block::new().borders(Borders::ALL))
                            .x_bounds([0.0, image_points.width])
                            .y_bounds([0.0, image_points.height])
                            .paint(|ctx| {
                                ctx.draw(&canvas::Points {
                                    coords: &image_points.points,
                                    ..Default::default()
                                })
                            }),
                        content_layout[2],
                    );
                } else {
                    frame.render_widget(
                        Paragraph::new("看板娘不在了！").block(Block::new().borders(Borders::ALL)),
                        content_layout[2],
                    );
                }
            }
        }
    }
    fn step_list_state(
        item_list: &Vec<ListItem<'a>>,
        list_state: &mut ListState,
        step_length: i64,
    ) {
        if let Some(i) = list_state.selected() {
            list_state.select(Some((i as i64 + step_length).clamp(
                0,
                (item_list.len() as i64 - 1).clamp(0, item_list.len() as i64),
            ) as usize));
        }
    }
}
impl<'a> Drop for App<'a> {
    fn drop(&mut self) {
        match io::stdout().execute(LeaveAlternateScreen) {
            Ok(_) => info!("执行切换回终端主屏幕的命令成功"),
            Err(e) => {
                error!("执行切换回终端主屏幕的命令失败，返回的错误信息：{}", e);
                exit(-1);
            }
        }
        match disable_raw_mode() {
            Ok(_) => info!("关闭终端原始模式成功"),
            Err(e) => {
                error!("关闭终端原始模式失败，返回的错误信息：{}", e);
                exit(-1);
            }
        }
    }
}
