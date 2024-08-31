use super::DialogResponse;
use crate::{BotIdExt, Button, Ui};
use kartoffels_world::prelude::{BotId, Snapshot};
use ratatui::style::Stylize;
use ratatui::widgets::{Cell, Row, StatefulWidget, Table, TableState};
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct BotsDialog {
    pub table: TableState,
}

impl BotsDialog {
    const WIDTHS: [u16; 4] = [
        4,                    // #
        BotId::LENGTH as u16, // id
        6,                    // age
        7,                    // score
    ];

    pub fn render(
        &mut self,
        ui: &mut Ui,
        snapshot: &Snapshot,
    ) -> Option<DialogResponse> {
        let width = Self::WIDTHS.iter().copied().sum::<u16>() + 4;
        let height = ui.area().height - 2;

        let mut response = None;

        ui.info_dialog(width, height, Some(" bots "), |ui| {
            let header = Row::new(vec!["#", "id", "age", "score ⯆"]);

            let rows =
                snapshot.bots.alive.iter_sorted_by_scores().enumerate().map(
                    |(place, (bot, score))| {
                        Row::new([
                            Cell::new(format!("#{}", place + 1)),
                            Cell::new(bot.id.to_string()).fg(bot.id.color()),
                            Cell::new(bot.age.to_string()),
                            Cell::new(score.to_string()),
                        ])
                    },
                );

            if Button::new(KeyCode::Escape, "close")
                .right()
                .block()
                .render(ui)
                .pressed
            {
                response = Some(DialogResponse::Close);
            }

            ui.step(1);

            Table::new(rows, Self::WIDTHS).header(header).render(
                ui.area(),
                ui.buf(),
                &mut self.table,
            );
        });

        response
    }
}
