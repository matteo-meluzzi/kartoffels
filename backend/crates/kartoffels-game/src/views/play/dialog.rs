mod bots;
mod error;
mod help;
mod join_bot;
mod leaving;
mod upload_bot;

pub use self::bots::*;
pub use self::error::*;
pub use self::help::*;
pub use self::join_bot::*;
pub use self::leaving::*;
pub use self::upload_bot::*;
use super::Event;
use kartoffels_ui::{Backdrop, Ui};
use kartoffels_world::prelude::Snapshot;
use termwiz::input::{KeyCode, Modifiers};

pub enum Dialog {
    Bots(BotsDialog),
    Error(ErrorDialog),
    Help(HelpDialogRef),
    JoinBot(JoinBotDialog),
    Leaving(LeavingDialog),
    UploadBot(UploadBotDialog),

    Custom(Box<dyn FnMut(&mut Ui) + Send>),
}

impl Dialog {
    pub fn render(&mut self, ui: &mut Ui, world: &Snapshot) {
        Backdrop::render(ui);

        match self {
            Dialog::Bots(this) => {
                this.render(ui, world);
            }
            Dialog::Error(this) => {
                this.render(ui);
            }
            Dialog::JoinBot(this) => {
                this.render(ui, world);
            }
            Dialog::Leaving(this) => {
                this.render(ui);
            }
            Dialog::UploadBot(this) => {
                this.render(ui);
            }

            Dialog::Help(this) => {
                this.render(ui);

                if let Some(event) = ui.catch::<HelpDialogResponse>() {
                    match event {
                        HelpDialogResponse::Copy(payload) => {
                            ui.throw(Event::CopyToClipboard(
                                payload.to_owned(),
                            ));
                        }
                        HelpDialogResponse::Close => {
                            ui.throw(Event::CloseDialog);
                        }
                    }
                }

                if ui.key(KeyCode::Escape, Modifiers::NONE) {
                    ui.throw(Event::CloseDialog);
                }
            }

            Dialog::Custom(this) => {
                (this)(ui);
            }
        }
    }
}
