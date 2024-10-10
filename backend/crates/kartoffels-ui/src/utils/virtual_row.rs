use crate::{Render, Ui};
use ratatui::prelude::Rect;

#[derive(Debug)]
pub struct VirtualRow<'a, 'b, T> {
    ui: &'a mut Ui<'b, T>,
    widths: &'static [u16],
    nth: usize,
    offset: u16,
}

impl<'a, 'b, T> VirtualRow<'a, 'b, T> {
    pub fn new(ui: &'a mut Ui<'b, T>, widths: &'static [u16]) -> Self {
        Self {
            ui,
            widths,
            nth: 0,
            offset: 0,
        }
    }

    pub fn add(&mut self, widget: impl Render<T>) -> &mut Self {
        let width = self.widths[self.nth];

        let area = {
            let area = self.ui.area();

            Rect {
                x: area.x + self.offset,
                y: area.y,
                width,
                height: area.height,
            }
        };

        self.ui.clamp(area, |ui| {
            widget.render(ui);
        });

        self.nth += 1;
        self.offset += width;
        self
    }
}
