use std::{fs, io, time};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{canvas::Canvas, *},
};

use crate::{deck::*, error::Error};

enum PopupState {
    Show,
    Hide,
}
enum Focus {
    MainMenu,
    SideMenu,
    Popup,
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

struct Popup<'a> {
    state: PopupState,
    title: Span<'a>,
    message: String,
}
impl<'a> Popup<'a> {
    fn new(state: PopupState) -> Self {
        Self {
            state,
            title: Default::default(),
            message: Default::default(),
        }
    }
}

struct Menu<'a, T> {
    state: T,
    title: Span<'a>,
    items_state: ListState,
    items: Vec<String>,
}
impl<'a, T> Menu<'a, T> {
    fn new(state: T) -> Self {
        Self {
            state,
            title: Default::default(),
            items_state: Default::default(),
            items: Default::default(),
        }
    }
}

pub struct App<'a> {
    is_run: bool,
    focus: Focus,
    popup: Popup<'a>,
    main_menu: Menu<'a, MainMenuState>,
    side_menu: Menu<'a, SideMenuState>,
    kbn_points: Option<Points>,
    deck: Option<Deck>,
}
impl<'a> App<'a> {
    fn step_list_state(item_len: usize, list_state: &mut ListState, step_length: i64) {
        if let Some(i) = list_state.selected() {
            list_state.select(Some(
                (i as i64 + step_length).clamp(0, (item_len as i64 - 1).clamp(0, item_len as i64))
                    as usize,
            ));
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
    pub fn new() -> Self {
        Self {
            is_run: true,
            focus: Focus::MainMenu,
            popup: Popup::new(PopupState::Hide),
            main_menu: Menu::new(MainMenuState::Root),
            side_menu: Menu::new(SideMenuState::Null),
            kbn_points: Default::default(),
            deck: Default::default(),
        }
    }
    pub async fn run(mut self) -> Result<(), Error> {
        self.main_menu.items_state.select(Some(0));
        self.side_menu.items_state.select(Some(0));
        let gimg = image::open("./assets/texture/kbn.png")?.flipv().to_luma8();
        let mut points = vec![];
        for (x, y, pixel) in gimg.enumerate_pixels() {
            if pixel[0] > 128 {
                points.push((x as f64, y as f64));
            }
        }
        self.kbn_points = Some(Points {
            width: gimg.width() as f64,
            height: gimg.height() as f64,
            points,
        });
        io::stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut tui = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        while self.is_run {
            let mut draw_result = Ok(());
            tui.draw(|frame| {
                draw_result = self.draw(frame);
            })?;
            match draw_result {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
            if event::poll(time::Duration::from_millis(0))? {
                self.input_process(event::read()?).await?;
            }
        }
        io::stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }
    fn draw(&mut self, frame: &mut Frame) -> Result<(), Error> {
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
                self.main_menu.title = Span::from("主菜单");
                match self.focus {
                    Focus::MainMenu => {
                        self.main_menu.title = self
                            .main_menu
                            .title
                            .clone()
                            .add_modifier(Modifier::REVERSED)
                    }
                    _ => (),
                }
                match self.main_menu.state {
                    MainMenuState::Root => {
                        self.main_menu.items =
                            vec!["让我康康你的卡组".to_string(), "退出牌佬助手".to_string()]
                    }
                    MainMenuState::SelectDeck => {
                        self.main_menu.items =
                            vec!["从文件读取卡组".to_string(), "返回".to_string()]
                    }
                }
                frame.render_stateful_widget(
                    List::new(self.main_menu.items.clone())
                        .block(
                            Block::new()
                                .borders(Borders::ALL)
                                .title(self.main_menu.title.clone()),
                        )
                        .highlight_style(Style::new().add_modifier(Modifier::BOLD))
                        .highlight_symbol(">> "),
                    content_layout[0],
                    &mut self.main_menu.items_state,
                );
                //副菜单
                self.side_menu.title = Span::from("副菜单");
                match self.focus {
                    Focus::SideMenu => {
                        self.side_menu.title = self
                            .side_menu
                            .title
                            .clone()
                            .add_modifier(Modifier::REVERSED)
                    }
                    _ => (),
                }
                match self.side_menu.state {
                    SideMenuState::Null => self.side_menu.items = vec![],
                    SideMenuState::SelectDeckFromFile => {
                        self.side_menu.items = Self::query_dir_file_name_suffix(
                            "./assets/deck".to_string(),
                            ".ydk".to_string(),
                        )?;
                    }
                }
                frame.render_stateful_widget(
                    List::new(self.side_menu.items.clone())
                        .block(
                            Block::new()
                                .borders(Borders::ALL)
                                .title(self.side_menu.title.clone()),
                        )
                        .highlight_style(Style::new().add_modifier(Modifier::BOLD))
                        .highlight_symbol(">> "),
                    content_layout[1],
                    &mut self.side_menu.items_state,
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
        match self.popup.state {
            PopupState::Show => frame.render_widget(
                Paragraph::new(self.popup.message.clone()).block(
                    Block::new()
                        .borders(Borders::ALL)
                        .title(self.popup.title.clone()),
                ),
                frame.size(),
            ),
            PopupState::Hide => (),
        }
        Ok(())
    }
    async fn input_process(&mut self, event: Event) -> Result<(), Error> {
        match event {
            Event::Key(key) => match key.kind {
                KeyEventKind::Press => match key.code {
                    KeyCode::Up => match self.focus {
                        Focus::MainMenu => {
                            Self::step_list_state(
                                self.main_menu.items.len(),
                                &mut self.main_menu.items_state,
                                -1,
                            );
                        }
                        Focus::SideMenu => {
                            Self::step_list_state(
                                self.side_menu.items.len(),
                                &mut self.side_menu.items_state,
                                -1,
                            );
                        }
                        _ => (),
                    },
                    KeyCode::Down => match self.focus {
                        Focus::MainMenu => {
                            Self::step_list_state(
                                self.main_menu.items.len(),
                                &mut self.main_menu.items_state,
                                1,
                            );
                        }
                        Focus::SideMenu => {
                            Self::step_list_state(
                                self.side_menu.items.len(),
                                &mut self.side_menu.items_state,
                                1,
                            );
                        }
                        _ => (),
                    },
                    KeyCode::Tab => match self.focus {
                        Focus::MainMenu => self.focus = Focus::SideMenu,
                        Focus::SideMenu => self.focus = Focus::MainMenu,
                        _ => (),
                    },
                    KeyCode::Enter => match self.focus {
                        Focus::MainMenu => match self.main_menu.state {
                            MainMenuState::Root => {
                                if let Some(i) = self.main_menu.items_state.selected() {
                                    match i {
                                        0 => self.main_menu.state = MainMenuState::SelectDeck,
                                        1 => self.is_run = false,
                                        _ => (),
                                    }
                                }
                            }
                            MainMenuState::SelectDeck => {
                                if let Some(i) = self.main_menu.items_state.selected() {
                                    match i {
                                        0 => {
                                            self.side_menu.state =
                                                SideMenuState::SelectDeckFromFile;
                                            self.focus = Focus::SideMenu;
                                        }
                                        1 => {
                                            self.side_menu.state = SideMenuState::Null;
                                            self.main_menu.state = MainMenuState::Root;
                                        }
                                        _ => (),
                                    }
                                }
                            }
                        },
                        Focus::SideMenu => match self.side_menu.state {
                            SideMenuState::Null => (),
                            SideMenuState::SelectDeckFromFile => {
                                self.focus = Focus::Popup;
                                self.popup.state = PopupState::Show;
                                self.popup.title =
                                    Span::from("解析卡组").add_modifier(Modifier::BOLD);
                                self.popup.message = "正在联网解析卡组中...".to_string();
                                if let Some(i) = self.side_menu.items_state.selected() {
                                    self.deck = Some(
                                        Deck::from_file(
                                            self.side_menu.items[i].clone(),
                                            format!(
                                                "./assets/deck/{}.ydk",
                                                self.side_menu.items[i]
                                            ),
                                        )
                                        .await?,
                                    );
                                }
                                self.popup.state = PopupState::Hide;
                                self.focus = Focus::SideMenu;
                            }
                        },
                        Focus::Popup => (),
                    },
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }
        Ok(())
    }
}
