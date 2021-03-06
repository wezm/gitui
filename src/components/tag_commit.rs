use super::{
    textinput::TextInputComponent, visibility_blocking,
    CommandBlocking, CommandInfo, Component, DrawableComponent,
};
use crate::{
    queue::{InternalEvent, NeedsUpdate, Queue},
    strings::{self, commands},
    ui::style::SharedTheme,
};
use anyhow::Result;
use asyncgit::{
    sync::{self, CommitId},
    CWD,
};
use crossterm::event::{Event, KeyCode};
use tui::{backend::Backend, layout::Rect, Frame};

pub struct TagCommitComponent {
    input: TextInputComponent,
    commit_id: Option<CommitId>,
    queue: Queue,
}

impl DrawableComponent for TagCommitComponent {
    fn draw<B: Backend>(
        &self,
        f: &mut Frame<B>,
        rect: Rect,
    ) -> Result<()> {
        self.input.draw(f, rect)?;

        Ok(())
    }
}

impl Component for TagCommitComponent {
    fn commands(
        &self,
        out: &mut Vec<CommandInfo>,
        force_all: bool,
    ) -> CommandBlocking {
        if self.is_visible() || force_all {
            self.input.commands(out, force_all);

            out.push(CommandInfo::new(
                commands::TAG_COMMIT_CONFIRM_MSG,
                true,
                true,
            ));
        }

        visibility_blocking(self)
    }

    fn event(&mut self, ev: Event) -> Result<bool> {
        if self.is_visible() {
            if self.input.event(ev)? {
                return Ok(true);
            }

            if let Event::Key(e) = ev {
                if let KeyCode::Enter = e.code {
                    self.tag()
                }

                return Ok(true);
            }
        }
        Ok(false)
    }

    fn is_visible(&self) -> bool {
        self.input.is_visible()
    }

    fn hide(&mut self) {
        self.input.hide()
    }

    fn show(&mut self) -> Result<()> {
        self.input.show()?;

        Ok(())
    }
}

impl TagCommitComponent {
    ///
    pub fn new(queue: Queue, theme: SharedTheme) -> Self {
        Self {
            queue,
            input: TextInputComponent::new(
                theme,
                strings::TAG_COMMIT_POPUP_TITLE,
                strings::TAG_COMMIT_POPUP_MSG,
            ),
            commit_id: None,
        }
    }

    ///
    pub fn open(&mut self, id: CommitId) -> Result<()> {
        self.commit_id = Some(id);
        self.show()?;

        Ok(())
    }

    ///
    pub fn tag(&mut self) {
        if let Some(commit_id) = self.commit_id {
            match sync::tag(CWD, &commit_id, self.input.get_text()) {
                Ok(_) => {
                    self.input.clear();
                    self.hide();

                    self.queue.borrow_mut().push_back(
                        InternalEvent::Update(NeedsUpdate::ALL),
                    );
                }
                Err(e) => {
                    self.hide();
                    log::error!("e: {}", e,);
                    self.queue.borrow_mut().push_back(
                        InternalEvent::ShowErrorMsg(format!(
                            "tag error:\n{}",
                            e,
                        )),
                    );
                }
            }
        }
    }
}
