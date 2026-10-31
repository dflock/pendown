//! Breakpoints (ADR 0029): which of the editor's lines can take one, how
//! they follow their lines as the program is edited, and where the
//! interpreter stops for them. Lines count from 0 here, as spans do.

use std::collections::BTreeSet;

/// Whether `line` has code to stop at: not blank, not just a comment, not
/// `END`, and not just brackets, which only close a list started above.
pub fn can_take(line: &str) -> bool {
    let code = line.split(';').next().unwrap_or("").trim();
    !code.is_empty()
        && !code.eq_ignore_ascii_case("end")
        && !code.chars().all(|c| "[]()".contains(c))
}

/// Where breakpoints on `old`'s lines are after the edit that made `new`.
/// The lines before and after the change keep theirs, moving with any lines
/// added or taken away above them. Within the change, a line that was edited
/// in place keeps its own; lines that were deleted or replaced lose theirs.
/// A breakpoint left on a line with no code goes.
pub fn follow(old: &str, new: &str, lines: &BTreeSet<usize>) -> BTreeSet<usize> {
    let old: Vec<&str> = old.split('\n').collect();
    let new: Vec<&str> = new.split('\n').collect();
    let before = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
    let after = old[before..]
        .iter()
        .rev()
        .zip(new[before..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let (old_changed, new_changed) = (old.len() - before - after, new.len() - before - after);
    lines
        .iter()
        .filter_map(|&line| {
            if line < before {
                Some(line)
            } else if line >= old.len() - after {
                Some(line + new.len() - old.len())
            } else if old_changed == 1 && new_changed >= 1 {
                // One line edited, perhaps split: it stays where it starts.
                Some(line)
            } else {
                None
            }
        })
        .filter(|&line| new.get(line).is_some_and(|l| can_take(l)))
        .collect()
}

/// The lines the interpreter stops at for these breakpoints in `text`. A
/// breakpoint on a `TO` line stops each time the procedure is called, at
/// its first instruction, so it becomes that instruction's line.
pub fn stops(text: &str, lines: &BTreeSet<usize>) -> Vec<u32> {
    let all: Vec<&str> = text.split('\n').collect();
    lines
        .iter()
        .filter_map(|&line| {
            let first = all.get(line)?.split_whitespace().next()?;
            if !first.eq_ignore_ascii_case("to") {
                return Some(line as u32);
            }
            let body = (line + 1..all.len()).find(|&l| {
                let code = all[l].split(';').next().unwrap_or("").trim();
                !code.is_empty()
            })?;
            can_take(all[body]).then_some(body as u32)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(lines: &[usize]) -> BTreeSet<usize> {
        lines.iter().copied().collect()
    }

    const PROGRAM: &str = "TO BOX :n\n  REPEAT 4 [FD :n RT 90]\nEND\n\nBOX 50\nBOX 100";

    #[test]
    fn lines_with_code_take_one() {
        let takes: Vec<bool> = PROGRAM.split('\n').map(can_take).collect();
        assert_eq!(takes, [true, true, false, false, true, true]);
        assert!(!can_take("; just a comment"));
        assert!(can_take("FD 10 ; and a comment"));
        assert!(!can_take("  ]"));
        assert!(!can_take("end"));
    }

    #[test]
    fn typing_on_its_line_keeps_it() {
        let new = PROGRAM.replace("BOX 50", "BOX 75");
        assert_eq!(follow(PROGRAM, &new, &set(&[4, 5])), set(&[4, 5]));
    }

    #[test]
    fn lines_added_above_move_it_down() {
        let new = format!("CS\nPU\n{PROGRAM}");
        assert_eq!(follow(PROGRAM, &new, &set(&[1, 4])), set(&[3, 6]));
    }

    #[test]
    fn lines_added_below_leave_it() {
        let new = format!("{PROGRAM}\nBOX 150\nBOX 200");
        assert_eq!(follow(PROGRAM, &new, &set(&[1, 5])), set(&[1, 5]));
    }

    #[test]
    fn deleting_its_line_takes_it_away() {
        let new = PROGRAM.replace("BOX 50\n", "");
        assert_eq!(follow(PROGRAM, &new, &set(&[1, 4, 5])), set(&[1, 4]));
    }

    #[test]
    fn pasting_lines_into_the_middle() {
        let new = PROGRAM.replace("\nBOX 50", "\nRT 45\nBOX 50\nRT 45");
        assert_eq!(follow(PROGRAM, &new, &set(&[4, 5])), set(&[4, 7]));
    }

    #[test]
    fn enter_at_the_end_of_its_line_keeps_it_there() {
        let new = PROGRAM.replace("BOX 50", "BOX 50\n");
        assert_eq!(follow(PROGRAM, &new, &set(&[4])), set(&[4]));
    }

    #[test]
    fn turning_its_line_into_a_comment_takes_it_away() {
        let new = PROGRAM.replace("BOX 50", "; BOX 50");
        assert_eq!(follow(PROGRAM, &new, &set(&[4, 5])), set(&[5]));
    }

    #[test]
    fn undo_brings_back_the_line_but_not_the_breakpoint() {
        let deleted = PROGRAM.replace("BOX 50\n", "");
        let after_delete = follow(PROGRAM, &deleted, &set(&[4]));
        assert!(after_delete.is_empty());
        assert!(follow(&deleted, PROGRAM, &after_delete).is_empty());
    }

    #[test]
    fn a_program_that_doesnt_parse_still_works() {
        let old = "REPEAT 4 [FD 10\nRT 90";
        let new = "CS\nREPEAT 4 [FD 10\nRT 90";
        assert_eq!(follow(old, new, &set(&[1])), set(&[2]));
    }

    #[test]
    fn a_to_line_stops_at_the_procedures_first_instruction() {
        assert_eq!(stops(PROGRAM, &set(&[0, 5])), [1, 5]);
        let empty = "TO NOTHING\n\n; nothing\nEND\nNOTHING";
        assert_eq!(stops(empty, &set(&[0])), Vec::<u32>::new());
    }
}
