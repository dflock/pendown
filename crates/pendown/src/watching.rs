//! The counters and variables panels (phase 4): what the highlighted
//! instruction sees, from the animator's last `Watch` (ADR 0022). A value
//! or counter that changes flashes, and fades.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use gpui_kit::component::{ActiveTheme, h_flex, v_flex};
use gpui_kit::{prelude::FluentBuilder as _, *};
use pendown_logo::{
    Watch,
    watch::{Scope, ScopeKind, Variable},
};

/// How long a change flashes for.
pub const FLASH: Duration = Duration::from_millis(700);

/// What the panels show, shared by both.
#[derive(Default)]
pub struct Watched {
    pub watch: Option<Arc<Watch>>,
    /// When each counter last changed, if it has since the panels first
    /// showed this run.
    counters_changed: Vec<Option<Instant>>,
    /// The same for each scope's variables.
    variables_changed: Vec<Vec<Option<Instant>>>,
}

impl Watched {
    /// Shows `watch`, flashing what's different from what was shown. With
    /// nothing shown before, as when a run starts, nothing flashes.
    pub fn set(&mut self, watch: Option<Arc<Watch>>) {
        let now = Instant::now();
        let old = self.watch.as_deref();
        let counters_changed = match &watch {
            Some(new) => new
                .counters
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let before = old?.counters.get(i);
                    match before {
                        Some(b) if b.name == c.name && b.count == c.count => self.counters_changed[i],
                        _ => Some(now),
                    }
                })
                .collect(),
            None => Vec::new(),
        };
        let variables_changed = match &watch {
            Some(new) => new
                .scopes
                .iter()
                .enumerate()
                .map(|(i, scope)| {
                    let before = old.and_then(|o| o.scopes.get(i)).filter(|b| b.kind == scope.kind && b.name == scope.name);
                    scope
                        .variables
                        .iter()
                        .map(|v| {
                            old?;
                            let Some(before) = before else { return Some(now) };
                            match before.variables.iter().position(|b| b.name == v.name) {
                                Some(k) if before.variables[k].value == v.value => self.variables_changed[i][k],
                                _ => Some(now),
                            }
                        })
                        .collect()
                })
                .collect(),
            None => Vec::new(),
        };
        self.watch = watch;
        self.counters_changed = counters_changed;
        self.variables_changed = variables_changed;
    }

    /// After Stop: the interpreter has unwound every procedure and loop, so
    /// only the globals are left, with the values the panel already shows.
    pub fn stopped(&mut self) {
        let Some(watch) = &self.watch else { return };
        let globals: Vec<Scope> = watch
            .scopes
            .iter()
            .filter(|s| s.kind == ScopeKind::Globals)
            .map(|s| Scope { variables: s.variables.iter().map(|v| Variable { hidden: false, ..v.clone() }).collect(), ..s.clone() })
            .collect();
        let watch = Watch { counters: Vec::new(), scopes: globals };
        self.set(Some(Arc::new(watch)));
    }
}

/// A panel's title, as the console's is shown.
fn title(text: &'static str, cx: &App) -> Div {
    h_flex().px_2().py_1().child(div().text_sm().text_color(cx.theme().muted_foreground).child(text))
}

/// `row`, flashing if it changed in the last `FLASH`.
fn flashing(row: Div, id: String, changed: Option<Instant>) -> AnyElement {
    match changed {
        Some(at) if at.elapsed() < FLASH => {
            let colour = hsla(48.0 / 360.0, 0.95, 0.55, 0.5);
            let id = ElementId::Name(format!("{id}@{at:?}").into());
            row.with_animation(id, Animation::new(FLASH).with_easing(ease_out_quint()).with_max_fps(30.0), move |row, t| {
                row.bg(colour.opacity(1.0 - t))
            })
            .into_any_element()
        }
        _ => row.into_any_element(),
    }
}

/// A message for an empty panel.
fn empty(text: &'static str, cx: &App) -> Div {
    div().px_2().text_xs().text_color(cx.theme().muted_foreground).child(text)
}

/// Drawn cached: the window notifies it when `Watched` changes.
pub struct CountersPane {
    watched: Entity<Watched>,
    /// What it last drew, for the scripted session.
    pub drawn: Option<Arc<Watch>>,
}

impl CountersPane {
    pub fn new(watched: Entity<Watched>) -> CountersPane {
        CountersPane { watched, drawn: None }
    }
}

impl Render for CountersPane {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let watched = self.watched.read(cx);
        self.drawn = watched.watch.clone();
        let counters = watched.watch.as_ref().map_or(&[][..], |w| &w.counters[..]);
        let rows: Vec<AnyElement> = counters
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let of = c.of.map_or(String::new(), |of| format!(" of {of}"));
                let row = h_flex()
                    .px_2()
                    .gap_2()
                    .child(div().pl(px(10. * i as f32)).child(format!("{} {}{of}", c.name, c.count)))
                    .when(c.repcount, |row| {
                        row.child(div().flex_1()).child(div().text_color(theme.muted_foreground).child("REPCOUNT"))
                    });
                flashing(row, format!("counter-{i}"), watched.counters_changed.get(i).copied().flatten())
            })
            .collect();
        v_flex()
            .size_full()
            .child(title("Repeat counters", cx))
            .child(
                div()
                    .id("counters")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .font_family(theme.mono_font_family.clone())
                    .text_xs()
                    .when(rows.is_empty(), |list| list.child(empty("No loops running", cx)))
                    .children(rows),
            )
    }
}

/// Drawn cached: the window notifies it when `Watched` changes.
pub struct VariablesPane {
    watched: Entity<Watched>,
    /// What it last drew, for the scripted session.
    pub drawn: Option<Arc<Watch>>,
}

impl VariablesPane {
    pub fn new(watched: Entity<Watched>) -> VariablesPane {
        VariablesPane { watched, drawn: None }
    }
}

impl Render for VariablesPane {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let watched = self.watched.read(cx);
        self.drawn = watched.watch.clone();
        let scopes = watched.watch.as_ref().map_or(&[][..], |w| &w.scopes[..]);
        let mut rows: Vec<AnyElement> = Vec::new();
        for (i, scope) in scopes.iter().enumerate() {
            // The globals only get a heading when there's a call chain.
            if scope.kind != ScopeKind::Globals || scopes.len() > 1 {
                let heading = match scope.kind {
                    ScopeKind::Globals => "Globals".to_string(),
                    ScopeKind::Procedure if scope.variables.is_empty() => format!("{} (no inputs)", scope.name),
                    _ => scope.name.clone(),
                };
                rows.push(div().px_2().pt_1().text_color(theme.muted_foreground).child(heading).into_any_element());
            }
            for (j, v) in scope.variables.iter().enumerate() {
                let value = v.value.clone().unwrap_or_else(|| "no value".into());
                let row = h_flex()
                    .px_2()
                    .gap_2()
                    .when(scope.kind != ScopeKind::Globals || scopes.len() > 1, |row| row.pl_4())
                    .when(v.hidden || v.value.is_none(), |row| row.text_color(theme.muted_foreground))
                    .child(div().flex_none().max_w(px(140.)).truncate().child(v.name.clone()))
                    .child(div().flex_1().min_w_0().truncate().child(value));
                let changed = watched.variables_changed.get(i).and_then(|c| c.get(j)).copied().flatten();
                rows.push(flashing(row, format!("variable-{i}-{}", v.name), changed));
            }
        }
        let empty_list = scopes.iter().all(|s| s.variables.is_empty()) && scopes.len() <= 1;
        v_flex()
            .size_full()
            .child(title("Variables", cx))
            .child(
                div()
                    .id("variables")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .font_family(theme.mono_font_family.clone())
                    .text_xs()
                    .when(empty_list, |list| list.child(empty("No variables", cx)))
                    .children(rows),
            )
    }
}
