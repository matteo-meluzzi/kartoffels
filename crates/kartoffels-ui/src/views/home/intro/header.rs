use crate::{theme, BlockExt};
use ratatui::prelude::{Buffer, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Block, Padding, Widget};

const TEXT: &[&str] = &[
    "welcome to kartoffels, a game where you're given a potato:",
    "",
    "     ██████     ",
    "   ██░░░░░░██   ",
    " ██░░░░░░░░░░██ ",
    " ██░░░░░░░░░░██ ",
    "   ██░░░░░░░░██ ",
    "   oo████████oo ",
    "   oo        oo ",
    "",
    "... and your job is to implement a firmware for it",
    "",
    "robots are limited to 64 khz cpu & 128 kb of ram and the",
    "game happens online - you can see your robot fighting",
    "other players' bots and you can learn from their behavior",
    "",
    "can you develop the best, the longest surviving, the",
    "most deadly machine imaginable?",
];

#[derive(Debug)]
pub struct Header;

impl Header {
    pub const WIDTH: u16 = 58 + 2 + 2;
    pub const HEIGHT: u16 = TEXT.len() as u16 + 2;
}

impl Widget for Header {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = Block::bordered()
            .border_style(Style::new().fg(theme::GREEN).bg(theme::BG))
            .padding(Padding::horizontal(1))
            .render_and_measure(area, buf);

        let mut text = Text::default();

        for (idx, &line) in TEXT.iter().enumerate() {
            if (2..=8).contains(&idx) {
                text.push_line(line.fg(theme::POTATO));
            } else {
                text.push_line(line);
            }
        }

        text.centered().render(area, buf);
    }
}
