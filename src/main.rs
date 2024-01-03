mod deck;
mod error;

use std::{fs, io, process::exit, time};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use deck::*;
use error::*;
use log::error;
use log4rs::{append::file::FileAppender, encode::pattern::PatternEncoder};
use ratatui::{prelude::*, widgets::*};

enum Focus {
    MainMenu,
    SideMenu,
}
enum MainMenuState {
    Root,
    SelectDeck,
}
enum SideMenuState {
    Null,
    SelectDeckFromFile,
}

struct Menu<'a, T> {
    state: T,
    title: Span<'a>,
    items_state: ListState,
    items: Vec<String>,
}
struct App<'a> {
    is_run: bool,
    title: Text<'a>,
    focus: Focus,
    main_menu: Menu<'a, MainMenuState>,
    side_menu: Menu<'a, SideMenuState>,
    deck: Option<Deck>,
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => {
            error!("{}", e);
            exit(-1);
        }
    }
}
fn run() -> Result<(), Error> {
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
                            .build("./logs/latest.log")?,
                    ),
                ),
            )
            .build(
                log4rs::config::Root::builder()
                    .appender("file_log")
                    .build(log::LevelFilter::Trace),
            )?,
    )?;
    //初始化应用程序
    let mut app = App {
        is_run: true,
        title: "牌佬助手".into(),
        focus: Focus::MainMenu,
        main_menu: Menu {
            state: MainMenuState::Root,
            title: "主菜单".into(),
            items_state: {
                let mut list_state = ListState::default();
                list_state.select(Some(0));
                list_state
            },
            items: vec![],
        },
        side_menu: Menu {
            state: SideMenuState::Null,
            title: "副菜单".into(),
            items_state: {
                let mut list_state = ListState::default();
                list_state.select(Some(0));
                list_state
            },
            items: vec![],
        },
        deck: None,
    };
    //初始化终端
    io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    //主循环
    while app.is_run {
        //输入处理
        if event::poll(time::Duration::from_millis(0))? {
            input_process(&mut app, event::read()?)?;
        }
        //状态机
        state_machine(&mut app)?;
        //终端绘制
        terminal.draw(|frame| terminal_gui(&mut app, frame))?;
    }
    io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
fn input_process(app: &mut App, event: Event) -> Result<(), Error> {
    match event {
        Event::Key(key) => match key.kind {
            KeyEventKind::Press => match key.code {
                KeyCode::Up => match app.focus {
                    Focus::MainMenu => {
                        step_list_state(
                            &mut app.main_menu.items_state,
                            app.main_menu.items.len(),
                            -1,
                        );
                    }
                    Focus::SideMenu => {
                        step_list_state(
                            &mut app.side_menu.items_state,
                            app.side_menu.items.len(),
                            -1,
                        );
                    }
                },
                KeyCode::Down => match app.focus {
                    Focus::MainMenu => {
                        step_list_state(
                            &mut app.main_menu.items_state,
                            app.main_menu.items.len(),
                            1,
                        );
                    }
                    Focus::SideMenu => {
                        step_list_state(
                            &mut app.side_menu.items_state,
                            app.side_menu.items.len(),
                            1,
                        );
                    }
                },
                KeyCode::Tab => match app.focus {
                    Focus::MainMenu => app.focus = Focus::SideMenu,
                    Focus::SideMenu => app.focus = Focus::MainMenu,
                },
                KeyCode::Enter => match app.focus {
                    Focus::MainMenu => match app.main_menu.state {
                        MainMenuState::Root => {
                            if let Some(i) = app.main_menu.items_state.selected() {
                                match i {
                                    0 => app.main_menu.state = MainMenuState::SelectDeck,
                                    1 => app.is_run = false,
                                    _ => (),
                                }
                            }
                        }
                        MainMenuState::SelectDeck => {
                            if let Some(i) = app.main_menu.items_state.selected() {
                                match i {
                                    0 => {
                                        app.side_menu.state = SideMenuState::SelectDeckFromFile;
                                        app.focus = Focus::SideMenu;
                                    }
                                    1 => {
                                        app.main_menu.state = MainMenuState::Root;
                                        app.side_menu.state = SideMenuState::Null;
                                    }
                                    _ => (),
                                }
                            }
                        }
                    },
                    Focus::SideMenu => match app.side_menu.state {
                        SideMenuState::Null => (),
                        SideMenuState::SelectDeckFromFile => {
                            if let Some(i) = app.side_menu.items_state.selected() {
                                app.deck = Some(
                                    Deck::from(
                                        app.side_menu.items[i],
                                        DeckFromType::File(format!(
                                            "./assets/deck/{}.ydk",
                                            app.side_menu.items[i]
                                        )),
                                    )
                                    .await?,
                                )
                            }
                        }
                    },
                },
                _ => (),
            },
            _ => (),
        },
        _ => (),
    }
    Ok(())
}
fn state_machine(app: &mut App) -> Result<(), Error> {
    match app.focus {
        Focus::MainMenu => {
            app.main_menu.title = app.main_menu.title.clone().add_modifier(Modifier::REVERSED);
            app.side_menu.title = app
                .side_menu
                .title
                .clone()
                .remove_modifier(Modifier::REVERSED);
        }
        Focus::SideMenu => {
            app.main_menu.title = app
                .main_menu
                .title
                .clone()
                .remove_modifier(Modifier::REVERSED);
            app.side_menu.title = app.side_menu.title.clone().add_modifier(Modifier::REVERSED);
        }
    }
    match app.main_menu.state {
        MainMenuState::Root => {
            app.main_menu.items = vec!["让我康康你的卡组".to_string(), "退出牌佬助手".to_string()]
        }
        MainMenuState::SelectDeck => {
            app.main_menu.items = vec!["从文件读取卡组".to_string(), "返回".to_string()]
        }
    }
    match app.side_menu.state {
        SideMenuState::Null => app.side_menu.items = vec![],
        SideMenuState::SelectDeckFromFile => {
            app.side_menu.items =
                query_dir_file_name_suffix("./assets/deck".to_string(), ".ydk".to_string())?;
        }
    }
    Ok(())
}
fn terminal_gui(app: &mut App, frame: &mut Frame) {
    //根布局
    let root_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(3), Constraint::Min(0)],
    )
    .split(frame.size());
    {
        //应用标题
        frame.render_widget(
            Paragraph::new(app.title.clone())
                .block(Block::new().borders(Borders::ALL))
                .alignment(Alignment::Center)
                .add_modifier(Modifier::BOLD),
            root_layout[0],
        );
        //内容布局
        let content_layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Length(21), Constraint::Min(0)],
        )
        .split(root_layout[1]);
        {
            //主菜单
            frame.render_stateful_widget(
                List::new(app.main_menu.items.clone())
                    .block(
                        Block::new()
                            .borders(Borders::ALL)
                            .title(app.main_menu.title.clone()),
                    )
                    .highlight_style(Style::new().add_modifier(Modifier::BOLD))
                    .highlight_symbol(">> "),
                content_layout[0],
                &mut app.main_menu.items_state,
            );
            //副菜单
            frame.render_stateful_widget(
                List::new(app.side_menu.items.clone())
                    .block(
                        Block::new()
                            .borders(Borders::ALL)
                            .title(app.side_menu.title.clone()),
                    )
                    .highlight_style(Style::new().add_modifier(Modifier::BOLD))
                    .highlight_symbol(">> "),
                content_layout[1],
                &mut app.side_menu.items_state,
            );
        }
    }
}
fn step_list_state(list_state: &mut ListState, item_len: usize, step_length: i64) {
    if let Some(i) = list_state.selected() {
        list_state.select(Some(
            (i as i64 + step_length).clamp(0, (item_len as i64 - 1).clamp(0, item_len as i64))
                as usize,
        ));
    } else {
        list_state.select(Some(0));
    }
}
fn query_dir_file_name_suffix(
    path: String,
    file_name_suffix: String,
) -> Result<Vec<String>, Error> {
    let mut file_name_list = vec![];
    for dir_entry_result in fs::read_dir(path)? {
        let dir_entry = dir_entry_result?;
        if fs::metadata(dir_entry.path())?.is_file() {
            if let Some(file_name) = dir_entry.file_name().to_str() {
                if let Some(i) = file_name.rfind(file_name_suffix.as_str()) {
                    file_name_list.push(file_name[0..i].to_string());
                }
            }
        }
    }
    Ok(file_name_list)
}
