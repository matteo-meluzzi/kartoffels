mod bottom;
mod camera;
mod dialog;
mod driver;
mod event;
mod map;
mod perms;
mod side;

use self::bottom::*;
use self::camera::*;
use self::dialog::*;
pub use self::dialog::{HelpDialog, HelpDialogRef, HelpDialogResponse};
use self::event::*;
use self::map::*;
pub use self::perms::*;
use self::side::*;
use crate::DriverEventRx;
use anyhow::Result;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use futures_util::FutureExt;
use itertools::Either;
use kartoffels_store::{SessionId, Store};
use kartoffels_ui::{Clear, Fade, FadeDir, Render, Term, Ui};
use kartoffels_world::prelude::{
    BotId, ClockSpeed, CreateBotRequest, Handle as WorldHandle,
    Snapshot as WorldSnapshot, SnapshotStream,
};
use ratatui::layout::{Constraint, Layout};
use std::ops::ControlFlow;
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;

pub async fn run(
    store: &Store,
    sess: SessionId,
    term: &mut Term,
    mut driver: DriverEventRx,
) -> Result<()> {
    debug!("run()");

    let mut fade = Some(Fade::new(FadeDir::In));
    let mut state = State::default();
    let mut frame = Instant::now();

    loop {
        let event = term
            .draw(|ui| {
                state.tick(frame.elapsed().as_secs_f32(), store);
                state.render(ui, sess);

                if let Some(fade) = &fade {
                    fade.render(ui);
                }

                frame = Instant::now();
            })
            .await?;

        term.poll().await?;

        if let Some(event) = event {
            if let ControlFlow::Break(_) =
                event.handle(store, sess, term, &mut state).await?
            {
                fade = Some(Fade::new(FadeDir::Out));
            }
        }

        state.poll(term, &mut driver).await?;

        if let Some(fade) = &fade {
            if fade.dir() == FadeDir::Out && fade.is_completed() {
                return Ok(());
            }
        }
    }
}

#[derive(Default)]
struct State {
    bot: Option<JoinedBot>,
    camera: Camera,
    dialog: Option<Dialog>,
    handle: Option<WorldHandle>,
    help: Option<HelpDialogRef>,
    map: Map,
    paused: bool,
    perms: Perms,
    snapshot: Arc<WorldSnapshot>,
    snapshots: Option<SnapshotStream>,
    speed: ClockSpeed,
    status: Option<(String, Instant)>,
}

impl State {
    fn tick(&mut self, dt: f32, store: &Store) {
        if let Some(bot) = &self.bot {
            if bot.follow {
                if let Some(bot) = self.snapshot.bots().alive().by_id(bot.id) {
                    self.camera.animate_to(bot.pos);
                }
            }
        }

        self.camera.tick(dt, store);
    }

    fn render(&mut self, ui: &mut Ui<Event>, sess: SessionId) {
        let [main_area, bottom_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
                .areas(ui.area());

        let [map_area, side_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(SidePanel::WIDTH),
        ])
        .areas(main_area);

        Clear::render(ui);

        ui.enable(self.dialog.is_none(), |ui| {
            ui.clamp(bottom_area, |ui| {
                BottomPanel::render(ui, self);
            });

            if self.handle.is_some() {
                ui.enable(self.perms.ui_enabled, |ui| {
                    ui.clamp(side_area, |ui| {
                        SidePanel::render(ui, self);
                    });

                    ui.clamp(map_area, |ui| {
                        Map::render(ui, self);
                    });
                });
            }
        });

        if let Some(dialog) = &mut self.dialog {
            dialog.render(ui, sess, &self.snapshot);
        }
    }

    async fn poll(
        &mut self,
        term: &mut Term,
        driver: &mut DriverEventRx,
    ) -> Result<()> {
        while let Some(event) = driver.recv().now_or_never().flatten() {
            event.handle(self, term).await?;
        }

        if let Some(snapshots) = &mut self.snapshots {
            if let Some(snapshot) = snapshots.next().now_or_never() {
                self.update_snapshot(snapshot?);
            }
        }

        Ok(())
    }

    fn update_snapshot(&mut self, snapshot: Arc<WorldSnapshot>) {
        // If map size's changed, recenter the camera - this comes handy for
        // drivers which call `world.set_map()`, e.g. the tutorial
        if snapshot.map().size() != self.snapshot.map().size() {
            self.camera.move_to(snapshot.map().center());
        }

        self.snapshot = snapshot;

        if let Some(bot) = &mut self.bot {
            let exists_now = self.snapshot.bots().by_id(bot.id).is_some();

            bot.exists |= exists_now;

            if bot.exists && !exists_now {
                self.bot = None;
            }
        }
    }

    async fn pause(&mut self) -> Result<()> {
        if !self.paused {
            self.paused = true;
            self.snapshots = None;

            if self.perms.sync_pause
                && let Some(handle) = &self.handle
            {
                handle.pause().await?;
            }
        }

        Ok(())
    }

    async fn resume(&mut self) -> Result<()> {
        if self.paused {
            self.paused = false;

            self.snapshots =
                self.handle.as_ref().map(|handle| handle.snapshots());

            if self.perms.sync_pause
                && let Some(handle) = &self.handle
            {
                handle.resume().await?;
            }
        }

        Ok(())
    }

    fn join_bot(&mut self, id: BotId) {
        self.bot = Some(JoinedBot {
            id,
            follow: true,
            exists: false,
        });

        self.map.blink = Instant::now();
    }

    async fn upload_bot(&mut self, src: Either<String, Vec<u8>>) -> Result<()> {
        let src = match src {
            Either::Left(src) => {
                let src = src.trim().replace('\r', "");
                let src = src.trim().replace('\n', "");

                match BASE64_STANDARD.decode(src) {
                    Ok(src) => src,
                    Err(err) => {
                        self.dialog = Some(Dialog::Error(ErrorDialog::new(
                            format!("couldn't decode pasted content:\n\n{err}"),
                        )));

                        return Ok(());
                    }
                }
            }

            Either::Right(src) => src,
        };

        let id = self
            .handle
            .as_ref()
            .unwrap()
            .create_bot(CreateBotRequest::new(src))
            .await;

        let id = match id {
            Ok(id) => id,

            Err(err) => {
                self.dialog =
                    Some(Dialog::Error(ErrorDialog::new(format!("{err:?}"))));

                return Ok(());
            }
        };

        self.join_bot(id);
        self.resume().await?;

        Ok(())
    }
}

#[derive(Debug)]
struct JoinedBot {
    id: BotId,
    follow: bool,
    exists: bool,
}
