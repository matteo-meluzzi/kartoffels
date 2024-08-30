use super::SidePanelEvent;
use crate::{Button, Ui};
use ratatui::layout::{Constraint, Layout};
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct IdleSidePanel;

impl IdleSidePanel {
    pub fn render(ui: &mut Ui, enabled: bool) -> Option<SidePanelEvent> {
        let mut event = None;

        let [_, join_area, upload_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(ui.area());

        ui.clamp(join_area, |ui| {
            if Button::new(KeyCode::Char('j'), "join bot")
                .enabled(enabled)
                .render(ui)
                .activated
            {
                event = Some(SidePanelEvent::JoinBot);
            }
        });

        ui.clamp(upload_area, |ui| {
            if Button::new(KeyCode::Char('u'), "upload bot")
                .enabled(enabled)
                .render(ui)
                .activated
            {
                event = Some(SidePanelEvent::UploadBot);
            }
        });

        event
    }
}
