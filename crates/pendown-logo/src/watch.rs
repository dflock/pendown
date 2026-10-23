//! What the counters and variables panels show (phase 4, ADR 0022): a
//! snapshot of the loops running and the variables the running instruction
//! can see. The interpreter thread formats the values, since a `Value` can't
//! leave it.

use std::sync::Arc;

/// The panels' contents, sent as `Event::Watch`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Watch {
    /// The `REPEAT`, `FOREVER` and `FOR` loops and the templates running,
    /// outermost first.
    pub counters: Vec<Counter>,
    /// The globals, then each scope of the call chain, outermost first.
    pub scopes: Vec<Scope>,
    /// The pen, fill and background colours, which aren't variables but
    /// change as the program runs.
    /// Shared, since they rarely change from one `Watch` to the next.
    pub colours: Arc<[Colour]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Colour {
    /// The command that outputs it: `PENCOLOR`, `FILLCOLOR` or `BACKGROUND`.
    pub name: String,
    /// In the form it was set, as that command outputs it.
    pub value: String,
    /// What it looks like, or `None` for no fill.
    pub rgba: Option<[u8; 4]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Counter {
    /// The command, in capitals: `REPEAT`, `FOR`, `MAP`.
    pub name: String,
    /// Which pass it's on, from 1.
    pub count: i64,
    /// How many passes there will be, if that's known.
    pub of: Option<i64>,
    /// This is the counter `REPCOUNT` outputs.
    pub repcount: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeKind {
    Globals,
    /// A procedure's inputs and locals.
    Procedure,
    /// `FOR`'s variable, or a lambda template's inputs.
    Loop,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scope {
    pub kind: ScopeKind,
    /// The procedure's name as it was defined, or the command in capitals.
    /// Empty for the globals.
    pub name: String,
    pub variables: Vec<Variable>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    /// The spelling of its definition (ADR 0004).
    pub name: String,
    /// In short form, as `SHOW` prints it. `None` for a `LOCAL` with no
    /// value yet.
    pub value: Option<String>,
    /// A later scope has a variable with the same name, so the running
    /// instruction can't see this one.
    pub hidden: bool,
}

/// Values are shown at most this many characters long.
pub const VALUE_LIMIT: usize = 200;
