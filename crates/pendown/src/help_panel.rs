//! The side panel (phase 5): Help, a reference for every command generated
//! from `pendown-logo.kdl`, and the Tutorial. It's drawn as a cached view,
//! so it only redraws when it changes, not on every animation frame.

use std::{
    collections::{HashMap, HashSet},
    ops::Range,
    sync::{Arc, Mutex, OnceLock},
};

use gpui_kit::component::{
    ActiveTheme, Disableable as _, Icon, Selectable as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};
use gpui_kit::{assets::IconName, prelude::FluentBuilder as _, *};
use pendown::{
    help::{PICTURE_SIZE, picture_options},
    markup::{self, Block, Inline},
};
use pendown_logo::reference::{Command, Kind, reference};

use crate::{app::App as PendownWindow, colours_page::ColoursPage};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tab {
    Help,
    Tutorial,
}

/// A help page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Page {
    Contents,
    /// A command or syntax entry, by its index in the reference.
    Command(usize),
}

/// An example's picture, drawn on a background thread when first shown.
#[derive(Clone)]
pub enum Picture {
    /// An SVG, and its pictures at the sizes it's been shown.
    Drawing(Arc<[u8]>, Rendered),
    /// It drew nothing, but printed this.
    Printed(String),
    Failed(String),
}

/// A program in the panel that runs on the canvas: a help example, or a
/// tutorial lesson's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Source {
    Example { command: usize, example: usize },
    Lesson { lesson: usize, program: usize },
}

impl Source {
    pub fn code(&self) -> String {
        match *self {
            Source::Example { command, example } => {
                reference().commands[command].examples[example].code.clone()
            }
            Source::Lesson { lesson, program } => pendown::tutorial::lessons()[lesson]
                .programs()
                .nth(program)
                .unwrap_or_default()
                .to_string(),
        }
    }

    /// What the banner over the canvas says it is.
    fn describe(&self) -> String {
        match *self {
            Source::Example { command, example } => {
                let command = &reference().commands[command];
                match &command.examples[example].title {
                    Some(title) => format!("“{title}”, from {}", command.title),
                    None => format!("From {}", command.title),
                }
            }
            Source::Lesson { lesson, .. } => format!(
                "From the lesson “{}”",
                pendown::tutorial::lessons()[lesson].title
            ),
        }
    }
}

/// The program that's running, highlighted in the panel (issue 06).
#[derive(Clone, Debug, PartialEq)]
pub struct Running {
    pub source: Source,
    /// The running instruction, in the program's code.
    pub highlight: Option<Range<usize>>,
    pub finished: bool,
}

pub struct HelpPanel {
    app: WeakEntity<PendownWindow>,
    pub tab: Tab,
    pub page: Page,
    back: Vec<Page>,
    forward: Vec<Page>,
    pub search: Entity<InputState>,
    pub show_obsolete: bool,
    /// Pin keeps the page while the editor's cursor moves.
    pub pinned: bool,
    /// What the editor's cursor is on, and which line, while the panel
    /// follows it.
    pub following: Option<(String, u32)>,
    /// Open chapters and sections of the contents, by key.
    expanded: HashSet<String>,
    pictures: HashMap<String, Picture>,
    pub running: Option<Running>,
    /// The tutorial's lesson, counting from 0.
    pub lesson: usize,
    /// Enter opened a search result: the search box is emptied when the
    /// panel next draws, which has the window it needs.
    clear_search: bool,
    /// The Colours page's picker and swatches, once it's been shown.
    pub colours: Option<Entity<ColoursPage>>,
    /// The user's program is running, so Copy to editor is off.
    program_running: bool,
    /// Scrolls the help page, for the scripted session.
    pub page_scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

/// Lines of `eg` without the manual's notes, such as `(library procedure)`.
fn usage_lines(usage: &str) -> Vec<String> {
    usage
        .lines()
        .map(|l| {
            let l = l.trim();
            ["(library procedure)", "(special form)", "(variable)"]
                .iter()
                .fold(l.to_string(), |l, note| l.replace(note, ""))
                .trim()
                .to_string()
        })
        .filter(|l| !l.is_empty())
        .collect()
}

/// How many commands a list of indexes holds that the panel lists.
fn listed(indexes: impl Iterator<Item = usize>, obsolete: bool) -> usize {
    let r = reference();
    indexes
        .filter(|&i| r.commands[i].implemented || (obsolete && r.commands[i].obsolete.is_some()))
        .count()
}

impl HelpPanel {
    pub fn new(
        app: WeakEntity<PendownWindow>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> HelpPanel {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search commands"));
        let subscriptions = vec![cx.subscribe_in(
            &search,
            window,
            |this, _, event: &InputEvent, _, cx| match event {
                InputEvent::Change => cx.notify(),
                InputEvent::PressEnter { .. } => this.open_first_result(cx),
                _ => {}
            },
        )];
        HelpPanel {
            app,
            tab: Tab::Help,
            page: Page::Contents,
            back: Vec::new(),
            forward: Vec::new(),
            search,
            show_obsolete: false,
            pinned: false,
            following: None,
            expanded: HashSet::new(),
            pictures: HashMap::new(),
            running: None,
            lesson: 0,
            clear_search: false,
            colours: None,
            program_running: false,
            page_scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    pub fn query(&self, cx: &App) -> String {
        self.search.read(cx).value().trim().to_string()
    }

    /// Goes to a page, so Back comes back here.
    pub fn go(&mut self, page: Page, window: &mut Window, cx: &mut Context<Self>) {
        self.tab = Tab::Help;
        if !self.query(cx).is_empty() {
            self.search.update(cx, |s, cx| s.set_value("", window, cx));
        }
        if page != self.page {
            self.back.push(std::mem::replace(&mut self.page, page));
            self.forward.clear();
            // The page is the one asked for, not what the cursor is on.
            self.following = None;
        }
        self.show_in_contents(page);
        cx.notify();
    }

    /// Opens the page for a name, alias or syntax key, if there is one.
    pub fn open_name(&mut self, name: &str, window: &mut Window, cx: &mut Context<Self>) -> bool {
        match reference().find_index(name) {
            Some(i) => {
                self.go(Page::Command(i), window, cx);
                true
            }
            None => false,
        }
    }

    fn open_first_result(&mut self, cx: &mut Context<Self>) {
        let first = reference()
            .search(&self.query(cx), self.show_obsolete)
            .first()
            .copied();
        if let Some(i) = first {
            self.back
                .push(std::mem::replace(&mut self.page, Page::Command(i)));
            self.forward.clear();
            self.show_in_contents(Page::Command(i));
            // The search is cleared when the window next has a chance.
            self.clear_search = true;
            cx.notify();
        }
    }

    /// Opens the contents at a page's chapter and section.
    fn show_in_contents(&mut self, page: Page) {
        if let Page::Command(i) = page {
            let command = &reference().commands[i];
            let chapter = &reference().chapters[command.chapter];
            self.expanded.insert(chapter.key.clone());
            if let Some(s) = command.section {
                self.expanded
                    .insert(format!("{}/{}", chapter.key, chapter.sections[s].key));
            }
        }
    }

    fn go_back(&mut self, cx: &mut Context<Self>) {
        if let Some(page) = self.back.pop() {
            self.forward.push(std::mem::replace(&mut self.page, page));
            self.pinned = true;
            cx.notify();
        }
    }

    fn go_forward(&mut self, cx: &mut Context<Self>) {
        if let Some(page) = self.forward.pop() {
            self.back.push(std::mem::replace(&mut self.page, page));
            self.pinned = true;
            cx.notify();
        }
    }

    /// The editor's cursor is on `name`, on `line`, counting from 0. Unless
    /// pinned or searching, the panel shows its page, without adding to the
    /// history.
    pub fn follow(&mut self, word: Option<(String, u32)>, cx: &mut Context<Self>) {
        // The Colours page stays while the cursor moves, since it copies to
        // wherever the cursor is.
        if self.pinned || self.tab != Tab::Help || !self.query(cx).is_empty() || self.on_colours() {
            return;
        }
        let Some((name, line)) = word else { return };
        let Some(i) = reference().find_index(&name) else {
            return;
        };
        let following = Some((name, line));
        if self.page != Page::Command(i) || self.following != following {
            if self.page != Page::Command(i) {
                self.page = Page::Command(i);
                self.show_in_contents(self.page);
            }
            self.following = following;
            cx.notify();
        }
    }

    /// F1: Help for the word under the cursor, following it from now on.
    pub fn help_for(
        &mut self,
        word: Option<(String, u32)>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.tab = Tab::Help;
        self.pinned = false;
        self.search.update(cx, |s, cx| s.set_value("", window, cx));
        if let Some((name, _)) = &word
            && let Some(i) = reference().find_index(name)
        {
            if self.page != Page::Command(i) {
                self.back
                    .push(std::mem::replace(&mut self.page, Page::Command(i)));
                self.forward.clear();
            }
            self.show_in_contents(self.page);
        }
        self.following = word;
        cx.notify();
    }

    /// The picture of an example, starting to draw it if it isn't yet.
    fn picture(&mut self, code: &str, cx: &mut Context<Self>) -> Option<Picture> {
        if let Some(picture) = self.pictures.get(code) {
            return Some(picture.clone());
        }
        let key = code.to_string();
        // Drawing it can take a moment: in the meantime, the panel shows a
        // blank square.
        self.pictures
            .insert(key.clone(), Picture::Printed(String::new()));
        let code = key.clone();
        let task =
            cx.background_spawn(async move { pendown_cli::render(&code, picture_options()) });
        cx.spawn(async move |this, cx| {
            let output = task.await;
            let picture = match &output.result {
                Err(e) if output.drawn == 0 => Picture::Failed(format!("{e:?}")),
                _ if output.drawn == 0 => Picture::Printed(output.printed.clone()),
                _ => Picture::Drawing(
                    Arc::from(output.svg.expect("asked for").into_bytes()),
                    Rendered::default(),
                ),
            };
            this.update(cx, |this, cx| {
                this.pictures.insert(key, picture);
                cx.notify();
            })
            .ok();
        })
        .detach();
        None
    }

    /// Whether the user's program is running, which turns Copy to editor
    /// off on the Colours page.
    pub fn set_program_running(&mut self, running: bool, cx: &mut Context<Self>) {
        if self.program_running != running {
            self.program_running = running;
            cx.notify();
        }
    }

    /// Whether the page shows the Colours entry.
    fn on_colours(&self) -> bool {
        matches!(self.page, Page::Command(i) if reference().commands[i].key == "colours")
    }

    /// Makes the Colours page's picker the first time it's shown, at Kame's
    /// pen colour.
    fn ensure_colours(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.colours.is_some() || !self.on_colours() {
            return;
        }
        let pen = self
            .app
            .upgrade()
            .and_then(|app| {
                let watch = app.read(cx).watched.read(cx).watch.clone()?;
                watch.colours.iter().find(|c| c.name == "PENCOLOR")?.rgba
            })
            .unwrap_or([255, 255, 255, 255]);
        let app = self.app.clone();
        let page = cx.new(|cx| ColoursPage::new(app, pen, window, cx));
        self._subscriptions
            .push(cx.observe(&page, |_, _, cx| cx.notify()));
        self.colours = Some(page);
    }

    pub fn set_running(&mut self, running: Option<Running>, cx: &mut Context<Self>) {
        if self.running != running {
            self.running = running;
            cx.notify();
        }
    }
}

/// A tab at the top of the panel.
fn tab(id: &'static str, label: &'static str, active: bool, cx: &App) -> Stateful<Div> {
    let theme = cx.theme();
    div()
        .id(id)
        .px_3()
        .h_full()
        .flex()
        .items_center()
        .text_sm()
        .cursor_pointer()
        .border_b_2()
        .when(active, |t| {
            t.border_color(theme.foreground)
                .text_color(theme.foreground)
                .font_weight(FontWeight::SEMIBOLD)
        })
        .when(!active, |t| {
            t.border_color(transparent_black())
                .text_color(theme.muted_foreground)
        })
        .child(label)
}

/// Pictures of an SVG already rendered, by their size in device pixels.
pub(crate) type Rendered = Arc<Mutex<HashMap<(i32, i32), Arc<RenderImage>>>>;

/// An SVG, `width` by `height` logical pixels. It's rendered at exactly
/// that size in device pixels, once for each size, in `rendered`: an SVG
/// image is otherwise rendered at its own size and shrunk on the GPU,
/// which leaves it jagged or blurred.
pub(crate) fn svg_image(svg: Arc<[u8]>, rendered: Rendered, width: f32, height: f32) -> Img {
    let source = ImageSource::Custom(Arc::new(move |window: &mut Window, cx: &mut App| {
        let scale = window.scale_factor();
        let device = (
            (width * scale).round() as i32,
            (height * scale).round() as i32,
        );
        if let Some(image) = rendered.lock().unwrap().get(&device) {
            return Some(Ok(image.clone()));
        }
        let renderer = cx.svg_renderer();
        let image = renderer
            .parse_svg(&svg)
            .and_then(|tree| {
                renderer.render_parsed(
                    &tree,
                    SvgSize::Size(size(DevicePixels(device.0), DevicePixels(device.1))),
                )
            })
            .map_err(|e| ImageCacheError::Asset(e.to_string().into()));
        if let Ok(image) = &image {
            rendered.lock().unwrap().insert(device, image.clone());
        }
        Some(image)
    }));
    img(source).w(px(width)).h(px(height)).flex_none()
}

/// Kame's picture, `width` by `height` logical pixels.
pub(crate) fn kame(width: f32, height: f32) -> impl IntoElement {
    static KAME: OnceLock<(Arc<[u8]>, Rendered)> = OnceLock::new();
    let (svg, rendered) =
        KAME.get_or_init(|| (Arc::from(pendown::tutorial::KAME), Rendered::default()));
    svg_image(svg.clone(), rendered.clone(), width, height)
}

/// A heading for a part of a page, such as Examples.
fn part(text: &'static str, cx: &App) -> Div {
    div()
        .mt_4()
        .mb_2()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(cx.theme().muted_foreground)
        .child(text.to_uppercase())
}

impl HelpPanel {
    /// Text with markup, its links clickable.
    fn rich(&self, id: SharedString, inlines: &[Inline], cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let mono = theme.mono_font_family.clone();
        let mut text = String::new();
        let mut highlights = Vec::new();
        let mut fonts = Vec::new();
        let mut links: Vec<Range<usize>> = Vec::new();
        let mut targets: Vec<String> = Vec::new();
        for inline in inlines {
            let start = text.len();
            match inline {
                Inline::Text(t) => text.push_str(t),
                Inline::Code(t) => {
                    text.push_str(t);
                    fonts.push((start..text.len(), mono.clone()));
                    highlights.push((
                        start..text.len(),
                        HighlightStyle {
                            background_color: Some(theme.muted),
                            ..Default::default()
                        },
                    ));
                }
                Inline::Bold(t) => {
                    text.push_str(t);
                    highlights.push((
                        start..text.len(),
                        HighlightStyle {
                            font_weight: Some(FontWeight::BOLD),
                            ..Default::default()
                        },
                    ));
                }
                Inline::Italic(t) => {
                    text.push_str(t);
                    highlights.push((
                        start..text.len(),
                        HighlightStyle {
                            font_style: Some(FontStyle::Italic),
                            ..Default::default()
                        },
                    ));
                }
                Inline::Link(name) => {
                    text.push_str(name);
                    let is_command = reference()
                        .find(name)
                        .is_some_and(|c| c.kind != Kind::Syntax);
                    if is_command {
                        fonts.push((start..text.len(), mono.clone()));
                    }
                    highlights.push((
                        start..text.len(),
                        HighlightStyle {
                            color: Some(theme.blue),
                            ..Default::default()
                        },
                    ));
                    links.push(start..text.len());
                    targets.push(name.clone());
                }
            }
        }
        let panel = cx.entity().downgrade();
        InteractiveText::new(
            id,
            StyledText::new(text)
                .with_highlights(highlights)
                .with_font_family_overrides(fonts),
        )
        .on_click(links, move |ix, window, cx| {
            let name = targets[ix].clone();
            panel
                .update(cx, |p, cx| p.open_name(&name, window, cx))
                .ok();
        })
        .into_any_element()
    }

    /// Blocks of markup.
    fn blocks(&self, id: &str, blocks: &[Block], cx: &mut Context<Self>) -> Vec<AnyElement> {
        let theme = cx.theme().clone();
        blocks
            .iter()
            .enumerate()
            .map(|(b, block)| {
                let id = |n: usize| SharedString::from(format!("{id}-{b}-{n}"));
                match block {
                    Block::Paragraph(inlines) => div().mt_2().child(self.rich(id(0), inlines, cx)).into_any_element(),
                    Block::Heading(_, inlines) => {
                        div().mt_4().text_base().font_weight(FontWeight::SEMIBOLD).child(self.rich(id(0), inlines, cx)).into_any_element()
                    }
                    Block::Note(inlines) => {
                        let try_this = matches!(inlines.first(), Some(Inline::Bold(t)) if t.starts_with("Try this"));
                        h_flex()
                            .mt_3()
                            .p_2()
                            .gap_2()
                            .items_start()
                            .rounded_md()
                            .bg(theme.success.opacity(0.12))
                            .when(try_this, |n| n.child(Icon::new(IconName::PencilLine).small().mt(px(3.)).flex_none()))
                            .child(div().flex_1().min_w_0().child(self.rich(id(0), inlines, cx)))
                            .into_any_element()
                    }
                    Block::List(items) => v_flex()
                        .mt_2()
                        .gap_1()
                        .children(items.iter().enumerate().map(|(n, inlines)| {
                            h_flex().items_start().gap_2().child(div().child("•")).child(div().flex_1().min_w_0().child(self.rich(id(n), inlines, cx)))
                        }))
                        .into_any_element(),
                    Block::Code { code, .. } => code_block(code, None, &theme).mt_2().into_any_element(),
                }
            })
            .collect()
    }
}

/// A program, in the editor's font and colours, with the running
/// instruction highlighted.
pub(crate) fn code_block(
    code: &str,
    highlight: Option<Range<usize>>,
    theme: &gpui_kit::component::Theme,
) -> Div {
    let running = HighlightStyle {
        background_color: Some(theme.warning.opacity(0.35)),
        ..Default::default()
    };
    let styles = crate::highlighter::program_styles(
        code,
        &*theme.highlight_theme,
        highlight.map(|r| (r, running)),
    );
    let text = StyledText::new(code.to_string()).with_highlights(styles);
    div()
        .p_2()
        .rounded_md()
        .bg(theme.muted)
        .font_family(theme.mono_font_family.clone())
        .text_xs()
        .whitespace_normal()
        .child(text)
}

impl HelpPanel {
    fn render_command(&mut self, i: usize, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let theme = cx.theme().clone();
        let r = reference();
        let command: &Command = &r.commands[i];
        let chapter = &r.chapters[command.chapter];
        let mut out = Vec::new();

        // Where it is.
        let mut crumbs = h_flex()
            .gap_1()
            .text_xs()
            .text_color(theme.muted_foreground);
        let chapter_key = chapter.key.clone();
        crumbs = crumbs.child(
            div()
                .id("crumb-chapter")
                .cursor_pointer()
                .text_color(theme.blue)
                .child(chapter.title.clone())
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.expanded.insert(chapter_key.clone());
                    this.go(Page::Contents, window, cx);
                })),
        );
        if let Some(s) = command.section {
            let key = format!("{}/{}", chapter.key, chapter.sections[s].key);
            let chapter_key = chapter.key.clone();
            crumbs = crumbs.child("›").child(
                div()
                    .id("crumb-section")
                    .cursor_pointer()
                    .text_color(theme.blue)
                    .child(chapter.sections[s].title.clone())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.expanded.insert(chapter_key.clone());
                        this.expanded.insert(key.clone());
                        this.go(Page::Contents, window, cx);
                    })),
            );
        }
        out.push(crumbs.into_any_element());

        // Its names.
        let kind = match command.kind {
            Kind::Variable => Some("special variable"),
            Kind::Syntax => Some("syntax"),
            Kind::Command => None,
        };
        let title = h_flex()
            .mt_1()
            .gap_2()
            .items_center()
            .flex_wrap()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .when(command.kind != Kind::Syntax, |d| {
                        d.font_family(theme.mono_font_family.clone())
                    })
                    .child(command.title.clone()),
            )
            .children(
                command
                    .names
                    .iter()
                    .skip(if command.kind == Kind::Syntax { 0 } else { 1 })
                    // A syntax entry's words, such as COLOR, are only for
                    // search: its badges are what the cursor can be on.
                    .filter(|alias| {
                        command.kind != Kind::Syntax || !alias.chars().any(char::is_alphabetic)
                    })
                    .map(|alias| {
                        div()
                            .px_1()
                            .rounded_sm()
                            .border_1()
                            .border_color(theme.border)
                            .text_xs()
                            .font_family(theme.mono_font_family.clone())
                            .text_color(theme.muted_foreground)
                            .child(alias.clone())
                    }),
            )
            .when(command.extension, |t| {
                t.child(div().text_xs().text_color(theme.primary).child("Pendown"))
            })
            .when_some(kind, |t, kind| {
                t.child(div().flex_1()).child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(kind),
                )
            });
        out.push(title.into_any_element());

        if let Some(reason) = &command.obsolete
            && !command.implemented
        {
            out.push(
                div()
                    .mt_3()
                    .p_2()
                    .rounded_md()
                    .bg(theme.danger.opacity(0.1))
                    .child(format!("Not in Pendown: {reason}"))
                    .into_any_element(),
            );
        } else if !command.implemented {
            out.push(
                div()
                    .mt_3()
                    .p_2()
                    .rounded_md()
                    .bg(theme.muted)
                    .child("Not in Pendown yet.")
                    .into_any_element(),
            );
        }

        // How to call it.
        let usage = usage_lines(&command.usage);
        if !usage.is_empty() {
            out.push(
                v_flex()
                    .mt_3()
                    .p_2()
                    .rounded_md()
                    .bg(theme.muted)
                    .font_family(theme.mono_font_family.clone())
                    .text_xs()
                    .children(usage.into_iter().map(|l| div().child(l)))
                    .into_any_element(),
            );
        }

        // What it does: ours, or the manual's for a command we don't have.
        match &command.help {
            Some(help) => out.extend(self.blocks("help", &markup::parse(help), cx)),
            // An obsolete command's reason says enough.
            None if !command.desc.is_empty() && command.obsolete.is_none() => out.push(
                div()
                    .mt_2()
                    .text_color(theme.muted_foreground)
                    .child(command.desc.clone())
                    .into_any_element(),
            ),
            None => {}
        }

        // The Colours page's forms, picker and swatches.
        if command.key == "colours"
            && let Some(page) = self.colours.clone()
        {
            out.push(part("The forms", cx).into_any_element());
            out.push(page.update(cx, |page, cx| page.forms_table(cx)));
            out.push(part("Find a colour", cx).into_any_element());
            out.push(page.update(cx, |page, cx| page.picker(cx)));
            out.push(part("Palette", cx).into_any_element());
            out.push(page.update(cx, |page, cx| page.palette(cx)));
            out.extend(self.blocks(
                "colours-palette",
                &markup::parse("[SETPALETTE] can change 8 to 15. Run puts them back."),
                cx,
            ));
            out.push(part("Names", cx).into_any_element());
            out.push(page.update(cx, |page, cx| page.names(cx)));
        }

        if !command.examples.is_empty() {
            out.push(part("Examples", cx).into_any_element());
            for n in 0..command.examples.len() {
                out.push(self.example_card(i, n, cx));
            }
        }

        let see = r.see_also(i);
        if !see.is_empty() {
            out.push(part("See also", cx).into_any_element());
            out.push(
                h_flex()
                    .flex_wrap()
                    .gap_x_3()
                    .gap_y_1()
                    .children(see.into_iter().map(|j| {
                        let target = &r.commands[j];
                        div()
                            .id(SharedString::from(format!("see-{j}")))
                            .cursor_pointer()
                            .text_color(theme.blue)
                            .when(target.kind != Kind::Syntax, |d| {
                                d.font_family(theme.mono_font_family.clone()).text_xs()
                            })
                            .child(target.title.clone())
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.go(Page::Command(j), window, cx)
                            }))
                    }))
                    .into_any_element(),
            );
        }
        out
    }

    fn example_card(&mut self, i: usize, n: usize, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let example = &reference().commands[i].examples[n];
        let picture = self.picture(&example.code, cx);
        let running = self.running.as_ref().filter(|r| {
            r.source
                == Source::Example {
                    command: i,
                    example: n,
                }
        });
        let highlight = running.and_then(|r| r.highlight.clone());
        let is_running = running.is_some_and(|r| !r.finished);
        let code = example.code.clone();
        let shown = PICTURE_SIZE as f32;
        let size = px(shown);
        let picture_element = match &picture {
            Some(Picture::Drawing(svg, rendered)) => Some(
                svg_image(svg.clone(), rendered.clone(), shown, shown)
                    .rounded_sm()
                    .into_any_element(),
            ),
            None => Some(
                div()
                    .size(size)
                    .flex_none()
                    .rounded_sm()
                    .bg(black())
                    .into_any_element(),
            ),
            _ => None,
        };
        let printed = match &picture {
            Some(Picture::Printed(text)) if !text.is_empty() => Some(text.clone()),
            Some(Picture::Failed(text)) => Some(text.clone()),
            _ => None,
        };
        let app = self.app.clone();
        let app_copy = self.app.clone();
        let copy_code = code.clone();
        v_flex()
            .border_1()
            .border_color(if is_running {
                theme.foreground
            } else {
                theme.border
            })
            .rounded_md()
            .mb_2()
            .child(
                v_flex()
                    .p_2()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                div().flex_1().font_weight(FontWeight::SEMIBOLD).child(
                                    example
                                        .title
                                        .clone()
                                        .unwrap_or_else(|| format!("Example {}", n + 1)),
                                ),
                            )
                            .when(example.fails, |h| {
                                h.child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .child("stops with an error"),
                                )
                            }),
                    )
                    .child(
                        h_flex()
                            .items_start()
                            .gap_2()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .child(code_block(&code, highlight, &theme)),
                            )
                            .children(picture_element),
                    )
                    .when_some(printed, |v, text| {
                        v.child(
                            v_flex()
                                .gap_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .child("Prints"),
                                )
                                .child(
                                    div()
                                        .p_2()
                                        .rounded_md()
                                        .border_1()
                                        .border_color(theme.border)
                                        .font_family(theme.mono_font_family.clone())
                                        .text_xs()
                                        .children(
                                            text.lines()
                                                .take(12)
                                                .map(|l| div().child(l.to_string())),
                                        ),
                                ),
                        )
                    }),
            )
            .child(
                h_flex()
                    .gap_2()
                    .p_2()
                    .border_t_1()
                    .border_color(theme.border)
                    .bg(theme.muted.opacity(0.4))
                    .child(if is_running {
                        Button::new(SharedString::from(format!("stop-{i}-{n}")))
                            .small()
                            .icon(IconName::Square)
                            .label("Stop")
                            .on_click(move |_, _, cx| {
                                app.update(cx, |this, cx| this.stop(cx)).ok();
                            })
                    } else {
                        Button::new(SharedString::from(format!("run-{i}-{n}")))
                            .small()
                            .icon(IconName::Play)
                            .label("Run")
                            .on_click(move |_, _, cx| {
                                app.update(cx, |this, cx| this.run_example(i, n, cx)).ok();
                            })
                    })
                    .child(
                        Button::new(SharedString::from(format!("copy-{i}-{n}")))
                            .small()
                            .ghost()
                            .label("Copy to editor")
                            .on_click(move |_, window, cx| {
                                app_copy
                                    .update(cx, |this, cx| {
                                        this.copy_to_editor(&copy_code, window, cx)
                                    })
                                    .ok();
                            }),
                    ),
            )
            .into_any_element()
    }

    fn render_contents(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let theme = cx.theme().clone();
        let r = reference();
        let obsolete = self.show_obsolete;
        let mut out = vec![part("Contents", cx).mt_1().into_any_element()];
        let row = |id: String, label: String, count: usize, open: bool, indent: f32| {
            h_flex()
                .id(SharedString::from(id))
                .pl(px(indent))
                .pr_2()
                .py_1()
                .gap_1()
                .rounded_md()
                .cursor_pointer()
                .hover(|s| s.bg(theme.muted))
                .child(
                    Icon::new(if open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .small()
                    .text_color(theme.muted_foreground),
                )
                .child(div().flex_1().child(label))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(count.to_string()),
                )
        };
        for chapter in &r.chapters {
            let count = listed(chapter.all(), obsolete);
            if count == 0 {
                continue;
            }
            let open = self.expanded.contains(&chapter.key);
            let key = chapter.key.clone();
            out.push(
                row(
                    format!("chapter-{}", chapter.key),
                    chapter.title.clone(),
                    count,
                    open,
                    4.,
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    if !this.expanded.remove(&key) {
                        this.expanded.insert(key.clone());
                    }
                    cx.notify();
                }))
                .into_any_element(),
            );
            if !open {
                continue;
            }
            out.extend(self.command_rows(&chapter.commands, 24., cx));
            for section in &chapter.sections {
                let count = listed(section.commands.iter().copied(), obsolete);
                if count == 0 {
                    continue;
                }
                let key = format!("{}/{}", chapter.key, section.key);
                let open = self.expanded.contains(&key);
                let toggle = key.clone();
                out.push(
                    row(
                        format!("section-{key}"),
                        section.title.clone(),
                        count,
                        open,
                        22.,
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if !this.expanded.remove(&toggle) {
                            this.expanded.insert(toggle.clone());
                        }
                        cx.notify();
                    }))
                    .into_any_element(),
                );
                if open {
                    out.extend(self.command_rows(&section.commands, 44., cx));
                }
            }
        }
        out
    }

    /// A row for each listed command, to open its page.
    fn command_rows(
        &self,
        indexes: &[usize],
        indent: f32,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let theme = cx.theme().clone();
        let r = reference();
        indexes
            .iter()
            .copied()
            .filter(|&i| {
                r.commands[i].implemented
                    || (self.show_obsolete && r.commands[i].obsolete.is_some())
            })
            .map(|i| {
                let command = &r.commands[i];
                let current = self.page == Page::Command(i);
                h_flex()
                    .id(SharedString::from(format!("command-{i}")))
                    .pl(px(indent))
                    .pr_2()
                    .py(px(3.))
                    .gap_2()
                    .rounded_md()
                    .cursor_pointer()
                    .when(current, |r| r.bg(theme.accent))
                    .hover(|s| s.bg(theme.muted))
                    .when(command.kind != Kind::Syntax, |r| {
                        r.font_family(theme.mono_font_family.clone()).text_xs()
                    })
                    .when(!command.implemented, |r| {
                        r.text_color(theme.muted_foreground)
                    })
                    .child(command.title.clone())
                    .when(command.kind != Kind::Syntax, |r| {
                        r.child(
                            div().text_color(theme.muted_foreground).child(
                                command
                                    .names
                                    .iter()
                                    .skip(1)
                                    .cloned()
                                    .collect::<Vec<_>>()
                                    .join(" "),
                            ),
                        )
                    })
                    .when(command.extension, |r| {
                        r.child(div().flex_1())
                            .child(div().text_color(theme.primary).child("Pendown"))
                    })
                    .on_click(
                        cx.listener(move |this, _, window, cx| {
                            this.go(Page::Command(i), window, cx)
                        }),
                    )
                    .into_any_element()
            })
            .collect()
    }

    fn render_results(&mut self, query: &str, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let theme = cx.theme().clone();
        let r = reference();
        let hits = r.search(query, self.show_obsolete);
        let (current, old): (Vec<usize>, Vec<usize>) =
            hits.into_iter().partition(|&i| r.commands[i].implemented);
        let mut out = Vec::new();
        if current.is_empty() && old.is_empty() {
            out.push(
                div()
                    .mt_2()
                    .text_color(theme.muted_foreground)
                    .child(format!("No name has {} in it.", query.to_uppercase()))
                    .into_any_element(),
            );
        }
        for (heading, group) in [("Commands", current), ("Obsolete", old)] {
            if group.is_empty() {
                continue;
            }
            out.push(part(heading, cx).into_any_element());
            for i in group {
                let command = &r.commands[i];
                let summary = command.help.as_deref().map(markup::summary).or_else(|| {
                    command
                        .obsolete
                        .clone()
                        .map(|r| format!("Not in Pendown: {r}"))
                });
                let place = match command.section {
                    Some(s) => r.chapters[command.chapter].sections[s].title.clone(),
                    None => r.chapters[command.chapter].title.clone(),
                };
                out.push(
                    v_flex()
                        .id(SharedString::from(format!("hit-{i}")))
                        .p_2()
                        .rounded_md()
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.muted))
                        .when(!command.implemented, |v| {
                            v.text_color(theme.muted_foreground)
                        })
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    div()
                                        .font_family(theme.mono_font_family.clone())
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(command.title.clone()),
                                )
                                .child(
                                    div()
                                        .font_family(theme.mono_font_family.clone())
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .child(
                                            command
                                                .names
                                                .iter()
                                                .skip(1)
                                                .cloned()
                                                .collect::<Vec<_>>()
                                                .join(" "),
                                        ),
                                )
                                .child(div().flex_1())
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .child(place),
                                ),
                        )
                        .when_some(summary, |v, s| {
                            v.child(
                                div()
                                    .mt_1()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(s),
                            )
                        })
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.go(Page::Command(i), window, cx)
                        }))
                        .into_any_element(),
                );
            }
        }
        out
    }
}

impl Render for HelpPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if std::mem::take(&mut self.clear_search) {
            self.search.update(cx, |s, cx| s.set_value("", window, cx));
        }
        self.ensure_colours(window, cx);
        let theme = cx.theme().clone();
        let query = self.query(cx);
        let app = self.app.clone();

        let tabs = h_flex()
            .h(px(36.))
            .flex_none()
            .px_2()
            .gap_1()
            .border_b_1()
            .border_color(theme.border)
            .child(
                tab("tab-help", "Help", self.tab == Tab::Help, cx).on_click(cx.listener(
                    |this, _, _, cx| {
                        this.tab = Tab::Help;
                        cx.notify();
                    },
                )),
            )
            .child(
                tab("tab-tutorial", "Tutorial", self.tab == Tab::Tutorial, cx).on_click(
                    cx.listener(|this, _, _, cx| {
                        this.tab = Tab::Tutorial;
                        cx.notify();
                    }),
                ),
            )
            .child(div().flex_1())
            .child(
                Button::new("close-help")
                    .ghost()
                    .small()
                    .icon(IconName::Close)
                    .tooltip("Close the panel")
                    .on_click(move |_, _, cx| {
                        app.update(cx, |this, cx| this.toggle_help(cx)).ok();
                    }),
            );

        let body = match self.tab {
            Tab::Help => self.render_help(&query, window, cx),
            Tab::Tutorial => self.render_tutorial(window, cx),
        };
        v_flex()
            .size_full()
            .bg(theme.background)
            .text_sm()
            .child(tabs)
            .child(body)
    }
}

impl HelpPanel {
    fn render_help(&mut self, query: &str, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let nav = h_flex()
            .flex_none()
            .gap_1()
            .p_2()
            .child(
                Button::new("help-back")
                    .ghost()
                    .small()
                    .icon(IconName::ArrowLeft)
                    .tooltip("Back")
                    .disabled(self.back.is_empty())
                    .on_click(cx.listener(|this, _, _, cx| this.go_back(cx))),
            )
            .child(
                Button::new("help-forward")
                    .ghost()
                    .small()
                    .icon(IconName::ArrowRight)
                    .tooltip("Forward")
                    .disabled(self.forward.is_empty())
                    .on_click(cx.listener(|this, _, _, cx| this.go_forward(cx))),
            )
            .child(
                Button::new("help-contents")
                    .ghost()
                    .small()
                    .icon(IconName::Menu)
                    .tooltip("Contents")
                    .selected(self.page == Page::Contents && query.is_empty())
                    .on_click(
                        cx.listener(|this, _, window, cx| this.go(Page::Contents, window, cx)),
                    ),
            )
            .child(
                div().flex_1().child(
                    Input::new(&self.search)
                        .small()
                        .prefix(Icon::new(IconName::Search).small())
                        .cleanable(true),
                ),
            );

        // What the page is following, or that it's pinned.
        let strip = match (&self.page, query.is_empty()) {
            (Page::Command(_), true) if !self.on_colours() => {
                let (text, button) = match (&self.following, self.pinned) {
                    (_, true) => (
                        "Pinned: the page stays as the cursor moves".to_string(),
                        "Follow",
                    ),
                    (Some((word, line)), false) => (
                        format!("Following the cursor: {word}, line {}", line + 1),
                        "Pin",
                    ),
                    (None, false) => ("Following the editor's cursor".to_string(), "Pin"),
                };
                Some(
                    h_flex()
                        .flex_none()
                        .mx_2()
                        .mb_2()
                        .px_2()
                        .py_1()
                        .gap_2()
                        .rounded_md()
                        .bg(theme.info.opacity(0.12))
                        .text_xs()
                        .child(div().flex_1().child(text))
                        .child(
                            Button::new("help-pin")
                                .ghost()
                                .xsmall()
                                .label(button)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.pinned = !this.pinned;
                                    cx.notify();
                                })),
                        ),
                )
            }
            _ => None,
        };

        let (id, content) = if !query.is_empty() {
            ("help-results".to_string(), self.render_results(query, cx))
        } else {
            match self.page {
                Page::Contents => ("help-contents".to_string(), self.render_contents(cx)),
                Page::Command(i) => (format!("help-page-{i}"), self.render_command(i, cx)),
            }
        };
        let count = reference()
            .commands
            .iter()
            .filter(|c| c.implemented && c.kind != Kind::Syntax)
            .count();
        let footer = h_flex()
            .flex_none()
            .px_3()
            .py_2()
            .gap_2()
            .border_t_1()
            .border_color(theme.border)
            .text_xs()
            .child(
                Checkbox::new("show-obsolete")
                    .label("Show obsolete commands")
                    .checked(self.show_obsolete)
                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                        this.show_obsolete = *checked;
                        cx.notify();
                    })),
            )
            .child(div().flex_1())
            .child(
                div()
                    .text_color(theme.muted_foreground)
                    .child(format!("{count} commands")),
            );

        v_flex()
            .flex_1()
            .min_h_0()
            .child(nav)
            .children(strip)
            .child(
                div()
                    .id(SharedString::from(id))
                    .track_scroll(&self.page_scroll)
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_3()
                    .pb_4()
                    .children(content),
            )
            .map(|el| match self.colours.clone() {
                // The current colour, always in view on the Colours page.
                Some(page) if query.is_empty() && self.on_colours() => {
                    let running = self.program_running;
                    el.child(page.update(cx, |page, cx| page.bar(running, cx)))
                }
                _ => el.child(footer),
            })
            .into_any_element()
    }

    fn render_tutorial(&mut self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let lessons = pendown::tutorial::lessons();
        let l = self.lesson.min(lessons.len() - 1);
        let lesson = &lessons[l];

        // Which lesson, and a bar to go to any of them. It shows where the
        // lesson is in the order, not what's been done: earlier lessons are
        // grey, not a "passed" green (ADR 0027).
        let progress = h_flex()
            .mt_2()
            .gap_1()
            .children((0..lessons.len()).map(|i| {
                let colour = match i.cmp(&l) {
                    std::cmp::Ordering::Less => theme.muted_foreground,
                    std::cmp::Ordering::Equal => theme.foreground,
                    std::cmp::Ordering::Greater => theme.border,
                };
                div()
                    .id(SharedString::from(format!("lesson-{i}")))
                    .flex_1()
                    .h(px(5.))
                    .rounded_sm()
                    .bg(colour)
                    .cursor_pointer()
                    .tooltip(move |window, cx| {
                        gpui_kit::component::tooltip::Tooltip::new(lessons[i].title.clone())
                            .build(window, cx)
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.lesson = i;
                        cx.notify();
                    }))
            }));
        let header = v_flex()
            .flex_none()
            .px_3()
            .pt_3()
            .pb_2()
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(format!("Lesson {} of {}", l + 1, lessons.len())),
                            )
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .child(lesson.title.clone()),
                            ),
                    )
                    // The first lesson has Kame beside its introduction instead.
                    .when(l > 0, |h| h.child(kame(66., 44.))),
            )
            .child(progress);

        // The lesson, with Load into editor and Run on its programs.
        let mut body: Vec<AnyElement> = Vec::new();
        let mut p = 0;
        for (b, block) in lesson.blocks.iter().enumerate() {
            match block {
                Block::Code { code, .. } => {
                    body.push(self.program_card(l, p, code, cx));
                    p += 1;
                }
                // The first lesson's first paragraph introduces Kame, beside it.
                Block::Paragraph(_) if l == 0 && b == 0 => body.push(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .child(div().mt_2().child(kame(96., 64.)))
                        .child(div().flex_1().min_w_0().children(self.blocks(
                            &format!("lesson-{l}-{b}"),
                            std::slice::from_ref(block),
                            cx,
                        )))
                        .into_any_element(),
                ),
                block => body.extend(self.blocks(
                    &format!("lesson-{l}-{b}"),
                    std::slice::from_ref(block),
                    cx,
                )),
            }
        }

        let nav = |id: &'static str, to: Option<usize>, back: bool| {
            to.map(|i| {
                let label = lessons[i].title.clone();
                let button = Button::new(id)
                    .small()
                    .flex_shrink(1.)
                    .min_w_0()
                    .tooltip(label.clone())
                    .label(label);
                let button = if back {
                    button.icon(IconName::ArrowLeft)
                } else {
                    button
                        .primary()
                        .child(Icon::new(IconName::ArrowRight).small().flex_none())
                };
                button.on_click(cx.listener(move |this, _, _, cx| {
                    this.lesson = i;
                    cx.notify();
                }))
            })
        };
        let footer = h_flex()
            .flex_none()
            .gap_2()
            .px_3()
            .py_2()
            .border_t_1()
            .border_color(theme.border)
            .children(nav("lesson-previous", l.checked_sub(1), true))
            .child(div().flex_1())
            .children(nav(
                "lesson-next",
                (l + 1 < lessons.len()).then_some(l + 1),
                false,
            ));

        v_flex()
            .flex_1()
            .min_h_0()
            .child(header)
            .child(
                div()
                    .id(SharedString::from(format!("lesson-{l}")))
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_3()
                    .pb_4()
                    .children(body),
            )
            .child(footer)
            .into_any_element()
    }

    /// One of a lesson's programs, with Load into editor and Run.
    fn program_card(
        &self,
        lesson: usize,
        program: usize,
        code: &str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let source = Source::Lesson { lesson, program };
        let running = self.running.as_ref().filter(|r| r.source == source);
        let is_running = running.is_some_and(|r| !r.finished);
        let highlight = running.and_then(|r| r.highlight.clone());
        let (app_load, app_run) = (self.app.clone(), self.app.clone());
        v_flex()
            .mt_2()
            .border_1()
            .border_color(if is_running {
                theme.foreground
            } else {
                theme.border
            })
            .rounded_md()
            .child(div().p_2().child(code_block(code, highlight, &theme)))
            .child(
                h_flex()
                    .gap_2()
                    .p_2()
                    .border_t_1()
                    .border_color(theme.border)
                    .child(
                        Button::new(SharedString::from(format!("load-{lesson}-{program}")))
                            .small()
                            .primary()
                            .label("Load into editor")
                            .on_click(move |_, window, cx| {
                                app_load
                                    .update(cx, |this, cx| {
                                        this.load_into_editor(lesson, program, window, cx)
                                    })
                                    .ok();
                            }),
                    )
                    .child(if is_running {
                        Button::new(SharedString::from(format!(
                            "stop-lesson-{lesson}-{program}"
                        )))
                        .small()
                        .icon(IconName::Square)
                        .label("Stop")
                        .on_click(move |_, _, cx| {
                            app_run.update(cx, |this, cx| this.stop(cx)).ok();
                        })
                    } else {
                        Button::new(SharedString::from(format!("run-lesson-{lesson}-{program}")))
                            .small()
                            .icon(IconName::Play)
                            .label("Run")
                            .on_click(move |_, _, cx| {
                                app_run
                                    .update(cx, |this, cx| this.run_source(source, cx))
                                    .ok();
                            })
                    }),
            )
            .into_any_element()
    }
}

impl PendownWindow {
    /// The Help button: shows or hides the side panel.
    pub(crate) fn toggle_help(&mut self, cx: &mut Context<Self>) {
        self.help_open = !self.help_open;
        if self.help_open {
            self.cursor_word = None;
            self.follow_cursor(cx);
        }
        cx.notify();
    }

    /// F1: opens the panel on help for the word under the cursor.
    pub(crate) fn show_help(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.help_open = true;
        let word = self.word_at_cursor(cx);
        self.cursor_word = word.clone();
        self.help_panel
            .update(cx, |panel, cx| panel.help_for(word, window, cx));
        cx.notify();
    }

    /// Load into editor: a lesson's program as a new, untitled program,
    /// once any unsaved changes have been dealt with, as New does.
    pub(crate) fn load_into_editor(
        &mut self,
        lesson: usize,
        program: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.then(crate::files::After::Load { lesson, program }, window, cx);
    }

    /// Opens the panel on a page, as the console's Help button does.
    pub(crate) fn open_help(&mut self, i: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.help_open = true;
        self.help_panel.update(cx, |panel, cx| {
            panel.go(Page::Command(i), window, cx);
            panel.pinned = true;
        });
        cx.notify();
    }

    /// What the editor's cursor is on, and its line.
    fn word_at_cursor(&self, cx: &App) -> Option<(String, u32)> {
        let editor = self.editor.read(cx);
        let text = editor.value();
        let offset = editor.cursor();
        let (name, range) = pendown::help::word_at(&text, offset)?;
        let line = text[..range.start].matches('\n').count() as u32;
        Some((name, line))
    }

    /// The editor changed or its cursor moved: the panel follows it.
    pub(crate) fn follow_cursor(&mut self, cx: &mut Context<Self>) {
        if !self.help_open {
            return;
        }
        let word = self.word_at_cursor(cx);
        if word.is_some() && word != self.cursor_word {
            self.cursor_word = word.clone();
            self.help_panel
                .update(cx, |panel, cx| panel.follow(word, cx));
        }
    }

    /// The Colours page's Copy to editor: puts `text` at the editor's
    /// cursor, in place of any selection, as one edit that Undo takes back,
    /// with a space before it if it needs one. Not while a program runs.
    pub(crate) fn insert_at_cursor(
        &mut self,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.running() {
            return;
        }
        self.editor.update(cx, |state, cx| {
            let start = state.selected_range().start;
            let before = state.value()[..start].chars().last();
            state.replace(pendown::colours::insertion(before, text), window, cx);
            state.focus(window, cx);
        });
        self.check_program(cx);
    }

    /// Copy to editor: adds an example to the end of the program, as one
    /// edit that Undo takes back out.
    pub(crate) fn copy_to_editor(
        &mut self,
        code: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editor.update(cx, |state, cx| {
            let text = state.value().to_string();
            let end = crate::app::position(&text, text.len());
            state.set_cursor_position(end, window, cx);
            let gap = if text.is_empty() || text.ends_with("\n\n") {
                ""
            } else if text.ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            };
            state.insert(format!("{gap}{code}\n"), window, cx);
        });
        self.check_program(cx);
    }

    /// Run on an example (issue 06): it runs on the main canvas, on an
    /// interpreter of its own, so the user's procedures and variables are
    /// untouched. The user's run, drawing and panels are set aside, and come
    /// back with Back to my drawing.
    pub(crate) fn run_example(&mut self, command: usize, example: usize, cx: &mut Context<Self>) {
        self.run_source(Source::Example { command, example }, cx);
    }

    pub(crate) fn run_source(&mut self, source: Source, cx: &mut Context<Self>) {
        self.stop(cx);
        if self.example.is_none() {
            let canvas = self.canvas.borrow();
            let saved = Saved {
                animator: self.animator.take(),
                run: self.run.take(),
                run_text: std::mem::take(&mut self.run_text),
                finished_at: self.finished_at,
                stop_at: self.stop_at,
                marks: canvas.drawing.marks().to_vec(),
                background: canvas.drawing.background(),
                watch: self.watched.read(cx).watch.clone(),
            };
            drop(canvas);
            self.example = Some(ExampleRun { source, saved });
        }
        self.example.as_mut().expect("set above").source = source;
        let code = source.code();
        self.run_text = code.clone();
        self.clear_editor_highlight(cx);
        self.canvas.borrow_mut().drawing.reset();
        self.show_watch(cx, |watched| watched.set(None));
        let (animator, run) = pendown::run::start(code, self.speed, pendown::run::DEFAULT_CHANNEL);
        self.play(animator, run, cx);
        let running = Running {
            source,
            highlight: None,
            finished: false,
        };
        self.help_panel
            .update(cx, |panel, cx| panel.set_running(Some(running), cx));
    }

    /// Back to my drawing: puts back what an example set aside.
    pub(crate) fn back_to_my_drawing(&mut self, cx: &mut Context<Self>) {
        let Some(example) = self.example.take() else {
            return;
        };
        if let Some(animator) = &mut self.animator {
            animator.stop();
        }
        let saved = example.saved;
        self.animator = saved.animator;
        self.run = saved.run;
        self.run_text = saved.run_text;
        self.finished_at = saved.finished_at;
        self.stop_at = saved.stop_at;
        let mut canvas = self.canvas.borrow_mut();
        canvas.drawing.reset();
        canvas.drawing.set_background(saved.background);
        canvas.drawing.push(&saved.marks);
        drop(canvas);
        self.show_watch(cx, |watched| watched.set(saved.watch));
        self.help_panel
            .update(cx, |panel, cx| panel.set_running(None, cx));
        self.show_highlight(cx);
        cx.notify();
    }

    /// Run: an example's picture goes, and so does what it set aside, since
    /// Run starts from a clean slate anyway.
    pub(crate) fn forget_example(&mut self, cx: &mut Context<Self>) {
        if self.example.take().is_some() {
            self.help_panel
                .update(cx, |panel, cx| panel.set_running(None, cx));
        }
    }

    /// The highlight goes on the example's code, in the panel.
    pub(crate) fn show_example_highlight(&mut self, cx: &mut Context<Self>) {
        let Some(example) = &self.example else { return };
        let highlight = self
            .animator
            .as_ref()
            .and_then(|a| a.highlight)
            .map(|s| s.range());
        let running = Running {
            source: example.source,
            highlight,
            finished: false,
        };
        self.help_panel
            .update(cx, |panel, cx| panel.set_running(Some(running), cx));
    }

    /// The example has finished or been stopped: its picture stays.
    pub(crate) fn example_finished(&mut self, cx: &mut Context<Self>) {
        let Some(example) = &self.example else { return };
        let running = Running {
            source: example.source,
            highlight: None,
            finished: true,
        };
        self.help_panel
            .update(cx, |panel, cx| panel.set_running(Some(running), cx));
    }

    /// The banner over the canvas while it shows an example.
    pub(crate) fn example_banner(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let example = self.example.as_ref()?;
        let theme = cx.theme();
        let text = example.source.describe();
        Some(
            h_flex()
                .absolute()
                .top_2()
                .left_2()
                .right_2()
                .gap_2()
                .px_2()
                .py_1()
                .rounded_md()
                .bg(theme.background.opacity(0.94))
                .text_color(theme.foreground)
                .text_sm()
                .child(
                    div()
                        .px_1()
                        .rounded_sm()
                        .bg(theme.foreground)
                        .text_color(theme.background)
                        .text_xs()
                        .child("Example"),
                )
                .child(div().flex_1().min_w_0().truncate().child(text))
                .child(
                    Button::new("back-to-my-drawing")
                        .small()
                        .label("Back to my drawing")
                        .on_click(cx.listener(|this, _, _, cx| this.back_to_my_drawing(cx))),
                )
                .into_any_element(),
        )
    }
}

/// What an example set aside: the user's run, drawing and variables.
pub struct Saved {
    animator: Option<pendown::animator::Animator>,
    run: Option<pendown::run::Run>,
    run_text: String,
    finished_at: Option<std::time::Instant>,
    stop_at: Option<std::time::Instant>,
    marks: Vec<pendown_render::Mark>,
    background: [u8; 4],
    watch: Option<Arc<pendown_logo::Watch>>,
}

/// The example on the canvas.
pub struct ExampleRun {
    pub source: Source,
    saved: Saved,
}
