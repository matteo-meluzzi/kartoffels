mod admin;
mod challenges;
mod play;
mod sandbox;
mod tutorial;
mod widgets;

use self::widgets::*;
use crate::Background;
use anyhow::Result;
use kartoffels_store::{Session, Store};
use kartoffels_ui::{Fade, FadeDir, KeyCode, Modifiers, Term, UiWidget};
use ratatui::layout::{Constraint, Layout};
use tracing::debug;

pub async fn run(
    store: &Store,
    sess: &Session,
    term: &mut Term,
    bg: &Background,
) -> Result<()> {
    let mut fade_in = true;

    loop {
        match run_once(store, term, bg, fade_in).await? {
            Event::Admin => {
                admin::run(store, sess, term, bg).await?;
                fade_in = false;
            }

            Event::Play => {
                play::run(store, sess, term, bg).await?;
                fade_in = false;
            }

            Event::Sandbox => {
                sandbox::run(store, sess, term, bg).await?;
                fade_in = false;
            }

            Event::Tutorial => {
                tutorial::run(store, sess, term).await?;
                fade_in = true;
            }

            Event::Challenges => {
                challenges::run(store, sess, term, bg).await?;
                fade_in = false;
            }

            Event::Quit => {
                return Ok(());
            }
        }
    }
}

async fn run_once(
    store: &Store,
    term: &mut Term,
    bg: &Background,
    fade_in: bool,
) -> Result<Event> {
    debug!("run()");

    let mut fade_in = if fade_in && !store.testing() {
        Some(Fade::new(FadeDir::In))
    } else {
        None
    };

    let mut fade_out: Option<(Fade, Event)> = None;

    loop {
        let event = term
            .frame(|ui| {
                let [_, area, _] = Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Length(Header::width()),
                    Constraint::Fill(1),
                ])
                .areas(ui.area);

                let [_, header_area, _, menu_area, _] = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(Header::height()),
                    Constraint::Fill(1),
                    Constraint::Length(Menu::height(store, ui)),
                    Constraint::Fill(1),
                ])
                .areas(area);

                let [_, menu_area, _] = Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Length(Menu::width()),
                    Constraint::Fill(1),
                ])
                .areas(menu_area);

                ui.widget(bg);

                ui.clamp(header_area, |ui| {
                    Header::render(ui);
                });

                ui.clamp(menu_area, |ui| {
                    Menu::render(store, ui);
                });

                Footer::render(store, ui);

                if let Some(fade) = &fade_in
                    && fade.render(ui).is_completed()
                {
                    fade_in = None;
                }

                if let Some((fade, _)) = &fade_out {
                    fade.render(ui);
                }

                if ui.key(KeyCode::Char('x'), Modifiers::ALT) {
                    ui.throw(Event::Admin);
                }
            })
            .await?;

        if let Some((fade, event)) = &fade_out
            && fade.is_completed()
        {
            return Ok(*event);
        }

        if let Some(event) = event {
            if event.fade_out() && !store.testing() {
                fade_out = Some((Fade::new(FadeDir::Out), event));
            } else {
                return Ok(event);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Event {
    Admin,
    Play,
    Sandbox,
    Tutorial,
    Challenges,
    Quit,
}

impl Event {
    fn fade_out(&self) -> bool {
        matches!(self, Event::Tutorial)
    }
}
