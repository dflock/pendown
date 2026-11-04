//! Everything shown about an error: its message, how the command is written
//! and an example, for the console, the hover and the error card.

use crate::{
    error::Error,
    reference::{Command, reference},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Explanation {
    pub message: String,
    /// `REPEAT number [instructions]`
    pub usage: Option<String>,
    /// `REPEAT 4 [FD 100 RT 90]`
    pub example: Option<String>,
    /// For a learner's own procedure: the line of its TO, counting from 1.
    pub defined_on: Option<u32>,
    /// Index into `Reference::commands`, for "Help on".
    pub help: Option<usize>,
}

/// Explains an error in a program with this text, where its procedures are
/// looked for.
pub fn explain(error: &Error, source: &str) -> Explanation {
    let mut explanation = Explanation {
        message: error.message.clone(),
        ..Explanation::default()
    };
    let Some(name) = error.command.as_deref() else {
        return explanation;
    };
    let r = reference();
    let found = r.find_index(name);
    match found.map(|i| &r.commands[i]) {
        Some(command) if command.implemented => {
            explanation.usage = usage_for(command, name);
            explanation.example = command.short_example.clone();
            explanation.help = found;
        }
        // A learner's procedure can have the name of a command that isn't
        // in Pendown.
        command => {
            if let Some((line, usage)) = procedure(name, source) {
                explanation.defined_on = Some(line);
                explanation.usage = Some(usage);
            } else if command.is_some() {
                explanation.help = found;
            }
        }
    }
    explanation
}

/// The line of a command's `usage` for the name as it was typed, in plain words.
/// `(SUM num1 num2 num3 ...)` is the last choice, as it's the unusual form.
fn usage_for(command: &Command, name: &str) -> Option<String> {
    let lines = command.usage_lines();
    let called = |line: &&String| {
        let mut words = line.split_whitespace();
        let first = words.next();
        first.is_some_and(|w| w.eq_ignore_ascii_case(name)) || words.next() == Some(name)
    };
    let line = lines
        .iter()
        .find(called)
        .or_else(|| lines.iter().find(|l| !l.starts_with('(')))
        .or(lines.first())?;
    Some(plain_usage(line))
}

fn plain_usage(line: &str) -> String {
    let words: Vec<_> = line.split_whitespace().map(plain_word).collect();
    words.join(" ")
}

/// A word of a usage, with the manual's name for an input made plain. The
/// parentheses of `(SUM num1 num2)` stay. Other words stay as they are.
fn plain_word(word: &str) -> String {
    let inner = word.trim_start_matches('(');
    let before = &word[..word.len() - inner.len()];
    let bare = inner.trim_end_matches(')');
    let after = &inner[bare.len()..];
    let plain = match bare {
        "num" | "num1" | "num2" | "num3" => "number",
        "instructionlist" | "instructionlist1" | "instructionlist2" => "[instructions]",
        "dist" => "distance",
        "thing" | "thing1" | "thing2" | "thing3" => "value",
        "list" => "[list]",
        "tf" | "tf1" | "tf2" | "tf3" | "tfexpression" => "condition",
        "varname" | "varname1" | "varname2" => "\"name",
        "varnamelist" => "[names]",
        "word1" | "word2" | "word3" => "word",
        "wordorlist" => "word-or-list",
        "data1" | "data2" => "data",
        "color" | "colornumber" | "colornumber.or.rgblist" => "colour",
        "pos" => "[x y]",
        "xcor" => "x",
        "ycor" => "y",
        "int" => "whole-number",
        "procname" => "name",
        other => other,
    };
    format!("{before}{plain}{after}")
}

/// A learner's procedure: the line of its TO, counting from 1, and how it's
/// written, as `SQ :n`.
fn procedure(name: &str, source: &str) -> Option<(u32, String)> {
    source.lines().enumerate().find_map(|(i, line)| {
        let code = line.split(';').next().unwrap_or_default();
        let mut words = code.split_whitespace();
        if !words.next()?.eq_ignore_ascii_case("to") {
            return None;
        }
        let title = words.next().filter(|w| w.eq_ignore_ascii_case(name))?;
        let inputs = words.take_while(|w| w.starts_with(':'));
        let usage = std::iter::once(title)
            .chain(inputs)
            .collect::<Vec<_>>()
            .join(" ");
        Some((i as u32 + 1, usage))
    })
}
