//! New, Open, Save, Save As and Export PNG (issue 14), with a warning before
//! unsaved changes are lost.

use std::path::{Path, PathBuf};

use gpui_kit::*;
use pendown::document::{Document, last_folder, remember_folder};

use crate::app::App;

actions!(pendown, [NewFile, OpenFile, SaveFile, SaveFileAs, ExportPicture, RunProgram, ToggleMetrics, ShowHelp]);

/// The usual shortcuts: Ctrl on Linux and Windows, Cmd on macOS.
pub fn bind_keys(cx: &mut gpui_kit::App) {
    cx.bind_keys([
        KeyBinding::new("secondary-n", NewFile, None),
        KeyBinding::new("secondary-o", OpenFile, None),
        KeyBinding::new("secondary-s", SaveFile, None),
        KeyBinding::new("secondary-shift-s", SaveFileAs, None),
        KeyBinding::new("secondary-e", ExportPicture, None),
        KeyBinding::new("secondary-enter", RunProgram, None),
        // The metrics overlay (ADR 0020).
        KeyBinding::new("f12", ToggleMetrics, None),
        // Help for the word under the editor's cursor (phase 5).
        KeyBinding::new("f1", ShowHelp, None),
    ]);
}

/// What to do once any unsaved changes have been dealt with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum After {
    Nothing,
    NewFile,
    OpenFile,
    Close,
}

impl App {
    fn text(&self, cx: &Context<Self>) -> String {
        self.editor.read(cx).value().to_string()
    }

    fn folder(&self) -> PathBuf {
        self.document
            .path
            .as_ref()
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .or_else(last_folder)
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Keeps the window title in step with the file and its changes.
    pub(crate) fn update_title(&mut self, window: &mut Window, cx: &Context<Self>) {
        let title = self.document.title(&self.text(cx));
        if title != self.title {
            window.set_window_title(&title);
            self.title = title;
        }
    }

    /// Does `after`, first asking whether to save unsaved changes.
    pub(crate) fn then(&mut self, after: After, window: &mut Window, cx: &mut Context<Self>) {
        if !self.document.changed(&self.text(cx)) {
            self.proceed(after, window, cx);
            return;
        }
        let message = format!("Save the changes to {}?", self.document.name());
        let answer = window.prompt(
            PromptLevel::Warning,
            &message,
            Some("If you don't, they'll be lost."),
            &["Save", "Don't Save", "Cancel"],
            cx,
        );
        cx.spawn_in(window, async move |this, cx| {
            let Ok(answer) = answer.await else { return };
            this.update_in(cx, |this, window, cx| match answer {
                0 => this.save(after, window, cx),
                1 => this.proceed(after, window, cx),
                _ => {}
            })
            .ok();
        })
        .detach();
    }

    fn proceed(&mut self, after: After, window: &mut Window, cx: &mut Context<Self>) {
        match after {
            After::Nothing => {}
            After::NewFile => {
                self.stop(cx);
                self.document = Document::new("");
                self.editor.update(cx, |state, cx| state.set_value("", window, cx));
                self.check_program(cx);
            }
            After::OpenFile => {
                let options = PathPromptOptions {
                    files: true,
                    directories: false,
                    multiple: false,
                    prompt: Some("Open".into()),
                };
                let paths = cx.prompt_for_paths(options);
                cx.spawn_in(window, async move |this, cx| {
                    let Ok(Ok(Some(paths))) = paths.await else { return };
                    let Some(path) = paths.into_iter().next() else { return };
                    this.update_in(cx, |this, window, cx| this.open_path(&path, window, cx)).ok();
                })
                .detach();
            }
            After::Close => {
                self.closing = true;
                window.remove_window();
            }
        }
    }

    pub(crate) fn open_path(&mut self, path: &Path, window: &mut Window, cx: &mut Context<Self>) {
        match Document::open(path) {
            Ok((document, text)) => {
                self.stop(cx);
                self.document = document;
                self.editor.update(cx, |state, cx| state.set_value(text, window, cx));
                // Setting the text sends no change event, so check it here.
                self.check_program(cx);
                remember_folder(path);
                self.print(format!("Opened {}", path.display()));
            }
            Err(e) => self.print(format!("Couldn't open {}: {e}", path.display())),
        }
        cx.notify();
    }

    /// Saves to the file, asking for one if there isn't one yet, then does
    /// `after`.
    pub(crate) fn save(&mut self, after: After, window: &mut Window, cx: &mut Context<Self>) {
        match self.document.path.clone() {
            Some(path) => {
                if self.save_to(&path, cx) {
                    self.proceed(after, window, cx);
                }
            }
            None => self.save_as(after, window, cx),
        }
    }

    pub(crate) fn save_as(&mut self, after: After, window: &mut Window, cx: &mut Context<Self>) {
        let path = cx.prompt_for_new_path(&self.folder(), Some(&self.document.suggested_name("logo")));
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(path))) = path.await else { return };
            this.update_in(cx, |this, window, cx| {
                if this.save_to(&path, cx) {
                    this.proceed(after, window, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    /// Writes the program to `path`. Returns whether it worked.
    pub(crate) fn save_to(&mut self, path: &Path, cx: &mut Context<Self>) -> bool {
        let text = self.text(cx);
        let saved = match self.document.save_as(path, &text) {
            Ok(saved) => {
                remember_folder(&saved);
                self.print(format!("Saved {}", saved.display()));
                true
            }
            Err(e) => {
                self.print(format!("Couldn't save {}: {e}", path.display()));
                false
            }
        };
        cx.notify();
        saved
    }

    /// Saves the drawing as a PNG, at the canvas's resolution, without the
    /// turtle (ADR 0010).
    pub(crate) fn export(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let path = cx.prompt_for_new_path(&self.folder(), Some(&self.document.suggested_name("png")));
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(path))) = path.await else { return };
            this.update_in(cx, |this, _, cx| this.export_to(&path, cx)).ok();
        })
        .detach();
    }

    pub(crate) fn export_to(&mut self, path: &Path, cx: &mut Context<Self>) {
        let path = if path.extension().is_none() { path.with_extension("png") } else { path.to_path_buf() };
        // The canvas only draws what's new when it paints, which may not
        // have happened since the program finished.
        let png = {
            let mut canvas = self.canvas.borrow_mut();
            canvas.drawing.draw_pending();
            canvas.drawing.png()
        };
        match std::fs::write(&path, png) {
            Ok(()) => {
                remember_folder(&path);
                self.print(format!("Saved the drawing to {}", path.display()));
            }
            Err(e) => self.print(format!("Couldn't save the drawing to {}: {e}", path.display())),
        }
        cx.notify();
    }

    /// Asked before the window closes: not if there are unsaved changes,
    /// until the user has answered.
    pub(crate) fn should_close(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.closing || !self.document.changed(&self.text(cx)) {
            return true;
        }
        self.then(After::Close, window, cx);
        false
    }
}
