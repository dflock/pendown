//! When an error on the code shows: the rules, with no window, so they can
//! be tested without timers. The app tells it what happened, and asks what
//! to show.
//!
//! While the learner types on a line, nothing new shows there until they
//! pause, or every half-typed command would get a red line under it. With the
//! cursor resting on an error's line, its message and card show. An error
//! from a run stays on show wherever the cursor is, until the next edit or
//! run. Esc hides the card until that line's text changes. Hovering over an
//! underline shows its card too, whichever line the cursor is on.

use pendown_logo::Span;

/// Where an error is in the program's text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spot {
    /// Counting from 0.
    pub line: usize,
    /// Bytes into the text.
    pub start: usize,
    pub end: usize,
}

impl Spot {
    /// `span` is in `text`.
    pub fn new(text: &str, span: Span) -> Spot {
        let start = (span.start as usize).min(text.len());
        Spot {
            line: text[..start].matches('\n').count(),
            start,
            end: span.end as usize,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cursor {
    /// Counting from 0.
    pub line: usize,
    /// Bytes into the text.
    pub offset: usize,
}

/// What to show. `underlined` and `card` are indexes into the errors.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shown {
    pub underlined: Vec<usize>,
    /// The error whose message and card show.
    pub card: Option<usize>,
    /// Lines with an error that the card isn't on, for a red dot.
    pub dots: Vec<usize>,
}

#[derive(Debug, Default)]
pub struct ErrorDisplay {
    errors: Vec<Spot>,
    text: String,
    cursor: Cursor,
    /// The line being typed on, until the learner pauses or moves off it.
    typing: Option<usize>,
    /// The line of a run's error.
    pinned: Option<usize>,
    /// The text of lines Esc hid the card on.
    hidden: Vec<String>,
    /// The error the pointer's on.
    hovered: Option<usize>,
    /// Esc hid the hovered error's card, until the pointer moves off it.
    hover_hidden: bool,
}

impl ErrorDisplay {
    /// The learner edited the program, leaving these errors.
    pub fn edited(&mut self, errors: Vec<Spot>, text: &str, cursor: Cursor) {
        self.replace(errors, text, cursor);
        self.typing = Some(cursor.line);
    }

    /// The program changed some other way, such as opening a file, or was
    /// checked again: its errors show at once.
    pub fn checked(&mut self, errors: Vec<Spot>, text: &str, cursor: Cursor) {
        self.replace(errors, text, cursor);
        self.typing = None;
    }

    /// A run stopped on these errors. Returns the line that's now pinned,
    /// to scroll into view. They show even on a line Esc hid the card on.
    pub fn run_stopped(&mut self, errors: Vec<Spot>, text: &str, cursor: Cursor) -> Option<usize> {
        self.replace(errors, text, cursor);
        self.hidden.clear();
        self.typing = None;
        self.pinned = self.errors.first().map(|e| e.line);
        self.pinned
    }

    /// A run started, or something else took the errors away.
    pub fn clear(&mut self) {
        self.errors.clear();
        self.typing = None;
        self.pinned = None;
        self.set_hovered(None);
    }

    fn replace(&mut self, errors: Vec<Spot>, text: &str, cursor: Cursor) {
        self.errors = errors;
        self.text = text.to_string();
        self.cursor = cursor;
        self.pinned = None;
        self.set_hovered(None);
        // A hidden line whose text changed is no longer there to find.
        self.hidden
            .retain(|hidden| text.split('\n').any(|l| l == hidden));
    }

    pub fn cursor_moved(&mut self, cursor: Cursor) {
        self.cursor = cursor;
        if self.typing != Some(cursor.line) {
            self.typing = None;
        }
    }

    /// The learner has stopped typing for a while.
    pub fn paused(&mut self) {
        self.typing = None;
    }

    /// The pointer moved onto this error, or off the one it was on.
    pub fn set_hovered(&mut self, error: Option<usize>) {
        if self.hovered != error {
            self.hovered = error;
            self.hover_hidden = false;
        }
    }

    pub fn hovered(&self) -> Option<usize> {
        self.hovered
    }

    /// The error starting at this byte of the text.
    pub fn error_at(&self, start: usize) -> Option<usize> {
        self.errors.iter().position(|e| e.start == start)
    }

    /// Esc: hides the card that's showing, and says if there was one.
    pub fn escape(&mut self) -> bool {
        let Some(i) = self.shown().card else {
            return false;
        };
        self.hover_hidden = self.hovered == Some(i);
        let line = self.errors[i].line;
        let text = self.text.split('\n').nth(line).unwrap_or_default();
        self.hidden.push(text.to_string());
        if self.pinned == Some(line) {
            self.pinned = None;
        }
        true
    }

    fn is_hidden(&self, line: usize) -> bool {
        let text = self.text.split('\n').nth(line).unwrap_or_default();
        self.hidden.iter().any(|hidden| hidden == text)
    }

    /// The error to show on a line: the one the cursor is in, which means
    /// anywhere in it or just after it, or else the first.
    fn on_line(&self, line: usize) -> Option<usize> {
        if self.is_hidden(line) {
            return None;
        }
        let mut on_line = (0..self.errors.len()).filter(|&i| self.errors[i].line == line);
        let first = on_line.next()?;
        let offset = self.cursor.offset;
        if line != self.cursor.line {
            return Some(first);
        }
        std::iter::once(first)
            .chain(on_line)
            .find(|&i| (self.errors[i].start..=self.errors[i].end).contains(&offset))
            .or(Some(first))
    }

    pub fn shown(&self) -> Shown {
        let line_of = |i: &usize| self.errors[*i].line;
        let underlined: Vec<usize> = (0..self.errors.len())
            .filter(|i| self.typing != Some(line_of(i)))
            .collect();
        // Moving the cursor onto another error's line shows that one, and a
        // pinned error shows again after moving off it.
        let at_cursor = (self.typing != Some(self.cursor.line))
            .then(|| self.on_line(self.cursor.line))
            .flatten();
        // The pointer on an underline asks for that error, even one Esc hid.
        let hovered = self.hovered.filter(|_| !self.hover_hidden);
        let card = hovered.or(at_cursor).or_else(|| self.on_line(self.pinned?));
        let card_line = card.as_ref().map(line_of);
        let mut dots: Vec<usize> = underlined
            .iter()
            .map(line_of)
            .filter(|&line| Some(line) != card_line)
            .collect();
        dots.dedup();
        Shown {
            underlined,
            card,
            dots,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `REPEAT 4` on line 1, `FOO` on line 3.
    const TEXT: &str = "CS\nREPEAT 4\nFD 50\nFOO\nRT 90";

    fn spot(text: &str, word: &str) -> Spot {
        let start = text.find(word).unwrap();
        Spot {
            line: text[..start].matches('\n').count(),
            start,
            end: start + word.len(),
        }
    }

    fn errors() -> Vec<Spot> {
        vec![spot(TEXT, "REPEAT"), spot(TEXT, "FOO")]
    }

    /// The cursor at the end of a line.
    fn end_of(text: &str, line: usize) -> Cursor {
        let lines: Vec<_> = text.split('\n').collect();
        let offset = lines[..=line].iter().map(|l| l.len() + 1).sum::<usize>() - 1;
        Cursor { line, offset }
    }

    fn on(line: usize) -> Cursor {
        end_of(TEXT, line)
    }

    fn checked(cursor: Cursor) -> ErrorDisplay {
        let mut display = ErrorDisplay::default();
        display.checked(errors(), TEXT, cursor);
        display
    }

    #[test]
    fn typing_on_a_line_shows_nothing_new_there() {
        let mut display = ErrorDisplay::default();
        display.edited(errors(), TEXT, on(1));
        let shown = display.shown();
        assert_eq!(shown.underlined, [1], "only the other line's");
        assert_eq!(shown.card, None);
        assert_eq!(shown.dots, [3]);
    }

    #[test]
    fn pausing_shows_the_underline_message_and_card() {
        let mut display = ErrorDisplay::default();
        display.edited(errors(), TEXT, on(1));
        display.paused();
        let shown = display.shown();
        assert_eq!(shown.underlined, [0, 1]);
        assert_eq!(shown.card, Some(0));
        assert_eq!(shown.dots, [3]);
    }

    #[test]
    fn typing_again_hides_it_until_the_next_pause() {
        let mut display = ErrorDisplay::default();
        display.edited(errors(), TEXT, on(1));
        display.paused();
        display.edited(errors(), TEXT, on(1));
        assert_eq!(display.shown().card, None);
        assert_eq!(display.shown().underlined, [1]);
        display.paused();
        assert_eq!(display.shown().card, Some(0));
    }

    #[test]
    fn moving_along_the_line_being_typed_on_keeps_waiting() {
        let mut display = ErrorDisplay::default();
        display.edited(errors(), TEXT, on(1));
        display.cursor_moved(Cursor {
            line: 1,
            offset: on(1).offset - 1,
        });
        assert_eq!(display.shown().card, None);
    }

    #[test]
    fn moving_off_the_line_being_typed_on_shows_its_underline_at_once() {
        let mut display = ErrorDisplay::default();
        display.edited(errors(), TEXT, on(1));
        display.cursor_moved(on(2));
        let shown = display.shown();
        assert_eq!(shown.underlined, [0, 1]);
        assert_eq!(shown.card, None);
        assert_eq!(shown.dots, [1, 3]);
    }

    #[test]
    fn moving_onto_an_errors_line_shows_it_at_once() {
        let mut display = ErrorDisplay::default();
        display.edited(errors(), TEXT, on(1));
        display.cursor_moved(on(3));
        assert_eq!(display.shown().card, Some(1));
    }

    #[test]
    fn the_cursor_on_another_line_gets_a_dot_on_each_error_line() {
        let shown = checked(on(2)).shown();
        assert_eq!(shown.underlined, [0, 1]);
        assert_eq!(shown.card, None);
        assert_eq!(shown.dots, [1, 3]);
    }

    #[test]
    fn opening_a_file_shows_errors_at_once() {
        assert_eq!(checked(on(1)).shown().card, Some(0));
    }

    #[test]
    fn a_runs_error_shows_wherever_the_cursor_is() {
        let mut display = ErrorDisplay::default();
        let line = display.run_stopped(errors(), TEXT, on(4));
        assert_eq!(line, Some(1));
        let shown = display.shown();
        assert_eq!(shown.card, Some(0));
        assert_eq!(shown.dots, [3]);
        display.cursor_moved(on(2));
        assert_eq!(display.shown().card, Some(0));
    }

    #[test]
    fn moving_onto_another_error_leaves_the_pin_to_come_back_to() {
        let mut display = ErrorDisplay::default();
        display.run_stopped(errors(), TEXT, on(4));
        display.cursor_moved(on(3));
        assert_eq!(display.shown().card, Some(1));
        display.cursor_moved(on(4));
        assert_eq!(display.shown().card, Some(0));
    }

    #[test]
    fn the_next_edit_clears_a_pin() {
        let mut display = ErrorDisplay::default();
        display.run_stopped(errors(), TEXT, on(4));
        display.edited(errors(), TEXT, on(4));
        assert_eq!(display.shown().card, None);
        display.paused();
        assert_eq!(display.shown().card, None, "the cursor's on no error");
    }

    #[test]
    fn the_next_run_clears_everything() {
        let mut display = ErrorDisplay::default();
        display.run_stopped(errors(), TEXT, on(4));
        display.clear();
        assert_eq!(display.shown(), Shown::default());
    }

    #[test]
    fn esc_hides_the_card_but_not_the_underline() {
        let mut display = checked(on(1));
        assert!(display.escape());
        let shown = display.shown();
        assert_eq!(shown.card, None);
        assert_eq!(shown.underlined, [0, 1]);
        assert_eq!(shown.dots, [1, 3]);
        assert!(!display.escape(), "nothing's showing now");
    }

    #[test]
    fn esc_lasts_through_moving_away_and_back() {
        let mut display = checked(on(1));
        display.escape();
        display.cursor_moved(on(2));
        display.cursor_moved(on(1));
        assert_eq!(display.shown().card, None);
    }

    #[test]
    fn esc_lasts_through_edits_to_other_lines() {
        let mut display = checked(on(1));
        display.escape();
        let text = TEXT.replace("FD 50", "FD 60");
        let errors = vec![spot(&text, "REPEAT"), spot(&text, "FOO")];
        display.edited(errors, &text, end_of(&text, 2));
        display.paused();
        display.cursor_moved(on(1));
        assert_eq!(display.shown().card, None);
    }

    #[test]
    fn esc_ends_when_the_lines_text_changes() {
        let mut display = checked(on(1));
        display.escape();
        let text = TEXT.replace("REPEAT 4", "REPEAT 5");
        let errors = vec![spot(&text, "REPEAT"), spot(&text, "FOO")];
        display.edited(errors.clone(), &text, end_of(&text, 1));
        assert_eq!(display.shown().card, None, "still typing");
        display.paused();
        assert_eq!(display.shown().card, Some(0));
        // Going back to the old text doesn't hide it again.
        display.edited(self::errors(), TEXT, on(1));
        display.paused();
        assert_eq!(display.shown().card, Some(0));
    }

    #[test]
    fn esc_hides_a_pinned_card() {
        let mut display = ErrorDisplay::default();
        display.run_stopped(errors(), TEXT, on(4));
        assert!(display.escape());
        assert_eq!(display.shown().card, None);
    }

    #[test]
    fn a_new_run_shows_an_error_esc_had_hidden() {
        let mut display = checked(on(1));
        display.escape();
        display.clear();
        display.run_stopped(errors(), TEXT, on(1));
        assert_eq!(display.shown().card, Some(0));
    }

    #[test]
    fn hovering_shows_a_card_on_any_line() {
        let mut display = checked(on(2));
        display.set_hovered(Some(1));
        let shown = display.shown();
        assert_eq!(shown.card, Some(1));
        assert_eq!(shown.dots, [1]);
        display.set_hovered(None);
        assert_eq!(display.shown().card, None);
    }

    #[test]
    fn hovering_wins_over_the_cursor_and_a_pin() {
        let mut display = checked(on(1));
        display.set_hovered(Some(1));
        assert_eq!(display.shown().card, Some(1));
        display.set_hovered(None);
        assert_eq!(display.shown().card, Some(0), "back to the cursor's");

        let mut display = ErrorDisplay::default();
        display.run_stopped(errors(), TEXT, on(4));
        display.set_hovered(Some(1));
        assert_eq!(display.shown().card, Some(1));
        display.set_hovered(None);
        assert_eq!(display.shown().card, Some(0), "back to the pinned one");
    }

    #[test]
    fn esc_hides_a_hovered_card_until_the_pointer_comes_back() {
        let mut display = checked(on(2));
        display.set_hovered(Some(0));
        assert!(display.escape());
        assert_eq!(display.shown().card, None);
        // Still on it: moving along the underline doesn't bring it back.
        display.set_hovered(Some(0));
        assert_eq!(display.shown().card, None);
        display.set_hovered(None);
        display.set_hovered(Some(0));
        assert_eq!(display.shown().card, Some(0));
    }

    #[test]
    fn hovering_shows_a_card_esc_hid_on_the_cursors_line() {
        let mut display = checked(on(1));
        display.escape();
        display.set_hovered(Some(0));
        assert_eq!(display.shown().card, Some(0));
        display.set_hovered(None);
        assert_eq!(display.shown().card, None, "still hidden for the cursor");
    }

    #[test]
    fn an_edit_ends_a_hover() {
        let mut display = checked(on(2));
        display.set_hovered(Some(0));
        display.edited(errors(), TEXT, on(2));
        assert_eq!(display.hovered(), None);
        assert_eq!(display.shown().card, None);
    }

    #[test]
    fn errors_are_found_by_where_they_start() {
        let display = checked(on(2));
        assert_eq!(display.error_at(3), Some(0));
        assert_eq!(display.error_at(4), None);
    }

    #[test]
    fn nothing_to_hide_leaves_esc_for_others() {
        assert!(!checked(on(2)).escape());
    }

    mod several_on_a_line {
        use super::*;

        const TEXT: &str = "FD 1\nFOO BAR  BAZ";

        fn display(offset_in_line: usize) -> ErrorDisplay {
            let errors = vec![spot(TEXT, "FOO"), spot(TEXT, "BAR"), spot(TEXT, "BAZ")];
            let mut display = ErrorDisplay::default();
            display.checked(errors, TEXT, cursor(offset_in_line));
            display
        }

        fn cursor(offset_in_line: usize) -> Cursor {
            Cursor {
                line: 1,
                offset: TEXT.find("FOO").unwrap() + offset_in_line,
            }
        }

        #[test]
        fn the_cursor_in_each_chooses_it() {
            assert_eq!(display(1).shown().card, Some(0));
            assert_eq!(display(5).shown().card, Some(1));
            assert_eq!(display(10).shown().card, Some(2));
        }

        #[test]
        fn just_after_one_counts_as_in_it() {
            assert_eq!(display(3).shown().card, Some(0));
            assert_eq!(display(7).shown().card, Some(1));
            assert_eq!(display(12).shown().card, Some(2));
        }

        #[test]
        fn in_none_of_them_chooses_the_first() {
            // Between BAR and BAZ.
            assert_eq!(display(8).shown().card, Some(0));
        }

        #[test]
        fn moving_switches_at_once() {
            let mut display = display(1);
            display.cursor_moved(cursor(5));
            assert_eq!(display.shown().card, Some(1));
            display.cursor_moved(cursor(10));
            assert_eq!(display.shown().card, Some(2));
        }

        #[test]
        fn esc_hides_the_card_for_the_whole_line() {
            let mut display = display(1);
            display.escape();
            display.cursor_moved(cursor(5));
            assert_eq!(display.shown().card, None);
        }

        #[test]
        fn a_line_with_several_gets_one_dot() {
            let mut display = display(1);
            display.cursor_moved(Cursor { line: 0, offset: 1 });
            assert_eq!(display.shown().dots, [1]);
        }
    }
}
