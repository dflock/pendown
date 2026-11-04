//! `config.kdl` (ADR 0032): what Pendown remembers between runs, and the
//! settings that are only changed by editing the file. The parts that don't
//! need a window.

use std::path::{Path, PathBuf};

use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

use crate::document::settings_dir;

/// A new file starts with this, so its settings can be found. Saving keeps
/// whatever the file says outside the nodes Pendown remembers, comments too.
const TEMPLATE: &str = "\
// Pendown's settings. It reads them when it starts, and keeps your changes
// and comments when it saves what it remembers.

// List the UCBLogo commands Pendown leaves out, with why, in Help.
show-obsolete #false

// What Pendown remembers from last time, rewritten when it closes.
last-folder
window
panes
speed
help
";

/// The usage `--help` prints.
pub const USAGE: &str = "\
Usage: pendown [--config FILE | --no-config]

  --config FILE  Read and save settings in FILE, not the usual config.kdl
  --no-config    Start with the default settings, and save none
  --help         Show this
";

const MIN_WINDOW: (f64, f64) = (480.0, 360.0);
/// The least the canvas, and the panes above the console, are left with.
const MIN_LEFT: f64 = 160.0;
const MIN_PANE: f64 = 80.0;

/// The side panel's tabs, in the order they're shown.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tab {
    Tutorial,
    Help,
    /// Examples to run or load, with their pictures (`.scratch/gallery/`).
    Gallery,
    /// Every keyboard shortcut (`.scratch/shortcuts/`).
    Shortcuts,
}

impl Tab {
    fn name(self) -> &'static str {
        match self {
            Tab::Tutorial => "tutorial",
            Tab::Help => "help",
            Tab::Gallery => "gallery",
            Tab::Shortcuts => "shortcuts",
        }
    }

    fn named(name: &str) -> Option<Tab> {
        [Tab::Tutorial, Tab::Help, Tab::Gallery, Tab::Shortcuts]
            .into_iter()
            .find(|t| t.name() == name)
    }
}

/// Whether the window fills the screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WindowState {
    Windowed,
    Maximized,
    Fullscreen,
}

impl WindowState {
    fn name(self) -> &'static str {
        match self {
            WindowState::Windowed => "windowed",
            WindowState::Maximized => "maximized",
            WindowState::Fullscreen => "fullscreen",
        }
    }

    fn named(name: &str) -> Option<WindowState> {
        [
            WindowState::Windowed,
            WindowState::Maximized,
            WindowState::Fullscreen,
        ]
        .into_iter()
        .find(|s| s.name() == name)
    }
}

/// The side panel.
#[derive(Clone, Debug, PartialEq)]
pub struct HelpConfig {
    pub open: bool,
    pub tab: Tab,
    /// The help page's command, by its key in the reference, or `None` for
    /// the contents.
    pub page: Option<String>,
    pub search: String,
    pub pinned: bool,
    /// The tutorial's lesson, counting from 0.
    pub lesson: usize,
    /// How far `tab` is scrolled down.
    pub scroll: f64,
}

/// Everything in the file. Sizes are in logical pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub show_obsolete: bool,
    /// The folder last opened from or saved to.
    last_folder: Option<PathBuf>,
    /// The window's size when it isn't maximized or fullscreen.
    pub window: (f64, f64),
    pub window_state: WindowState,
    pub editor_width: f64,
    pub console_height: f64,
    pub help_width: f64,
    /// The speed slider's level.
    pub speed: u8,
    pub help: HelpConfig,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            show_obsolete: false,
            last_folder: None,
            window: (1400.0, 900.0),
            window_state: WindowState::Windowed,
            editor_width: 460.0,
            console_height: 190.0,
            help_width: 380.0,
            speed: 5,
            help: HelpConfig {
                open: false,
                tab: Tab::Help,
                page: None,
                search: String::new(),
                pinned: false,
                lesson: 0,
                scroll: 0.0,
            },
        }
    }
}

fn number(node: &KdlNode, key: &str) -> Option<f64> {
    let value = match node.get(key)? {
        KdlValue::Integer(n) => *n as f64,
        KdlValue::Float(n) => *n,
        _ => return None,
    };
    value.is_finite().then_some(value)
}

fn boolean(node: &KdlNode, key: &str) -> Option<bool> {
    node.get(key)?.as_bool()
}

fn string<'a>(node: &'a KdlNode, key: &str) -> Option<&'a str> {
    node.get(key)?.as_string()
}

impl Config {
    /// What `text` says, with the default for anything it doesn't, or says
    /// wrongly. It's an error only if it isn't KDL at all.
    pub fn parse(text: &str) -> Result<Config, String> {
        let document: KdlDocument = text.parse().map_err(|e: kdl::KdlError| e.to_string())?;
        let mut config = Config::default();
        if let Some(on) = document
            .get("show-obsolete")
            .and_then(|n| n.get(0))
            .and_then(KdlValue::as_bool)
        {
            config.show_obsolete = on;
        }
        config.last_folder = document
            .get("last-folder")
            .and_then(|n| n.get(0))
            .and_then(KdlValue::as_string)
            .filter(|folder| !folder.is_empty())
            .map(PathBuf::from);
        if let Some(node) = document.get("window") {
            let (width, height) = config.window;
            config.window = (
                number(node, "width").unwrap_or(width).max(MIN_WINDOW.0),
                number(node, "height").unwrap_or(height).max(MIN_WINDOW.1),
            );
            if let Some(state) = string(node, "state").and_then(WindowState::named) {
                config.window_state = state;
            }
        }
        if let Some(node) = document.get("panes") {
            let at_least = |key, or: f64| number(node, key).unwrap_or(or).max(MIN_PANE);
            config.editor_width = at_least("editor", config.editor_width);
            config.console_height = at_least("console", config.console_height);
            config.help_width = at_least("help", config.help_width);
        }
        if let Some(KdlValue::Integer(level)) = document.get("speed").and_then(|n| n.get(0)) {
            config.speed = (*level).clamp(1, 11) as u8;
        }
        if let Some(node) = document.get("help") {
            let help = &mut config.help;
            help.open = boolean(node, "open").unwrap_or(help.open);
            help.pinned = boolean(node, "pinned").unwrap_or(help.pinned);
            if let Some(tab) = string(node, "tab").and_then(Tab::named) {
                help.tab = tab;
            }
            help.page = string(node, "page")
                .filter(|p| !p.is_empty())
                .map(str::to_string);
            help.search = string(node, "search").unwrap_or_default().to_string();
            help.lesson = number(node, "lesson").unwrap_or(0.0).max(0.0) as usize;
            help.scroll = number(node, "scroll").unwrap_or(0.0).max(0.0);
        }
        Ok(config)
    }

    /// `existing`, or a new file, saying what's remembered in this. Only the
    /// remembered nodes change. `show-obsolete` is the file's to say.
    pub fn merged_into(&self, existing: Option<&str>) -> Result<String, String> {
        let mut document = document_of(existing)?;
        self.set_last_folder(&mut document);
        let whole = |n: f64| KdlValue::Integer(n.round() as i128);
        let text = |s: &str| KdlValue::String(s.to_string());
        let help = &self.help;
        set(
            &mut document,
            "window",
            [
                ("width", whole(self.window.0)),
                ("height", whole(self.window.1)),
                ("state", text(self.window_state.name())),
            ],
        );
        set(
            &mut document,
            "panes",
            [
                ("editor", whole(self.editor_width)),
                ("console", whole(self.console_height)),
                ("help", whole(self.help_width)),
            ],
        );
        node(&mut document, "speed", |n| {
            n.entries_mut().clear();
            n.entries_mut().push(KdlEntry::new(self.speed as i128));
        });
        set(
            &mut document,
            "help",
            [
                ("open", KdlValue::Bool(help.open)),
                ("tab", text(help.tab.name())),
                ("page", text(help.page.as_deref().unwrap_or(""))),
                ("search", text(&help.search)),
                ("pinned", KdlValue::Bool(help.pinned)),
                ("lesson", whole(help.lesson as f64)),
                ("scroll", whole(help.scroll)),
            ],
        );
        Ok(document.to_string())
    }

    /// `existing`, or a new file, with only the last folder changed.
    fn folder_merged_into(&self, existing: Option<&str>) -> Result<String, String> {
        let mut document = document_of(existing)?;
        self.set_last_folder(&mut document);
        Ok(document.to_string())
    }

    fn set_last_folder(&self, document: &mut KdlDocument) {
        node(document, "last-folder", |n| {
            let folder = self.last_folder.as_deref().map(Path::to_string_lossy);
            n.entries_mut().clear();
            n.entries_mut()
                .push(KdlEntry::new(folder.as_deref().unwrap_or("").to_string()));
        });
    }

    /// The folder to start a file dialog in, if it's still there.
    pub fn last_folder(&self) -> Option<PathBuf> {
        self.last_folder.clone().filter(|folder| folder.is_dir())
    }

    /// `file` was just opened or saved: next time, start in its folder. It's
    /// kept in full, as Pendown may be started from somewhere else next time.
    pub fn remember_folder(&mut self, file: &Path) {
        if let Some(folder) = std::path::absolute(file)
            .ok()
            .and_then(|file| file.parent().map(Path::to_path_buf))
        {
            self.last_folder = Some(folder);
        }
    }

    /// Cuts the panes down to leave room in a window this size, which may be
    /// smaller than when they were sized, or the file may have been edited.
    pub fn fit_panes(&mut self, window: (f64, f64)) {
        (self.editor_width, self.console_height, self.help_width) = self.panes_fitted(window);
    }

    fn panes_fitted(&self, window: (f64, f64)) -> (f64, f64, f64) {
        let fit = |size: f64, room: f64| size.min(room.max(MIN_PANE));
        let editor = fit(self.editor_width, window.0 - MIN_LEFT - MIN_PANE);
        let help = fit(self.help_width, window.0 - MIN_LEFT - editor);
        let console = fit(self.console_height, window.1 - MIN_LEFT);
        (editor, console, help)
    }
}

/// Finds the size to come back to from maximized or fullscreen. Some window
/// systems say a window has its new, bigger size before saying it's maximized,
/// so that can look like an ordinary resize. X11 does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowedSize {
    latest: (f64, f64),
    before: (f64, f64),
}

impl WindowedSize {
    pub fn new(size: (f64, f64)) -> WindowedSize {
        WindowedSize {
            latest: size,
            before: size,
        }
    }

    /// The window has this size, and doesn't say it's maximized or fullscreen.
    pub fn resized(&mut self, size: (f64, f64)) {
        if size != self.latest {
            self.before = self.latest;
            self.latest = size;
        }
    }

    /// The size to come back to, from a window maximized or fullscreen at
    /// `filled`.
    pub fn restored(&self, filled: (f64, f64)) -> (f64, f64) {
        if self.latest == filled {
            self.before
        } else {
            self.latest
        }
    }
}

/// The file's text, or a new file's if it's empty or missing, to change.
fn document_of(existing: Option<&str>) -> Result<KdlDocument, String> {
    existing
        .filter(|text| !text.trim().is_empty())
        .unwrap_or(TEMPLATE)
        .parse()
        .map_err(|e: kdl::KdlError| e.to_string())
}

/// The node called `name`, added if it's not there, for `f` to change.
fn node(document: &mut KdlDocument, name: &str, f: impl FnOnce(&mut KdlNode)) {
    if document.get(name).is_none() {
        let mut new = KdlNode::new(name);
        new.autoformat();
        document.nodes_mut().push(new);
    }
    f(document.get_mut(name).expect("just added"));
}

/// Gives the node called `name` these properties and no others.
fn set<const N: usize>(document: &mut KdlDocument, name: &str, properties: [(&str, KdlValue); N]) {
    node(document, name, |n| {
        n.entries_mut().clear();
        for (key, value) in properties {
            n.entries_mut().push(KdlEntry::new_prop(key, value));
        }
    });
}

/// The file settings are kept in, if any.
#[derive(Clone, Debug, PartialEq)]
pub struct ConfigFile {
    path: Option<PathBuf>,
    /// The file that used to hold the last folder. Its folder moves to
    /// `config.kdl`, and it goes once that's been saved.
    old_folder_file: Option<PathBuf>,
}

impl ConfigFile {
    /// `config.kdl` in Pendown's settings folder.
    pub fn usual() -> ConfigFile {
        let dir = settings_dir();
        ConfigFile {
            path: dir.as_ref().map(|d| d.join("config.kdl")),
            old_folder_file: dir.map(|d| d.join("last-folder")),
        }
    }

    pub fn at(path: impl Into<PathBuf>) -> ConfigFile {
        ConfigFile {
            path: Some(path.into()),
            old_folder_file: None,
        }
    }

    /// None: the defaults are used, and nothing's saved.
    pub fn none() -> ConfigFile {
        ConfigFile {
            path: None,
            old_folder_file: None,
        }
    }

    /// What the file says. If it's missing that's the defaults, and if it
    /// isn't KDL, the defaults too, with a warning.
    pub fn load(&self) -> Config {
        let text = self
            .path
            .as_deref()
            .and_then(|p| std::fs::read_to_string(p).ok());
        let mut config = text.map_or_else(Config::default, |text| {
            Config::parse(&text).unwrap_or_else(|e| {
                log::warn!("{}: {e}: using the default settings", self.describe());
                Config::default()
            })
        });
        if config.last_folder.is_none() {
            config.last_folder = self.old_folder();
        }
        config
    }

    /// The old file's folder. One that isn't a full path can't be trusted,
    /// since it depended on where Pendown was started.
    fn old_folder(&self) -> Option<PathBuf> {
        let saved = std::fs::read_to_string(self.old_folder_file.as_ref()?).ok()?;
        Some(PathBuf::from(saved.trim())).filter(|folder| folder.is_absolute())
    }

    /// Saves what's remembered. It reads the file again first, to keep what's
    /// been edited since Pendown started. A file that's no longer KDL is left
    /// alone rather than lost: fixing it is all it needs.
    pub fn save(&self, config: &Config) {
        self.change(|existing| config.merged_into(existing));
    }

    /// Saves only the last folder, as soon as it changes, so that everything
    /// else can wait until the window closes.
    pub fn save_last_folder(&self, config: &Config) {
        self.change(|existing| config.folder_merged_into(existing));
    }

    /// Replaces the file with what `merged` makes of it.
    fn change(&self, merged: impl FnOnce(Option<&str>) -> Result<String, String>) {
        let Some(path) = &self.path else { return };
        let existing = std::fs::read_to_string(path).ok();
        let saved = merged(existing.as_deref())
            .and_then(|text| write(path, &text).map_err(|e| e.to_string()));
        match (saved, &self.old_folder_file) {
            (Err(e), _) => log::warn!("{}: {e}: not saving the settings", self.describe()),
            (Ok(()), Some(old)) => {
                let _ = std::fs::remove_file(old);
            }
            (Ok(()), None) => {}
        }
    }

    fn describe(&self) -> String {
        self.path
            .as_deref()
            .map_or("config".into(), |p| p.display().to_string())
    }
}

/// Writes through a temporary file, so a crash can't leave half a file.
fn write(path: &Path, text: &str) -> std::io::Result<()> {
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder)?;
    }
    let temporary = path.with_extension("kdl.new");
    std::fs::write(&temporary, text)?;
    std::fs::rename(&temporary, path)
}

/// What the command line asks for.
#[derive(Clone, Debug, PartialEq)]
pub enum Args {
    /// Which settings file was asked for, if the command line said.
    Run(Option<ConfigFile>),
    Help,
}

/// Reads the command line, without the program's name.
pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut file = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let chosen = match arg.as_str() {
            "--help" | "-h" => return Ok(Args::Help),
            "--no-config" => ConfigFile::none(),
            "--config" => ConfigFile::at(args.next().ok_or("--config needs a file")?),
            other => match other.strip_prefix("--config=") {
                Some(path) if !path.is_empty() => ConfigFile::at(path),
                _ => return Err(format!("unknown option: {other}")),
            },
        };
        if file.replace(chosen).is_some() {
            return Err("choose one of --config and --no-config".into());
        }
    }
    Ok(Args::Run(file))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remembered() -> Config {
        Config {
            show_obsolete: false,
            last_folder: Some(r#"/home/me/my "best" \ drawings"#.into()),
            window: (1000.0, 700.0),
            window_state: WindowState::Maximized,
            editor_width: 300.0,
            console_height: 250.0,
            help_width: 420.0,
            speed: 8,
            help: HelpConfig {
                open: true,
                tab: Tab::Tutorial,
                page: Some("forward".into()),
                search: "circ \"le".into(),
                pinned: true,
                lesson: 3,
                scroll: 20.0,
            },
        }
    }

    #[test]
    fn a_missing_setting_is_the_default() {
        assert_eq!(Config::parse("").unwrap(), Config::default());
    }

    #[test]
    fn what_is_saved_comes_back() {
        let config = remembered();
        let text = config.merged_into(None).unwrap();
        assert_eq!(Config::parse(&text).unwrap(), config);
    }

    #[test]
    fn a_new_file_says_what_its_settings_are() {
        let text = Config::default().merged_into(None).unwrap();
        assert!(text.contains("// List the UCBLogo commands"), "{text}");
        assert!(text.contains("show-obsolete #false"), "{text}");
        assert!(
            text.contains("\nwindow width=1400 height=900 state=windowed\n"),
            "{text}"
        );
    }

    #[test]
    fn saving_keeps_what_was_edited() {
        let edited = "// Mine.\nshow-obsolete #true // I like them\nspeed 3\nmy-own 1\n";
        let text = remembered().merged_into(Some(edited)).unwrap();
        assert!(text.contains("// Mine.\n"), "{text}");
        assert!(
            text.contains("show-obsolete #true // I like them"),
            "{text}"
        );
        assert!(text.contains("my-own 1"), "{text}");
        assert!(text.contains("speed 8"), "{text}");
        assert!(!text.contains("speed 3"), "{text}");
        let config = Config::parse(&text).unwrap();
        assert!(config.show_obsolete);
        assert_eq!(config.editor_width, 300.0);
    }

    #[test]
    fn saving_twice_changes_nothing() {
        let once = remembered().merged_into(None).unwrap();
        let twice = remembered().merged_into(Some(&once)).unwrap();
        assert_eq!(once, twice);
    }

    #[test]
    fn a_file_that_is_not_kdl_is_not_saved_over() {
        assert!(Config::parse("window {").is_err());
        assert!(Config::default().merged_into(Some("window {")).is_err());
    }

    #[test]
    fn wrong_values_are_ignored_or_limited() {
        let config = Config::parse(
            "window width=\"wide\" height=10 state=\"sideways\"\n\
             panes editor=-5 console=#true\n\
             speed 99\n\
             help tab=\"nowhere\" lesson=-2 scroll=-100\n",
        )
        .unwrap();
        assert_eq!(config.window, (1400.0, MIN_WINDOW.1));
        assert_eq!(config.window_state, WindowState::Windowed);
        assert_eq!(config.editor_width, MIN_PANE);
        assert_eq!(config.console_height, 190.0);
        assert_eq!(config.speed, 11);
        assert_eq!(config.help.tab, Tab::Help);
        assert_eq!(config.help.lesson, 0);
        assert_eq!(config.help.scroll, 0.0);
    }

    #[test]
    fn panes_leave_room_in_a_small_window() {
        let config = Config {
            editor_width: 2000.0,
            console_height: 2000.0,
            help_width: 2000.0,
            ..Config::default()
        };
        let (editor, console, help) = config.panes_fitted((1000.0, 700.0));
        assert_eq!((editor, console), (760.0, 540.0));
        assert_eq!(help, 80.0);
        assert!(editor + help + MIN_LEFT <= 1000.0);
        // Sizes that fit are left alone.
        assert_eq!(
            Config::default().panes_fitted((1400.0, 900.0)),
            (460.0, 190.0, 380.0)
        );
    }

    #[test]
    fn the_last_folder_is_kept_in_full() {
        let mut config = Config::default();
        assert_eq!(config.last_folder(), None);

        let here = std::env::current_dir().unwrap();
        config.remember_folder(Path::new("flower.logo"));
        assert_eq!(config.last_folder(), Some(here.clone()));
        let text = config.merged_into(None).unwrap();
        assert_eq!(Config::parse(&text).unwrap().last_folder(), Some(here));

        // A folder that's gone is no use.
        config.last_folder = Some("/no/such/folder".into());
        assert_eq!(config.last_folder(), None);
    }

    #[test]
    fn saving_the_last_folder_changes_nothing_else() {
        let folder =
            std::env::temp_dir().join(format!("pendown-config-folder-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("config.kdl");
        let file = ConfigFile::at(&path);

        // The window's size and so on wait until the window closes.
        std::fs::write(&path, "// Mine.\nspeed 3 // slow\n").unwrap();
        let mut config = remembered();
        config.remember_folder(&folder.join("flower.logo"));
        file.save_last_folder(&config);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("// Mine.\nspeed 3 // slow\n"), "{text}");
        let saved = Config::parse(&text).unwrap();
        assert_eq!(saved.last_folder(), Some(folder.clone()));
        assert_eq!(saved.speed, 3);
        assert_eq!(saved.window, Config::default().window);

        // With no file yet, it makes one that reads as the defaults.
        std::fs::remove_file(&path).unwrap();
        file.save_last_folder(&config);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("show-obsolete #false"), "{text}");
        let saved = Config::parse(&text).unwrap();
        assert_eq!(saved.last_folder(), Some(folder.clone()));
        assert_eq!(saved.speed, Config::default().speed);

        // A broken file is still left alone.
        std::fs::write(&path, "window {").unwrap();
        file.save_last_folder(&config);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "window {");
        std::fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn the_old_last_folder_file_moves_in() {
        let folder =
            std::env::temp_dir().join(format!("pendown-config-old-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        let old = folder.join("last-folder");
        let file = ConfigFile {
            path: Some(folder.join("config.kdl")),
            old_folder_file: Some(old.clone()),
        };

        // A relative path, as the scripted session used to leave, isn't kept.
        std::fs::write(&old, "target/autotest").unwrap();
        assert_eq!(file.load().last_folder, None);

        std::fs::write(&old, format!("{}\n", folder.display())).unwrap();
        let config = file.load();
        assert_eq!(config.last_folder(), Some(folder.clone()));
        assert!(old.exists(), "loading doesn't remove it");

        file.save(&config);
        assert!(!old.exists(), "saved, so it's not needed");
        assert_eq!(file.load().last_folder(), Some(folder.clone()));
        std::fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn the_size_to_come_back_to() {
        let (small, big, filled) = ((1000.0, 700.0), (1200.0, 800.0), (1900.0, 1167.0));
        let mut size = WindowedSize::new(small);
        size.resized(big);
        // Told it's maximized before its bounds change, or with them.
        assert_eq!(size.restored(filled), big);
        // Told its bounds change first, even more than once, as on X11.
        size.resized(filled);
        size.resized(filled);
        assert_eq!(size.restored(filled), big);
        // Back to a window again.
        size.resized(big);
        assert_eq!(size.restored(filled), big);
        // Started maximized: the size it was given is the one to come back to.
        let mut size = WindowedSize::new(small);
        size.resized(filled);
        assert_eq!(size.restored(filled), small);
    }

    #[test]
    fn a_file_is_saved_and_loaded() {
        let folder =
            std::env::temp_dir().join(format!("pendown-config-{}/deeper", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        let file = ConfigFile::at(folder.join("config.kdl"));
        assert_eq!(file.load(), Config::default(), "there's no file yet");
        file.save(&remembered());
        assert_eq!(file.load(), remembered());

        // A broken file gives the defaults, and survives a save.
        let broken = "window {";
        std::fs::write(folder.join("config.kdl"), broken).unwrap();
        assert_eq!(file.load(), Config::default());
        file.save(&remembered());
        assert_eq!(
            std::fs::read_to_string(folder.join("config.kdl")).unwrap(),
            broken
        );
        std::fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn no_file_keeps_nothing() {
        let file = ConfigFile::none();
        file.save(&remembered());
        assert_eq!(file.load(), Config::default());
    }

    #[test]
    fn the_command_line() {
        let args = |list: &[&str]| parse_args(list.iter().map(|a| a.to_string()));
        assert_eq!(args(&[]), Ok(Args::Run(None)));
        assert_eq!(
            args(&["--no-config"]),
            Ok(Args::Run(Some(ConfigFile::none())))
        );
        let at = Ok(Args::Run(Some(ConfigFile::at("a.kdl"))));
        assert_eq!(args(&["--config", "a.kdl"]), at);
        assert_eq!(args(&["--config=a.kdl"]), at);
        assert_eq!(args(&["--help"]), Ok(Args::Help));
        assert!(args(&["--config"]).is_err());
        assert!(args(&["--config="]).is_err());
        assert!(args(&["--no-config", "--config", "a.kdl"]).is_err());
        assert!(args(&["flower.logo"]).is_err());
    }
}
