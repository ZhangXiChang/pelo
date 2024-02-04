use ratatui::{prelude::*, widgets::*};

use crate::system::*;

pub struct Title {
    title: String,
}
impl Title {
    pub fn new() -> Self {
        Self {
            title: "牌佬助手".to_string(),
        }
    }
}
impl WidgetComponent for Title {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Paragraph::new(self.title.clone())
                .block(Block::new().borders(Borders::ALL))
                .alignment(Alignment::Center),
            area,
        )
    }
}
