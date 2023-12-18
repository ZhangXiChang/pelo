use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{canvas::Canvas, *},
};
use std::{
    fs,
    io::{stdout, Stdout},
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

pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    is_run: bool,
    menu_state: MenuState,
    main_menu_state: MainMenuState,
    main_menu_options_state: ListState,
    side_menu_options_state: ListState,
    kbn_points: Option<ImagePoints>,
}
impl App {
    pub fn new() -> Self {
        stdout().execute(EnterAlternateScreen).unwrap();
        enable_raw_mode().unwrap();
        let mut main_menu_options_state = ListState::default();
        main_menu_options_state.select(Some(0));
        let mut side_menu_options_state = ListState::default();
        side_menu_options_state.select(Some(0));
        let kbn_points = match image::open("./assets/texture/kbn.png") {
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
        Self {
            terminal: Terminal::new(CrosstermBackend::new(stdout())).unwrap(),
            is_run: true,
            menu_state: MenuState::MainMenu,
            main_menu_state: MainMenuState::RootMenu,
            main_menu_options_state,
            side_menu_options_state,
            kbn_points,
        }
    }
    pub fn draw(&mut self) {
        let main_menu_options = &match self.main_menu_state {
            MainMenuState::RootMenu => vec![
                ListItem::new(vec![Line::from("让我康康你的卡组")]),
                ListItem::new(vec![Line::from("退出牌佬助手")]),
            ],
            MainMenuState::DeckSelectMenu => vec![ListItem::new(vec![Line::from("返回")])],
        };
        let file_path_list = &match fs::read_dir("./assets/deck") {
            Ok(entrys) => {
                let mut path_vec = vec![];
                for entry in entrys {
                    let metadata = fs::metadata(entry.as_ref().unwrap().path()).unwrap();
                    if metadata.is_file() {
                        path_vec.push(entry.unwrap().file_name().to_str().unwrap().to_string());
                    }
                }
                path_vec
            }
            Err(_) => vec![],
        };
        let side_menu_options = &match self.main_menu_state {
            MainMenuState::DeckSelectMenu => {
                let mut file_path = vec![];
                for file_path_str in file_path_list {
                    file_path.push(ListItem::new(vec![Line::from(file_path_str.clone())]));
                }
                file_path
            }
            _ => vec![],
        };
        self.terminal
            .draw(|frame| {
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
                            List::new(main_menu_options.as_slice())
                                .block(Block::new().borders(Borders::ALL).title("主菜单"))
                                .highlight_style(Style::new().add_modifier(Modifier::BOLD))
                                .highlight_symbol(">> "),
                            content_layout[0],
                            &mut self.main_menu_options_state,
                        );
                        frame.render_stateful_widget(
                            List::new(side_menu_options.as_slice())
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
                                Paragraph::new("看板娘不在了！")
                                    .block(Block::new().borders(Borders::ALL)),
                                content_layout[2],
                            );
                        }
                    }
                }
            })
            .unwrap();
        if event::poll(std::time::Duration::from_millis(10)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Up => match self.menu_state {
                            MenuState::MainMenu => self.main_menu_options_state.select(Some(
                                (self.main_menu_options_state.selected().unwrap() as i64 - 1)
                                    .clamp(0, main_menu_options.len() as i64 - 1)
                                    as usize,
                            )),
                            MenuState::SideMenu => self.side_menu_options_state.select(Some(
                                (self.side_menu_options_state.selected().unwrap() as i64 - 1)
                                    .clamp(0, side_menu_options.len() as i64 - 1)
                                    as usize,
                            )),
                        },
                        KeyCode::Down => match self.menu_state {
                            MenuState::MainMenu => self.main_menu_options_state.select(Some(
                                (self.main_menu_options_state.selected().unwrap() as i64 + 1)
                                    .clamp(0, main_menu_options.len() as i64 - 1)
                                    as usize,
                            )),
                            MenuState::SideMenu => self.side_menu_options_state.select(Some(
                                (self.side_menu_options_state.selected().unwrap() as i64 + 1)
                                    .clamp(0, side_menu_options.len() as i64 - 1)
                                    as usize,
                            )),
                        },
                        KeyCode::Tab => match self.menu_state {
                            MenuState::MainMenu => self.menu_state = MenuState::SideMenu,
                            MenuState::SideMenu => self.menu_state = MenuState::MainMenu,
                        },
                        KeyCode::Enter => match self.menu_state {
                            MenuState::MainMenu => match self.main_menu_state {
                                MainMenuState::RootMenu => {
                                    match self.main_menu_options_state.selected().unwrap() {
                                        0 => {
                                            self.main_menu_state = MainMenuState::DeckSelectMenu;
                                            self.menu_state = MenuState::SideMenu;
                                        }
                                        1 => self.is_run = false,
                                        _ => (),
                                    }
                                }
                                MainMenuState::DeckSelectMenu => {
                                    match self.main_menu_options_state.selected().unwrap() {
                                        0 => self.main_menu_state = MainMenuState::RootMenu,
                                        _ => (),
                                    }
                                }
                            },
                            MenuState::SideMenu => {
                                let _file_path = &file_path_list
                                    [self.side_menu_options_state.selected().unwrap()];
                            }
                        },
                        _ => (),
                    }
                }
            }
        }
    }
    pub fn is_run(&mut self) -> bool {
        self.is_run
    }
}
impl Drop for App {
    fn drop(&mut self) {
        stdout().execute(LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
    }
}
