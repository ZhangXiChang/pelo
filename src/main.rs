use std::{fs, io};

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
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
    io::stdout().execute(EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout())).unwrap();
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
        Err(_) => None,
    };
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
                let mut dir_file_name = vec![];
                for entry in entrys {
                    let metadata = fs::metadata(entry.as_ref().unwrap().path()).unwrap();
                    if metadata.is_file() {
                        dir_file_name
                            .push(entry.unwrap().file_name().to_str().unwrap().to_string());
                    }
                }
                dir_file_name
            }
            Err(_) => vec![],
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
        input_process(&mut app);
        terminal
            .draw(|frame| {
                draw(frame, &mut app);
            })
            .unwrap();
    }
    io::stdout().execute(LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
}
fn input_process(app: &mut App) {
    if event::poll(std::time::Duration::from_millis(10)).unwrap() {
        if let event::Event::Key(key) = event::read().unwrap() {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Up => match app.menu_state {
                        MenuState::MainMenu => app.main_menu_options_state.select(Some(
                            (app.main_menu_options_state.selected().unwrap() as i64 - 1)
                                .clamp(0, app.main_menu_options.len() as i64 - 1)
                                as usize,
                        )),
                        MenuState::SideMenu => app.side_menu_options_state.select(Some(
                            (app.side_menu_options_state.selected().unwrap() as i64 - 1)
                                .clamp(0, app.side_menu_options.len() as i64 - 1)
                                as usize,
                        )),
                    },
                    KeyCode::Down => match app.menu_state {
                        MenuState::MainMenu => app.main_menu_options_state.select(Some(
                            (app.main_menu_options_state.selected().unwrap() as i64 + 1)
                                .clamp(0, app.main_menu_options.len() as i64 - 1)
                                as usize,
                        )),
                        MenuState::SideMenu => app.side_menu_options_state.select(Some(
                            (app.side_menu_options_state.selected().unwrap() as i64 + 1)
                                .clamp(0, app.side_menu_options.len() as i64 - 1)
                                as usize,
                        )),
                    },
                    KeyCode::Tab => match app.menu_state {
                        MenuState::MainMenu => app.menu_state = MenuState::SideMenu,
                        MenuState::SideMenu => app.menu_state = MenuState::MainMenu,
                    },
                    KeyCode::Enter => match app.menu_state {
                        MenuState::MainMenu => match app.main_menu_state {
                            MainMenuState::RootMenu => {
                                match app.main_menu_options_state.selected().unwrap() {
                                    0 => {
                                        app.main_menu_state = MainMenuState::DeckSelectMenu;
                                        app.menu_state = MenuState::SideMenu;
                                    }
                                    1 => app.is_run = false,
                                    _ => (),
                                }
                            }
                            MainMenuState::DeckSelectMenu => {
                                match app.main_menu_options_state.selected().unwrap() {
                                    0 => app.main_menu_state = MainMenuState::RootMenu,
                                    _ => (),
                                }
                            }
                        },
                        MenuState::SideMenu => {
                            let _file_name = &app.deck_dir_file_name[0];
                        }
                    },
                    _ => (),
                }
            }
        }
    }
}
fn draw(frame: &mut Frame, app: &mut App) {
    let main_layout = Layout::new()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.size());
    {
        frame.render_widget(
            Paragraph::new("牌佬助手")
                .block(Block::new().borders(Borders::ALL))
                .alignment(Alignment::Center)
                .add_modifier(Modifier::BOLD),
            main_layout[0],
        );
        let content_layout = Layout::new()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(21),
                Constraint::Min(0),
                Constraint::Max(main_layout[1].height * 2),
            ])
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
