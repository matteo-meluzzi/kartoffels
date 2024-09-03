mod idle;
mod joined;

use self::idle::*;
use self::joined::*;
use super::{Controller, Dialog, JoinedBot, Response, State};
use crate::{Clear, Term, Ui};
use anyhow::Result;
use kartoffels_world::prelude::Snapshot;
use ratatui::layout::Rect;
use std::ops::ControlFlow;

#[derive(Debug)]
pub struct SidePanel;

impl SidePanel {
    pub const WIDTH: u16 = 25;

    pub fn render(
        ui: &mut Ui,
        ctrl: &Controller,
        world: &Snapshot,
        bot: Option<&JoinedBot>,
        enabled: bool,
    ) -> Option<SidePanelResponse> {
        let area = {
            let area = ui.area();

            Rect {
                x: area.x + 1,
                y: area.y,
                width: area.width - 1,
                height: area.height,
            }
        };

        Clear::render(ui);

        ui.clamp(area, |ui| {
            if let Some(bot) = bot {
                JoinedSidePanel::render(ui, ctrl, world, bot, enabled)
            } else {
                IdleSidePanel::render(ui, enabled)
            }
        })
    }
}

#[derive(Debug)]
pub enum SidePanelResponse {
    UploadBot,
    JoinBot,
    LeaveBot,
    RestartBot,
    DestroyBot,
    FollowBot,
    ShowBotHistory,
}

impl SidePanelResponse {
    pub async fn handle(
        self,
        state: &mut State,
        term: &mut Term,
    ) -> Result<ControlFlow<Response, ()>> {
        match self {
            SidePanelResponse::UploadBot => {
                if term.ty().is_http() {
                    term.send(vec![0x04]).await?;
                }

                state.dialog = Some(Dialog::UploadBot(Default::default()));
            }

            SidePanelResponse::JoinBot => {
                state.dialog = Some(Dialog::JoinBot(Default::default()));
            }

            SidePanelResponse::LeaveBot => {
                state.bot = None;
            }

            SidePanelResponse::RestartBot => {
                if let Some(bot) = &state.bot {
                    state.handle.restart_bot(bot.id).await?;
                }
            }

            SidePanelResponse::DestroyBot => {
                if let Some(bot) = state.bot.take() {
                    state.handle.destroy_bot(bot.id).await?;
                }
            }

            SidePanelResponse::FollowBot => {
                if let Some(bot) = &mut state.bot {
                    bot.is_followed = !bot.is_followed;
                }
            }

            SidePanelResponse::ShowBotHistory => {
                todo!();
            }
        }

        Ok(ControlFlow::Continue(()))
    }
}
