use std::{fs, io, time};

use crossterm::{
    event::{self, poll, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use log::{error, info, LevelFilter};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
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

struct ImagePoints {
    width: f64,
    height: f64,
    points: Vec<(f64, f64)>,
}
struct App<'a> {
    pub is_run: bool,
    pub menu_state: MenuState,
    pub main_menu_options_state: ListState,
    pub main_menu_options: Vec<ListItem<'a>>,
    pub main_menu_state: MainMenuState,
    pub side_menu_options_state: ListState,
    pub side_menu_options: Vec<ListItem<'a>>,
    pub kbn_points: Option<ImagePoints>,
    pub deck_dir_file_name: Vec<String>,
}

impl<'a> App<'a> {
    fn new() -> Self {
        Self {
            is_run: true,
            menu_state: MenuState::MainMenu,
            main_menu_options_state: ListState::default(),
            main_menu_options: vec![],
            main_menu_state: MainMenuState::RootMenu,
            side_menu_options_state: ListState::default(),
            side_menu_options: vec![],
            kbn_points: None,
            deck_dir_file_name: vec![],
        }
    }
}

fn main() {
    match FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%d %H:%M:%S)}][{l}]:{m}{n}",
        )))
        .build("./logs/latest.log")
    {
        Ok(fa) => {
            match Config::builder()
                .appender(Appender::builder().build("file", Box::new(fa)))
                .build(Root::builder().appender("file").build(LevelFilter::Info))
            {
                Ok(c) => match log4rs::init_config(c) {
                    Ok(_) => println!("初始化日志系统成功"),
                    Err(e) => {
                        println!("{}", e);
                        return;
                    }
                },
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            }
        }
        Err(e) => {
            println!("{}", e);
            return;
        }
    }

    match io::stdout().execute(EnterAlternateScreen) {
        Ok(_) => info!("执行切换到终端备用屏幕的命令成功"),
        Err(e) => {
            error!("执行切换到终端备用屏幕的命令失败，返回的错误信息：{}", e);
            return;
        }
    }
    match enable_raw_mode() {
        Ok(_) => info!("开启终端原始模式成功"),
        Err(e) => {
            error!("开启终端原始模式失败，返回的错误信息：{}", e);
            return;
        }
    }

    let mut app = App::new();
    app.main_menu_options_state.select(Some(0));
    app.side_menu_options_state.select(Some(0));
    app.kbn_points = match image::open("./assets/texture/kbn.png") {
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
        _ => None,
    };

    match Terminal::new(CrosstermBackend::new(io::stdout())) {
        Ok(mut t) => {
            info!("实例化终端UI绘制对象成功");
            while app.is_run {
                //主菜单
                app.main_menu_options = match app.main_menu_state {
                    MainMenuState::RootMenu => vec![
                        ListItem::new("让我康康你的卡组"),
                        ListItem::new("退出牌佬助手"),
                    ],
                    MainMenuState::DeckSelectMenu => vec![ListItem::new("返回")],
                };
                //副菜单
                app.deck_dir_file_name = match fs::read_dir("./assets/deck") {
                    Ok(entrys) => {
                        let mut file_name_list = vec![];
                        for entry_res in entrys {
                            match entry_res {
                                Ok(de) => match fs::metadata(de.path()) {
                                    Ok(m) => {
                                        if m.is_file() {
                                            if let Some(file_name) = de.file_name().to_str() {
                                                file_name_list.push(file_name.to_string());
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("读取元数据失败，返回的错误信息：{}", e);
                                        app.is_run = false;
                                    }
                                },
                                Err(e) => {
                                    error!("读取目录条目失败，返回的错误信息：{}", e);
                                    app.is_run = false;
                                }
                            }
                        }
                        file_name_list
                    }
                    _ => vec![],
                };
                app.side_menu_options = match app.main_menu_state {
                    MainMenuState::DeckSelectMenu => {
                        let mut options = vec![];
                        for file_name in app.deck_dir_file_name.clone() {
                            options.push(ListItem::new(file_name));
                        }
                        options
                    }
                    _ => vec![],
                };
                match input_process(&mut app) {
                    Err(e) => {
                        error!("输入处理出错，返回的错误信息：{}", e);
                        app.is_run = false;
                    }
                    _ => (),
                }
                match t.draw(|frame| {
                    draw(frame, &mut app);
                }) {
                    Err(e) => {
                        error!("绘制终端UI失败，返回的错误信息：{}", e);
                        app.is_run = false;
                    }
                    _ => (),
                }
            }
        }
        Err(e) => {
            error!("实例化终端UI绘制对象失败，返回的错误信息：{}", e);
            return;
        }
    }
    match io::stdout().execute(LeaveAlternateScreen) {
        Ok(_) => info!("执行切换回终端主屏幕的命令成功"),
        Err(e) => {
            error!("执行切换回终端主屏幕的命令失败，返回的错误信息：{}", e);
            return;
        }
    }
    match disable_raw_mode() {
        Ok(_) => info!("关闭终端原始模式成功"),
        Err(e) => {
            error!("关闭终端原始模式失败，返回的错误信息：{}", e);
            return;
        }
    }
}
fn input_process(app: &mut App) -> io::Result<()> {
    if poll(time::Duration::from_millis(0))? {
        match event::read()? {
            Event::Key(key) => match key.kind {
                KeyEventKind::Press => match key.code {
                    KeyCode::Up => match app.menu_state {
                        MenuState::MainMenu => {
                            if let Some(i) = app.main_menu_options_state.selected() {
                                app.main_menu_options_state.select(Some(
                                    (i as i64 - 1).clamp(0, app.main_menu_options.len() as i64 - 1)
                                        as usize,
                                ))
                            }
                        }
                        MenuState::SideMenu => {
                            if let Some(i) = app.side_menu_options_state.selected() {
                                app.side_menu_options_state.select(Some(
                                    (i as i64 - 1).clamp(0, app.main_menu_options.len() as i64 - 1)
                                        as usize,
                                ))
                            }
                        }
                    },
                    KeyCode::Down => match app.menu_state {
                        MenuState::MainMenu => {
                            if let Some(i) = app.main_menu_options_state.selected() {
                                app.main_menu_options_state.select(Some(
                                    (i as i64 + 1).clamp(0, app.main_menu_options.len() as i64 - 1)
                                        as usize,
                                ))
                            }
                        }
                        MenuState::SideMenu => {
                            if let Some(i) = app.side_menu_options_state.selected() {
                                app.side_menu_options_state.select(Some(
                                    (i as i64 + 1).clamp(0, app.main_menu_options.len() as i64 - 1)
                                        as usize,
                                ))
                            }
                        }
                    },
                    KeyCode::Tab => match app.menu_state {
                        MenuState::MainMenu => app.menu_state = MenuState::SideMenu,
                        MenuState::SideMenu => app.menu_state = MenuState::MainMenu,
                    },
                    KeyCode::Enter => match app.menu_state {
                        MenuState::MainMenu => match app.main_menu_state {
                            MainMenuState::RootMenu => {
                                if let Some(i) = app.main_menu_options_state.selected() {
                                    match i {
                                        0 => {
                                            app.main_menu_state = MainMenuState::DeckSelectMenu;
                                            app.menu_state = MenuState::SideMenu;
                                        }
                                        1 => app.is_run = false,
                                        _ => (),
                                    }
                                }
                            }
                            MainMenuState::DeckSelectMenu => {
                                if let Some(i) = app.main_menu_options_state.selected() {
                                    match i {
                                        0 => app.main_menu_state = MainMenuState::RootMenu,
                                        _ => (),
                                    }
                                }
                            }
                        },
                        MenuState::SideMenu => {
                            let _file_name = &app.deck_dir_file_name[0];
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
fn draw(frame: &mut Frame, app: &mut App) {
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
                List::new(app.main_menu_options.clone())
                    .block(Block::new().borders(Borders::ALL).title("主菜单"))
                    .highlight_style(Style::new().add_modifier(Modifier::BOLD))
                    .highlight_symbol(">> "),
                content_layout[0],
                &mut app.main_menu_options_state,
            );
            frame.render_stateful_widget(
                List::new(app.side_menu_options.clone())
                    .block(Block::new().borders(Borders::ALL).title("副菜单"))
                    .highlight_style(Style::new().add_modifier(Modifier::BOLD))
                    .highlight_symbol(">> "),
                content_layout[1],
                &mut app.side_menu_options_state,
            );
            if let Some(image_points) = &app.kbn_points {
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
