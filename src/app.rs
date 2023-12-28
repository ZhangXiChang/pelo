use std::{fs, io, process::exit, time};

use crossterm::{
    event::{self, poll, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use log::{error, info, warn};
use ratatui::{
    prelude::*,
    widgets::{canvas::Canvas, *},
};

enum MenuState {
    Main,
    Side,
}
enum MainMenuState {
    Root,
    SelectDeck,
}
enum SideMenuState {
    Null,
    SelectDeckFromFile,
}

struct Points {
    width: f64,
    height: f64,
    points: Vec<(f64, f64)>,
}

pub struct App {
    is_run: bool,
    menu_state: MenuState,
    main_menu_state: MainMenuState,
    main_menu_items_state: ListState,
    main_menu_items_len: usize,
    side_menu_state: SideMenuState,
    side_menu_items_state: ListState,
    side_menu_items_len: usize,
    kbn_points: Option<Points>,
}
impl App {
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
            menu_state: MenuState::Main,
            main_menu_state: MainMenuState::Root,
            main_menu_items_state: ListState::default(),
            main_menu_items_len: 0,
            side_menu_state: SideMenuState::Null,
            side_menu_items_state: ListState::default(),
            side_menu_items_len: 0,
            kbn_points: None,
        }
    }
    pub fn run(&mut self) {
        match Terminal::new(CrosstermBackend::new(io::stdout())) {
            Ok(mut t) => {
                info!("实例化终端UI绘制对象成功");
                self.main_menu_items_state.select(Some(0));
                self.side_menu_items_state.select(Some(0));
                self.kbn_points = match image::open("./assets/texture/kbn.png") {
                    Ok(img) => {
                        let gimg = img.flipv().to_luma8();
                        let mut points = vec![];
                        for (x, y, pixel) in gimg.enumerate_pixels() {
                            if pixel[0] > 128 {
                                points.push((x as f64, y as f64));
                            }
                        }
                        Some(Points {
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
                    match t.draw(|frame| self.draw(frame)) {
                        Err(e) => {
                            error!("绘制终端UI失败，返回的错误信息：{}", e);
                            exit(-1);
                        }
                        _ => (),
                    }
                    match self.input_process() {
                        Err(e) => {
                            error!("输入处理出错，返回的错误信息：{}", e);
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
                //主菜单
                let mut main_menu_title: Span<'_> = "主菜单".into();
                match self.menu_state {
                    MenuState::Main => {
                        main_menu_title = main_menu_title.add_modifier(Modifier::REVERSED)
                    }
                    MenuState::Side => (),
                }
                let main_menu_items;
                match self.main_menu_state {
                    MainMenuState::Root => {
                        main_menu_items = vec!["让我康康你的卡组", "退出牌佬助手"]
                    }
                    MainMenuState::SelectDeck => main_menu_items = vec!["从文件读取卡组", "返回"],
                }
                self.main_menu_items_len = main_menu_items.len();
                frame.render_stateful_widget(
                    List::new(main_menu_items)
                        .block(Block::new().borders(Borders::ALL).title(main_menu_title))
                        .highlight_style(Style::new().add_modifier(Modifier::BOLD))
                        .highlight_symbol(">> "),
                    content_layout[0],
                    &mut self.main_menu_items_state,
                );
                //副菜单
                let mut side_menu_title: Span<'_> = "副菜单".into();
                match self.menu_state {
                    MenuState::Main => (),
                    MenuState::Side => {
                        side_menu_title = side_menu_title.add_modifier(Modifier::REVERSED)
                    }
                }
                let mut side_menu_items = Vec::<String>::new();
                match self.side_menu_state {
                    SideMenuState::Null => (),
                    SideMenuState::SelectDeckFromFile => {
                        side_menu_items = Self::query_dir_file_name_suffix(
                            "./assets/deck".to_string(),
                            ".ydk".to_string(),
                        );
                    }
                }
                self.side_menu_items_len = side_menu_items.len();
                frame.render_stateful_widget(
                    List::new(side_menu_items)
                        .block(Block::new().borders(Borders::ALL).title(side_menu_title))
                        .highlight_style(Style::new().add_modifier(Modifier::BOLD))
                        .highlight_symbol(">> "),
                    content_layout[1],
                    &mut self.side_menu_items_state,
                );
                //看板娘
                if let Some(points) = &self.kbn_points {
                    frame.render_widget(
                        Canvas::default()
                            .block(Block::new().borders(Borders::ALL))
                            .x_bounds([0.0, points.width])
                            .y_bounds([0.0, points.height])
                            .paint(|ctx| {
                                ctx.draw(&canvas::Points {
                                    coords: &points.points,
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
    fn input_process(&mut self) -> io::Result<()> {
        if poll(time::Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key) => match key.kind {
                    KeyEventKind::Press => match key.code {
                        KeyCode::Up => match self.menu_state {
                            MenuState::Main => {
                                Self::step_list_state(
                                    self.main_menu_items_len,
                                    &mut self.main_menu_items_state,
                                    -1,
                                );
                            }
                            MenuState::Side => {
                                Self::step_list_state(
                                    self.side_menu_items_len,
                                    &mut self.side_menu_items_state,
                                    -1,
                                );
                            }
                        },
                        KeyCode::Down => match self.menu_state {
                            MenuState::Main => {
                                Self::step_list_state(
                                    self.main_menu_items_len,
                                    &mut self.main_menu_items_state,
                                    1,
                                );
                            }
                            MenuState::Side => {
                                Self::step_list_state(
                                    self.side_menu_items_len,
                                    &mut self.side_menu_items_state,
                                    1,
                                );
                            }
                        },
                        KeyCode::Tab => match self.menu_state {
                            MenuState::Main => self.menu_state = MenuState::Side,
                            MenuState::Side => self.menu_state = MenuState::Main,
                        },
                        KeyCode::Enter => match self.menu_state {
                            MenuState::Main => match self.main_menu_state {
                                MainMenuState::Root => {
                                    if let Some(i) = self.main_menu_items_state.selected() {
                                        match i {
                                            0 => self.main_menu_state = MainMenuState::SelectDeck,
                                            1 => self.is_run = false,
                                            _ => (),
                                        }
                                    }
                                }
                                MainMenuState::SelectDeck => {
                                    if let Some(i) = self.main_menu_items_state.selected() {
                                        match i {
                                            0 => {
                                                self.side_menu_state =
                                                    SideMenuState::SelectDeckFromFile;
                                                self.menu_state = MenuState::Side;
                                            }
                                            1 => {
                                                self.side_menu_state = SideMenuState::Null;
                                                self.main_menu_state = MainMenuState::Root;
                                            }
                                            _ => (),
                                        }
                                    }
                                }
                            },
                            MenuState::Side => (),
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
    fn step_list_state(item_len: usize, list_state: &mut ListState, step_length: i64) {
        if let Some(i) = list_state.selected() {
            list_state.select(Some(
                (i as i64 + step_length).clamp(0, (item_len as i64 - 1).clamp(0, item_len as i64))
                    as usize,
            ));
        }
    }
    fn query_dir_file_name_suffix(path: String, file_name_suffix: String) -> Vec<String> {
        match fs::read_dir(path) {
            Ok(rd) => {
                let mut file_name_list = vec![];
                for entry_res in rd {
                    match entry_res {
                        Ok(de) => match fs::metadata(de.path()) {
                            Ok(m) => {
                                if m.is_file() {
                                    if let Some(file_name) = de.file_name().to_str() {
                                        if let Some(i) = file_name.rfind(file_name_suffix.as_str())
                                        {
                                            file_name_list.push(file_name[0..i].to_string());
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
        }
    }
}
impl Drop for App {
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
