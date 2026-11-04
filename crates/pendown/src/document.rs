//! The program being edited, as a file (ADR 0006): one program per file,
//! plain UTF-8 with a `.logo` extension.

use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Document {
    /// Where it's saved, if it has been.
    pub path: Option<PathBuf>,
    /// The text as last opened or saved, to tell whether there are changes.
    saved: String,
}

impl Document {
    /// A new, unsaved document that starts with `text`. It only counts as
    /// changed once it's edited.
    pub fn new(text: &str) -> Document {
        Document {
            path: None,
            saved: text.to_string(),
        }
    }

    pub fn open(path: &Path) -> io::Result<(Document, String)> {
        let text = std::fs::read_to_string(path)?;
        Ok((
            Document {
                path: Some(path.to_path_buf()),
                saved: text.clone(),
            },
            text,
        ))
    }

    /// Saves `text` to `path`, giving it the `.logo` extension if it has
    /// none, and remembers it as this document's file.
    pub fn save_as(&mut self, path: &Path, text: &str) -> io::Result<PathBuf> {
        let path = if path.extension().is_none() {
            path.with_extension("logo")
        } else {
            path.to_path_buf()
        };
        std::fs::write(&path, text)?;
        self.path = Some(path.clone());
        self.saved = text.to_string();
        Ok(path)
    }

    pub fn changed(&self, text: &str) -> bool {
        self.saved != text
    }

    /// The file's name, or "Untitled".
    pub fn name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map_or("Untitled".into(), |n| n.to_string_lossy().into_owned())
    }

    /// The window title: the file's name, with a dot when there are unsaved
    /// changes.
    pub fn title(&self, text: &str) -> String {
        let dot = if self.changed(text) { "● " } else { "" };
        format!("{dot}{} — Pendown", self.name())
    }

    /// A name for saving or exporting, from the file's: `flower.logo` is
    /// exported as `flower.png`.
    pub fn suggested_name(&self, extension: &str) -> String {
        let stem = self
            .path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map_or("untitled".into(), |s| s.to_string_lossy().into_owned());
        format!("{stem}.{extension}")
    }
}

/// `$XDG_CONFIG_HOME/pendown`, `~/.config/pendown`, or `%APPDATA%\pendown`.
pub(crate) fn settings_dir() -> Option<PathBuf> {
    let base = std::env::var_os("PENDOWN_SETTINGS")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(|d| PathBuf::from(d).join("pendown")))
        .or_else(|| std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join("pendown")))
        .or_else(|| std::env::var_os("HOME").map(|d| PathBuf::from(d).join(".config/pendown")))?;
    Some(base)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("pendown-document-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_new_document() {
        let doc = Document::new("FD 10");
        assert!(!doc.changed("FD 10"), "the starting text isn't a change");
        assert!(doc.changed("FD 20"));
        assert_eq!(doc.title("FD 10"), "Untitled — Pendown");
        assert_eq!(doc.title("FD 20"), "● Untitled — Pendown");
        assert_eq!(doc.suggested_name("png"), "untitled.png");
    }

    #[test]
    fn saving_and_opening() {
        let dir = temp("save");
        let mut doc = Document::new("");
        let path = doc.save_as(&dir.join("flower"), "FLOWER 80").unwrap();
        assert_eq!(path, dir.join("flower.logo"), "the extension is added");
        assert!(!doc.changed("FLOWER 80"));
        assert_eq!(doc.title("FLOWER 80"), "flower.logo — Pendown");
        assert_eq!(doc.suggested_name("png"), "flower.png");
        let (opened, text) = Document::open(&path).unwrap();
        assert_eq!(text, "FLOWER 80");
        assert_eq!(opened.path, Some(path.clone()));
        assert!(!opened.changed("FLOWER 80"));
        // An extension that's already there is kept.
        assert_eq!(
            doc.save_as(&dir.join("star.txt"), "x").unwrap(),
            dir.join("star.txt")
        );
    }

    #[test]
    fn opening_a_missing_file_is_an_error() {
        assert!(Document::open(Path::new("/no/such/file.logo")).is_err());
    }
}
