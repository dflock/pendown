//! A scripted session for what can only be checked with a window (issue 03).
//! `PENDOWN_AUTOTEST=1 cargo run --release -p pendown` runs it, writes
//! `autotest.log` and screenshots into `target/autotest/`, and exits with an
//! error if any check fails. Set `PENDOWN_AUTOTEST` to a directory to write
//! there instead.

use std::{
    fmt::Write as _,
    path::PathBuf,
    time::{Duration, Instant},
};

use gpui_kit::{component::WindowExt as _, *};
use pendown::{
    animator::{Mode, Speed},
    metrics::Snapshot,
    run::DEFAULT_CHANNEL,
    speed::level_speed,
};
use pendown_logo::SourceId;

use crate::app::App;

const BOX: &str = "REPEAT 4 [FD 300 RT 90]";

const STEPPER: &str = r#"MAKE "n 0
REPEAT 6 [
  MAKE "n :n + 1
  FD 50 * :n
  RT 60
  PRINT :n
]
PRINT "done
"#;

const FOREVER: &str = "FOREVER [FD 3 RT 1.5]";

const FOREVER_LINES: &str = "FOREVER [\n  FD 3\n  RT 1.5\n]";

const TREE: &str = r#"TO TREE :size :depth
  IF :depth = 0 [STOP]
  FD :size
  LT 30 TREE :size * 0.72 :depth - 1
  RT 60 TREE :size * 0.72 :depth - 1
  LT 30 BK :size
END
PU BK 450 PD
SETPC 10
TREE 230 16
"#;

/// At speed 3 it moves about 14 pixels a second, so it needs about that many
/// frames a second, and prints now and then.
const SLOW: &str = "REPEAT 8 [\n  FD 20\n  RT 45\n  PRINT REPCOUNT\n]";

/// Phase 4: a loop in a procedure, called from a template.
const WATCHED: &str = "TO TILE :size
  REPEAT 4 [FD :size RT 90]
END
MAKE \"sizes [40 80]
FOREACH :sizes [TILE ?]
";

/// ADR 0028: a program to pause and change, and the changes.
const SQUARE_SIDES: &str = "TO SIDE :n\n  FD :n\n  RT 90\nEND\nREPEAT 4 [SIDE 100]\nPRINT \"done\n";
const TRIANGLE_SIDES: &str =
    "TO SIDE :n\n  FD :n * 2\n  RT 120\nEND\nREPEAT 3 [SIDE 100]\nPRINT \"done\n";
const ONE_SIDE: &str = "TO SIDE :n\n  FD :n\n  RT 90\nEND\nREPEAT 1 [SIDE 100]\nPRINT \"done\n";
const UNCLOSED: &str = "TO SIDE :n\n  FD :n\n  RT 90\nEND\nREPEAT 4 [SIDE 100\nPRINT \"done\n";

/// ADR 0029: a procedure with a loop over several lines, to stop in.
const BREAKING: &str =
    "TO TILE :size\n  REPEAT 4 [\n    FD :size\n    RT 90\n  ]\nEND\n\nTILE 100\n";
/// Long enough to scroll: TILE 1 to TILE 50, each on its line after the
/// procedure.
const LONG_TILES: &str = "TO TILE :size\n  REPEAT 4 [FD :size RT 90]\nEND\nTILE 1\nTILE 2\nTILE 3\nTILE 4\nTILE 5\nTILE 6\nTILE 7\nTILE 8\nTILE 9\nTILE 10\nTILE 11\nTILE 12\nTILE 13\nTILE 14\nTILE 15\nTILE 16\nTILE 17\nTILE 18\nTILE 19\nTILE 20\nTILE 21\nTILE 22\nTILE 23\nTILE 24\nTILE 25\nTILE 26\nTILE 27\nTILE 28\nTILE 29\nTILE 30\nTILE 31\nTILE 32\nTILE 33\nTILE 34\nTILE 35\nTILE 36\nTILE 37\nTILE 38\nTILE 39\nTILE 40\nTILE 41\nTILE 42\nTILE 43\nTILE 44\nTILE 45\nTILE 46\nTILE 47\nTILE 48\nTILE 49\nTILE 50\n";
const FOREVER_TURNING: &str = "FOREVER [\n  FD 1\n  RT 1\n]\n";

const COUNTING: &str = "MAKE \"n 0\nFOREVER [MAKE \"n :n + 1]";

const ERRORS: &str = "TO BOX :n\n  FD :n RT 90\n  FD \"far\n  PRINT 1\nEND\nBOX 100";

/// One step of the session.
#[derive(Clone)]
#[allow(clippy::enum_variant_names)]
enum Step {
    Say(&'static str),
    Run(&'static str, Speed),
    Wait(Duration),
    WaitFinished,
    WaitPaused,
    SetSpeed(Speed),
    Pause,
    Resume,
    SingleStep,
    /// End the run, as Run and a console entry do (ADR 0028): there's no
    /// Stop button, but ending a run works as Stop did.
    EndRun,
    Screenshot(&'static str),
    /// Start recording the turtle's y position at every frame.
    Track,
    /// Stop recording, and check the turtle's speed before and after the
    /// given time: about the two speeds, in steps a second.
    CheckSpeeds(Duration, f64, f64),
    /// Start recording every highlight change.
    Trace,
    /// Stop recording, and check the highlighted texts in order.
    CheckTrace(&'static [&'static str]),
    CheckHighlight(&'static str),
    /// The turtle is part-way through a move.
    CheckMoving,
    /// Nothing is part-way through: the move, turn or shape is finished.
    CheckNotMoving,
    /// The editor shows the highlight, or doesn't.
    CheckEditorHighlight(bool),
    /// The editor can be typed in, or is read-only.
    CheckEditable(bool),
    /// The run is paused, with something left to run, or isn't.
    CheckPaused(bool),
    /// The console holds exactly these lines.
    CheckConsoleExactly(&'static [&'static str]),
    /// Click a line number, counting from 1, to set or take away its
    /// breakpoint (ADR 0029).
    ToggleBreakpoint(usize),
    /// The lines with a breakpoint, counting from 1.
    CheckBreakpoints(&'static [usize]),
    /// Press Run on whatever's in the editor, at this speed.
    RunAsIs(Speed),
    /// Scroll the editor so this line, counting from 1, is at the top.
    ScrollEditorTo(usize),
    /// The Reset button (ADR 0028).
    Reset,
    /// Moves the mouse to a point in the window, for tooltips.
    HoverAt(f32, f32),
    /// What the first run button says, and whether Reset can be pressed.
    CheckButtons(&'static str, bool),
    /// The line the gutter's arrow is on, counting from 1, if any.
    CheckCurrentLine(Option<usize>),
    /// The interpreter has started exactly as many lines as the highlight
    /// has reached.
    CheckNothingAhead,
    /// Stop was pressed, and the interpreter thread exits within this long.
    CheckStopsWithin(Duration),
    CheckFinishedWithin(Duration),
    CheckFramesAtMost(Duration),
    CheckConsole(&'static str),
    /// Type a line into the console and press Enter.
    Console(&'static str),
    /// Press keys in the console's input line, as `Keystroke::parse` reads them.
    ConsoleKeys(&'static str),
    /// Press the console's Undo button.
    UndoConsole,
    /// The console's input line holds exactly this.
    CheckConsoleTyped(&'static str),
    /// Press the console's Teach Kame button.
    OpenTeach,
    /// Type the new word's name in Teach Kame's dialog.
    TeachName(&'static str),
    /// Tick or untick an entry in the dialog, with Shift if asked.
    TeachToggle(usize, bool),
    /// Press the dialog's Teach Kame button.
    PressTeach,
    /// Press the dialog's All (true) or None (false).
    TeachAll(bool),
    /// The dialog's rows, as `TeachKame::describe` gives them.
    CheckTeachRows(&'static str),
    /// Why the dialog won't take the name, or "" if it will.
    CheckTeachProblem(&'static str),
    /// The dialog's picture has been drawn.
    CheckTeachPicture,
    /// A dialog is open, or not.
    CheckDialog(bool),
    /// The console's Undo button names this entry, or is disabled for "".
    CheckUndoLabel(&'static str),
    /// The console's Teach Kame button is enabled, or not.
    CheckCanTeach(bool),
    /// Press keys wherever the focus is, or in the editor.
    Keys(&'static str),
    EditorKeys(&'static str),
    /// Press the console's Copy to editor.
    CopyDefinitions,
    /// Open the side panel, if it isn't.
    ShowPanel,
    /// Turn Show thinking on or off.
    SetThinking(bool),
    /// Turn Drive on or off.
    Drive(bool),
    /// Press and let go of keys while driving: up, down, left, right, space,
    /// undo, escape.
    DriveKeys(&'static [&'static str]),
    /// Keys coming down with no key-up, as when the keyboard repeats a key
    /// that's held.
    DriveKeysDown(&'static [&'static str]),
    /// A key held while driving comes up.
    DriveKeyUp(&'static str),
    /// Wait until every press has played.
    WaitDriven,
    /// Driving is on, or not.
    CheckDriving(bool),
    /// Choose driving's steps for moves and turns.
    DriveSteps(f64, f64),
    /// How many presses are waiting to play.
    CheckDriveQueue(usize),
    /// What Kame is thinking about, and while driving what the entry shows.
    CheckDriveThinking(&'static str),
    /// Scroll the help page this far down, in logical pixels.
    ScrollHelp(f32),
    /// Scroll the lesson showing this far down, in logical pixels.
    ScrollLesson(f32),
    /// Type in the Colours page's R, G, B, A or hex field, counting from 0.
    ColourField(usize, &'static str),
    /// Click on the Colours page's wheel, from 0 to 1 across and down.
    WheelAt(f64, f64),
    /// Click a palette swatch, or a name.
    PickPalette(u8),
    PickName(&'static str),
    /// Choose the form to copy in, by its button's label.
    ChooseForm(&'static str),
    /// Press the Colours page's Copy to editor.
    CopyColour,
    /// What Copy to editor would put in.
    CheckCurrentColour(&'static str),
    /// Why a field doesn't make sense, or "" if they all do.
    CheckColourProblem(&'static str),
    /// Type in the names' filter.
    FilterNames(&'static str),
    /// The drawing's background, as RGBA.
    CheckBackground([u8; 4]),
    /// How many lines, arcs and shapes are drawn.
    CheckDrawn(usize),
    /// Where the turtle sprite is, and its heading.
    CheckTurtle(f64, f64, f64),
    /// Save the program as this file in the autotest folder, as Save As
    /// would once the dialog has a name.
    SaveTo(&'static str),
    OpenFrom(&'static str),
    ExportTo(&'static str),
    /// Replace the editor's text, as typing would.
    Type(&'static str),
    CheckTitle(&'static str),
    /// The editor holds exactly this.
    CheckEditor(&'static str),
    /// The exported picture is a PNG the size of the canvas.
    CheckExported(&'static str),
    /// The exported picture is an SVG with an element for every line and
    /// shape drawn.
    CheckExportedSvg(&'static str),
    /// The editor shows exactly these diagnostics.
    CheckDiagnostics(&'static [&'static str]),
    /// Start counting renders and highlight changes.
    MeasureRenders,
    /// Since `MeasureRenders`: the window rendered at most this many times a
    /// second, and at most once an animation frame; the editor only when the
    /// highlight moved, and the console only when something was printed.
    CheckRenders(f64),
    /// The counters and variables panels show exactly this, in the form
    /// `panels()` gives.
    CheckPanels(&'static str),
    /// They show something starting with this.
    CheckPanelsStart(&'static str),
    /// Both panels have drawn what they should show.
    CheckPanelsDrawn,
    /// The variables panel shows these colours, as `PENCOLOR=red ...`.
    CheckColours(&'static str),
    /// Press F12.
    ToggleMetrics,
    /// Phase 5: put the editor's cursor one character into the first
    /// `marker`.
    CursorAt(&'static str),
    /// The same, then press F1.
    F1At(&'static str),
    /// The help panel shows the page with this title.
    CheckHelpPage(&'static str),
    /// Type into the help panel's search box.
    SearchHelp(&'static str),
    /// The first search result is the command with this title.
    CheckFirstResult(&'static str),
    /// Open a help page by name, as a link does.
    OpenHelp(&'static str),
    /// The Contents button.
    HelpContents,
    /// Close the side panel, as its button does.
    CloseHelp,
    /// Press Run on an example: the command's name, and which example,
    /// counting from 0.
    RunExample(&'static str, usize),
    /// The example's code is highlighted here, in the panel.
    CheckExampleHighlight(&'static str),
    BackToMyDrawing,
    /// Press Copy to editor on an example.
    CopyExample(&'static str, usize),
    /// The editor's text ends with this.
    CheckEditorEnds(&'static str),
    /// The console has a Help button for this page, and pressing it opens
    /// the panel there.
    CheckErrorHelp(&'static str),
    /// Open the Tutorial tab at a lesson, counting from 0.
    Lesson(usize),
    /// Load into editor on a lesson's program, without the question about
    /// unsaved changes.
    LoadLesson(usize, usize),
    /// Run on a lesson's program.
    RunLesson(usize, usize),
    /// Open the side panel at the Gallery tab.
    Gallery,
    /// Every gallery picture is drawn.
    CheckGalleryDrawn,
    /// Opens the Shortcuts tab.
    Shortcuts,
    /// Every row of the Shortcuts tab has all its keys from the keymap.
    CheckShortcutsBound,
    /// The row that says this shows these keys, as this computer writes
    /// them.
    CheckShortcut(&'static str, &'static [&'static str]),
    /// Press Run, Load or the "from" link on the gallery card with this
    /// title.
    RunGallery(&'static str),
    LoadGallery(&'static str),
    GalleryFrom(&'static str),
    /// The banner over the canvas says this.
    CheckShowing(&'static str),
    /// Scroll the Gallery this far down, and check it's still there.
    ScrollGallery(f32),
    CheckGalleryScroll(f32),
    /// The panel shows this tab.
    CheckTab(&'static str),
    /// The metrics overlay is shown, with numbers in it, or isn't.
    CheckOverlay(bool),
    Quit,
}

const CATCHING: &str = "CATCH \"error [FD \"x]
SHOW ERROR
TO CHECK :n
IF :n < 0 [(THROW \"error [CHECK needs a positive number])]
END
CHECK -1";

fn script() -> Vec<Step> {
    use Step::*;
    let ms = Duration::from_millis;
    vec![
        Say("A console entry before any run"),
        Console("FD 1"),
        WaitFinished,
        CheckDrawn(1),
        Say("A console turn turns where the last entry left the turtle"),
        Console("CS"),
        WaitFinished,
        Console("FD 100"),
        WaitFinished,
        Console("RT 45"),
        WaitFinished,
        CheckTurtle(0.0, 100.0, 45.0),
        Say("The welcome program"),
        Run(crate::app::WELCOME, level_speed(10)),
        WaitFinished,
        CheckFinishedWithin(Duration::from_secs(30)),
        Say("The console uses what the last run left"),
        Console("BOX 50"),
        WaitFinished,
        CheckDrawn(52),
        CheckConsole("> BOX 50"),
        Console("FOO"),
        WaitFinished,
        CheckConsole("In the console: I don't know how to FOO"),
        Console("TO TRI :n"),
        Console("REPEAT 3 [FD :n RT 120]"),
        Console("END"),
        WaitFinished,
        Console("TRI 100"),
        WaitFinished,
        CheckDrawn(55),
        Screenshot("console"),
        Run(crate::app::WELCOME, Speed::Instant),
        WaitFinished,
        CheckConsole(
            "Run cleared the procedures you defined in the console. \"Copy to editor\" keeps them.",
        ),
        Console("TRI 10"),
        WaitFinished,
        CheckConsole("In the console: I don't know how to TRI"),
        Say("Errors show while typing"),
        CheckDiagnostics(&[]),
        // The editor sends its change event once the edit is done.
        Type("FD 10\nFOOBAR 3\nREPEAT 4 [FD]"),
        Wait(ms(100)),
        CheckDiagnostics(&[
            "I don't know how to FOOBAR",
            "FD needs 1 input, but got none",
        ]),
        Screenshot("errors-while-typing"),
        Type("REPEAT 4 [FD 10 RT 90]"),
        Wait(ms(100)),
        CheckDiagnostics(&[]),
        Type(crate::app::WELCOME),
        Say("Files"),
        CheckTitle("Untitled — Pendown"),
        Type("REPEAT 3 [FD 200 RT 120]"),
        CheckTitle("● Untitled — Pendown"),
        SaveTo("triangle"),
        CheckTitle("triangle.logo — Pendown"),
        Run("REPEAT 5 [FD 200 RT 144]", Speed::Instant),
        WaitFinished,
        CheckTitle("● triangle.logo — Pendown"),
        ExportTo("star.png"),
        CheckExported("star.png"),
        ExportTo("star.svg"),
        CheckExportedSvg("star.svg"),
        OpenFrom("triangle.logo"),
        CheckEditor("REPEAT 3 [FD 200 RT 120]"),
        CheckTitle("triangle.logo — Pendown"),
        Say("Highlighting follows the turtle"),
        Trace,
        Run(BOX, Speed::Animated(100.0)),
        Wait(ms(1500)),
        CheckHighlight("FD 300"),
        CheckMoving,
        Screenshot("highlight-mid-move"),
        WaitFinished,
        CheckTrace(&[
            "REPEAT 4 [FD 300 RT 90]",
            "FD 300",
            "RT 90",
            "FD 300",
            "RT 90",
            "FD 300",
            "RT 90",
            "FD 300",
            "RT 90",
        ]),
        Say("A speed change applies in the middle of a move"),
        Track,
        Run("FD 600", Speed::Animated(50.0)),
        Wait(ms(1500)),
        SetSpeed(Speed::Animated(200.0)),
        Wait(ms(600)),
        CheckSpeeds(ms(1500), 50.0, 200.0),
        SetSpeed(Speed::Instant),
        WaitFinished,
        Say("Pause lets the move finish, and pauses before the next instruction"),
        Run("FD 300\nRT 90\nFD 300", Speed::Animated(50.0)),
        Wait(ms(1500)),
        CheckMoving,
        Pause,
        WaitPaused,
        CheckNotMoving,
        CheckHighlight("RT 90"),
        Wait(ms(300)),
        CheckHighlight("RT 90"),
        Screenshot("paused-after-the-move"),
        EndRun,
        Say("Single Step stops at each instruction, with nothing run ahead"),
        Run(STEPPER, Speed::Animated(100.0)),
        Wait(ms(1200)),
        Pause,
        SingleStep,
        WaitPaused,
        Wait(ms(50)),
        CheckNothingAhead,
        SingleStep,
        WaitPaused,
        Wait(ms(50)),
        CheckNothingAhead,
        SingleStep,
        WaitPaused,
        Wait(ms(50)),
        CheckNothingAhead,
        Screenshot("paused"),
        EndRun,
        CheckStopsWithin(ms(50)),
        Say("ADR 0028: the program can be changed while paused, and Resume replays it"),
        Run(SQUARE_SIDES, Speed::Animated(100.0)),
        Wait(ms(1500)),
        CheckEditable(false),
        Pause,
        WaitPaused,
        Wait(ms(50)),
        CheckEditable(true),
        CheckEditorHighlight(true),
        Type(TRIANGLE_SIDES),
        Wait(ms(100)),
        CheckEditorHighlight(false),
        EditorKeys("secondary-z"),
        Wait(ms(100)),
        CheckEditor(SQUARE_SIDES),
        CheckEditorHighlight(true),
        Type(TRIANGLE_SIDES),
        Wait(ms(100)),
        Screenshot("changed-while-paused"),
        Resume,
        Wait(ms(300)),
        CheckEditorHighlight(true),
        Screenshot("replayed"),
        SetSpeed(Speed::Instant),
        WaitFinished,
        CheckDrawn(3),
        CheckTurtle(0.0, 0.0, 0.0),
        CheckConsoleExactly(&["done"]),
        Say("ADR 0028: Single Step after a change replays, then steps"),
        Run(SQUARE_SIDES, Speed::Animated(100.0)),
        Wait(ms(1500)),
        Pause,
        WaitPaused,
        Type(TRIANGLE_SIDES),
        Wait(ms(100)),
        SingleStep,
        WaitPaused,
        Wait(ms(50)),
        CheckPaused(true),
        CheckEditorHighlight(true),
        CheckNothingAhead,
        SetSpeed(Speed::Instant),
        Resume,
        WaitFinished,
        CheckDrawn(3),
        Say("ADR 0028: a change that doesn't parse leaves it paused"),
        Run(SQUARE_SIDES, Speed::Animated(100.0)),
        Wait(ms(1500)),
        Pause,
        WaitPaused,
        Type(UNCLOSED),
        Wait(ms(100)),
        Resume,
        Wait(ms(200)),
        CheckPaused(true),
        CheckEditorHighlight(false),
        Screenshot("doesnt-parse"),
        EditorKeys("secondary-z"),
        Wait(ms(100)),
        CheckEditorHighlight(true),
        SetSpeed(Speed::Instant),
        Resume,
        WaitFinished,
        CheckDrawn(4),
        Say("ADR 0028: a change that ends sooner just finishes"),
        Run(SQUARE_SIDES, Speed::Animated(100.0)),
        Wait(ms(2500)),
        Pause,
        WaitPaused,
        Type(ONE_SIDE),
        Wait(ms(100)),
        Resume,
        WaitFinished,
        CheckDrawn(1),
        CheckConsoleExactly(&["done"]),
        Screenshot("shorter-finishes"),
        Say("ADR 0028: typing in the console ends a paused program"),
        Run(FOREVER_LINES, Speed::Animated(100.0)),
        Wait(ms(300)),
        Pause,
        WaitPaused,
        Wait(ms(50)),
        CheckPaused(true),
        Console("PRINT \"hello"),
        WaitFinished,
        CheckPaused(false),
        CheckConsole("hello"),
        Screenshot("console-ends-a-paused-program"),
        Say("ADR 0029: a breakpoint stops the run each time it gets there"),
        Type(BREAKING),
        Wait(ms(100)),
        ToggleBreakpoint(3),
        ToggleBreakpoint(7),
        CheckBreakpoints(&[3]),
        RunAsIs(Speed::Animated(100.0)),
        WaitPaused,
        CheckHighlight("FD :size"),
        CheckDrawn(0),
        Wait(ms(100)),
        Screenshot("at-a-breakpoint"),
        Resume,
        WaitPaused,
        CheckHighlight("FD :size"),
        CheckDrawn(1),
        CheckNothingAhead,
        SingleStep,
        WaitPaused,
        CheckHighlight("RT 90"),
        EndRun,
        Say("ADR 0029: at Instant too"),
        RunAsIs(Speed::Instant),
        WaitPaused,
        CheckHighlight("FD :size"),
        CheckDrawn(0),
        Resume,
        WaitPaused,
        CheckDrawn(1),
        CheckNothingAhead,
        ToggleBreakpoint(3),
        CheckBreakpoints(&[]),
        Resume,
        WaitFinished,
        CheckDrawn(4),
        Say("ADR 0029: a breakpoint on a TO line stops at the procedure's first instruction"),
        ToggleBreakpoint(1),
        RunAsIs(Speed::Instant),
        WaitPaused,
        CheckHighlight("REPEAT 4 [\n    FD :size\n    RT 90\n  ]"),
        EndRun,
        Say("ADR 0029: breakpoints keep to their lines as the program changes"),
        ToggleBreakpoint(1),
        ToggleBreakpoint(4),
        CheckBreakpoints(&[4]),
        Type(
            "CS\nPU\nSETPOS [-50 -50]\nPD\nTO TILE :size\n  REPEAT 4 [\n    FD :size\n    RT 90\n  ]\nEND\n\nTILE 100\n",
        ),
        Wait(ms(100)),
        CheckBreakpoints(&[8]),
        Screenshot("breakpoint-moved-down"),
        EditorKeys("secondary-z"),
        Wait(ms(100)),
        CheckBreakpoints(&[4]),
        Type("TO TILE :size\n  REPEAT 4 [\n    FD :size\n  ]\nEND\n\nTILE 100\n"),
        Wait(ms(100)),
        CheckBreakpoints(&[]),
        Say("ADR 0029: the line numbers follow the editor as it scrolls"),
        Type(LONG_TILES),
        Wait(ms(100)),
        ToggleBreakpoint(41),
        ScrollEditorTo(30),
        Wait(ms(200)),
        RunAsIs(Speed::Instant),
        WaitPaused,
        CheckHighlight("TILE 38"),
        Wait(ms(100)),
        Screenshot("scrolled-breakpoint"),
        EndRun,
        Say("ADR 0029: a breakpoint set while it runs stops it"),
        Type(FOREVER_TURNING),
        Wait(ms(100)),
        RunAsIs(Speed::Instant),
        Wait(ms(300)),
        CheckPaused(false),
        ToggleBreakpoint(3),
        WaitPaused,
        CheckHighlight("RT 1"),
        ToggleBreakpoint(3),
        Resume,
        Wait(ms(300)),
        CheckPaused(false),
        EndRun,
        Say(
            "ADR 0028: Reset puts everything back as before Run, and Run says Restart while it runs",
        ),
        Type(BREAKING),
        Wait(ms(100)),
        Reset,
        CheckButtons("Run", false),
        CheckDrawn(0),
        CheckConsoleExactly(&[]),
        CheckCurrentLine(None),
        ToggleBreakpoint(3),
        Say("ADR 0029: the line numbers have tooltips"),
        Wait(ms(100)),
        HoverAt(22.0, 98.0),
        Wait(ms(1000)),
        Screenshot("breakpoint-tooltip"),
        HoverAt(22.0, 58.0),
        Wait(ms(1000)),
        Screenshot("to-line-tooltip"),
        HoverAt(700.0, 400.0),
        RunAsIs(Speed::Animated(100.0)),
        WaitPaused,
        CheckButtons("Restart", true),
        Say(
            "ADR 0029: the gutter's arrow is on the highlight's line, and a breakpoint's pill there has a white ring",
        ),
        CheckCurrentLine(Some(3)),
        Wait(ms(100)),
        Screenshot("current-line-at-a-breakpoint"),
        SingleStep,
        WaitPaused,
        CheckCurrentLine(Some(4)),
        Wait(ms(100)),
        Screenshot("current-line"),
        ToggleBreakpoint(3),
        Resume,
        WaitFinished,
        CheckButtons("Run", true),
        CheckDrawn(4),
        Console("MAKE \"x 5"),
        WaitFinished,
        Reset,
        CheckButtons("Run", false),
        CheckDrawn(0),
        CheckConsoleExactly(&[]),
        CheckCurrentLine(None),
        Console("PRINT :x"),
        WaitFinished,
        CheckConsole("In the console: :x has no value yet. Give it one with MAKE \"x 100"),
        CheckButtons("Run", true),
        SetSpeed(Speed::Animated(20.0)),
        Console("FOREVER [FD 1 RT 1]"),
        Wait(ms(200)),
        CheckButtons("Run", true),
        CheckCurrentLine(None),
        Screenshot("running-pause-is-dark"),
        EndRun,
        Say("Ending a run at Instant"),
        Run(FOREVER, Speed::Instant),
        Wait(ms(500)),
        EndRun,
        CheckStopsWithin(ms(50)),
        Say("Ending a run with the interpreter blocked on a full channel"),
        Run(FOREVER, Speed::Animated(20.0)),
        Wait(ms(500)),
        EndRun,
        CheckStopsWithin(ms(50)),
        Say("Ending a run while paused at a line"),
        Run(FOREVER_LINES, Speed::Animated(100.0)),
        Wait(ms(300)),
        Pause,
        Wait(ms(200)),
        EndRun,
        CheckStopsWithin(ms(50)),
        Say("A 131,070-move tree at Instant"),
        Run(TREE, Speed::Instant),
        WaitFinished,
        CheckFinishedWithin(Duration::from_secs(3)),
        CheckFramesAtMost(ms(20)),
        Screenshot("tree"),
        Say("Errors show with their call chain"),
        Run(ERRORS, Speed::Instant),
        WaitFinished,
        CheckConsole("Line 3: FD needs a number, but got the word far"),
        CheckConsole("    Written like this: FD distance"),
        CheckConsole("    Example: FD 100"),
        CheckConsole("    in BOX (line 6)"),
        Run("FD 10\nRT", Speed::Instant),
        WaitFinished,
        CheckConsole("Line 2: RT needs 1 input, but got none"),
        Say("Phase 2: filled shapes, FOR and a template, animated"),
        Run(
            "FOR [i 3 1] [SETFC ITEM :i [red orange gold] CIRCLE :i * 50]\nPRINT MAP [? * 2] [1 2 3]",
            Speed::Animated(2560.0),
        ),
        WaitFinished,
        CheckDrawn(3),
        CheckConsole("2 4 6"),
        Screenshot("filled"),
        Say("Phase 2: CATCH \"ERROR, and an error made by THROW"),
        Run(CATCHING, Speed::Instant),
        WaitFinished,
        CheckConsole("[0 FD needs a number, but got the word x [] 1]"),
        CheckConsole("Line 6: CHECK needs a positive number"),
        Say("Phase 3: a slow run draws only the frames that change"),
        Run(SLOW, level_speed(3)),
        Wait(ms(300)),
        MeasureRenders,
        Wait(ms(3000)),
        CheckRenders(25.0),
        EndRun,
        Say("Phase 3: the metrics overlay"),
        ToggleMetrics,
        Run("REPEAT 36 [REPEAT 4 [FD 200 RT 90] RT 10]", level_speed(10)),
        Wait(ms(1200)),
        CheckOverlay(true),
        Screenshot("metrics"),
        EndRun,
        ToggleMetrics,
        CheckOverlay(false),
        Say("Phase 4: the counters and variables keep in step with the highlight"),
        Run(WATCHED, Speed::Animated(100.0)),
        Pause,
        SingleStep,
        WaitPaused,
        CheckHighlight("MAKE \"sizes [40 80]"),
        CheckPanels(""),
        SingleStep,
        WaitPaused,
        CheckPanels("sizes=[40 80]"),
        SingleStep,
        WaitPaused,
        CheckHighlight("TILE ?"),
        CheckPanels("FOREACH 1 of 2* | sizes=[40 80]"),
        SingleStep,
        WaitPaused,
        SingleStep,
        WaitPaused,
        CheckHighlight("FD :size"),
        CheckPanels("FOREACH 1 of 2 | REPEAT 1 of 4* | sizes=[40 80] | TILE: size=40"),
        Wait(ms(100)),
        CheckPanelsDrawn,
        Screenshot("panels"),
        SetSpeed(Speed::Instant),
        Resume,
        WaitFinished,
        CheckPanels("sizes=[40 80]"),
        Say("Phase 4: at Instant the panels update as it runs, and Stop leaves the globals"),
        Run(COUNTING, Speed::Instant),
        Wait(ms(300)),
        CheckPanelsStart("FOREVER "),
        Wait(ms(100)),
        CheckPanelsDrawn,
        Screenshot("panels-instant"),
        EndRun,
        CheckPanelsStart("n="),
        Wait(ms(100)),
        CheckPanelsDrawn,
        Say("Phase 4: the colours, with a square of each"),
        Run(
            "SETPC \"orange\nSETBG [20 40 80]\nSETFC \"#40c0ff80\nCIRCLE 100",
            Speed::Animated(200.0),
        ),
        WaitFinished,
        CheckColours("PENCOLOR=orange FILLCOLOR=#40c0ff80 BACKGROUND=[20 40 80]"),
        Screenshot("colours"),
        Say("Phase 5: help for the word under the cursor"),
        Type(crate::app::WELCOME),
        F1At("FD :size"),
        Wait(ms(500)),
        CheckHelpPage("FORWARD"),
        Screenshot("help"),
        CursorAt("RT 90"),
        Wait(ms(100)),
        CheckHelpPage("RIGHT"),
        CursorAt(":size"),
        Wait(ms(100)),
        CheckHelpPage("Variables: :name"),
        Say("Phase 5: search and contents"),
        SearchHelp("setp"),
        Wait(ms(100)),
        CheckFirstResult("SETPENCOLOR"),
        Screenshot("help-search"),
        SearchHelp(""),
        OpenHelp("circle"),
        HelpContents,
        Wait(ms(100)),
        Screenshot("help-contents"),
        Say("Phase 5: an example runs on the canvas, and the user's drawing comes back"),
        Run(crate::app::WELCOME, Speed::Instant),
        WaitFinished,
        CheckDrawn(48),
        OpenHelp("forward"),
        SetSpeed(Speed::Animated(20.0)),
        RunExample("forward", 0),
        Wait(ms(900)),
        CheckExampleHighlight("FD 40"),
        Screenshot("example"),
        SetSpeed(Speed::Instant),
        WaitFinished,
        CheckDrawn(8),
        BackToMyDrawing,
        CheckDrawn(48),
        CopyExample("forward", 1),
        CheckEditorEnds("FLOWER 250\n\nREPEAT 5 [FD 20 PU FD 20 PD]\n"),
        CloseHelp,
        Say("Phase 5: errors offer help on their command"),
        Run("REPEAT 3 [FD]", Speed::Instant),
        WaitFinished,
        CheckErrorHelp("FORWARD"),
        Run("EDIT \"square", Speed::Instant),
        WaitFinished,
        CheckConsole(
            "Line 1: EDIT isn't in Pendown: it opens an external text editor, and Pendown has its own.",
        ),
        CheckErrorHelp("EDIT"),
        Wait(ms(100)),
        Screenshot("error-help"),
        Say("Phase 5: a lesson's programs load into the editor and run"),
        Lesson(0),
        Wait(ms(100)),
        Screenshot("tutorial-first"),
        Lesson(1),
        LoadLesson(1, 0),
        CheckEditor("REPEAT 4 [FD 100 RT 90]"),
        CheckTitle("Untitled — Pendown"),
        SetSpeed(level_speed(7)),
        RunLesson(1, 3),
        Wait(ms(1500)),
        Screenshot("tutorial"),
        SetSpeed(Speed::Instant),
        WaitFinished,
        CheckDrawn(32),
        BackToMyDrawing,
        Say("Tutorial: Help Kame fix the bug, with what Kame meant and the bug it drew"),
        Lesson(3),
        Wait(ms(500)),
        Screenshot("tutorial-bug"),
        RunLesson(3, 0),
        WaitFinished,
        // The box's four sides, and the triangle's three, inside it.
        CheckDrawn(7),
        Screenshot("tutorial-bug-run"),
        BackToMyDrawing,
        Say("Tutorial: lesson 8, L-systems"),
        Lesson(7),
        Wait(ms(500)),
        Screenshot("tutorial-l-systems"),
        // "Rules for growing a line": its program, pictures and text block.
        ScrollLesson(330.0),
        Wait(ms(500)),
        Screenshot("tutorial-l-systems-reading"),
        ScrollLesson(680.0),
        Wait(ms(500)),
        Screenshot("tutorial-l-systems-bump"),
        ScrollLesson(1050.0),
        Wait(ms(500)),
        Screenshot("tutorial-l-systems-width"),
        ScrollLesson(1600.0),
        Wait(ms(500)),
        Screenshot("tutorial-l-systems-rewriting"),
        ScrollLesson(0.0),
        // By its name, so a new program earlier in the lesson doesn't move it.
        RunLesson(
            7,
            pendown::tutorial::lessons()[7]
                .program_named("gasket")
                .expect("lesson 8 has the gasket"),
        ),
        WaitFinished,
        // The gasket at level 4: 81 triangles of 3 sides, and the 120
        // lines between corners at levels 1 to 4.
        CheckDrawn(363),
        Screenshot("tutorial-l-systems-run"),
        BackToMyDrawing,
        CloseHelp,
        Say("Gallery: examples from the help, the lessons and its own, to run or load"),
        Run(crate::app::WELCOME, Speed::Instant),
        WaitFinished,
        Gallery,
        Wait(ms(3000)),
        CheckGalleryDrawn,
        Screenshot("gallery"),
        SetSpeed(Speed::Instant),
        RunGallery("A spider's web"),
        WaitFinished,
        CheckShowing("“A spider's web”, from the gallery"),
        ScrollGallery(560.0),
        Wait(ms(200)),
        Screenshot("gallery-run"),
        ScrollGallery(0.0),
        BackToMyDrawing,
        CheckDrawn(48),
        RunGallery("Bigger and bigger squares"),
        WaitFinished,
        CheckShowing("“Bigger and bigger squares”, from FOR"),
        CheckDrawn(10),
        BackToMyDrawing,
        LoadGallery("A tree"),
        CheckEditor(
            "TO TREE :size\n  IF :size < 5 [STOP]\n  FD :size\n  LT 30\n  TREE :size * 0.7\n  RT 60\n  TREE :size * 0.7\n  LT 30\n  BK :size\nEND\n\nPU BK 200 PD\nTREE 120",
        ),
        CheckTitle("Untitled — Pendown"),
        LoadGallery("A friendly ghost"),
        CheckTitle("Untitled — Pendown"),
        ScrollGallery(600.0),
        Lesson(0),
        Gallery,
        Wait(ms(100)),
        CheckGalleryScroll(600.0),
        GalleryFrom("A tree"),
        CheckTab("Tutorial"),
        Gallery,
        GalleryFrom("A flower of squares"),
        CheckTab("Help"),
        CheckHelpPage("REPEAT"),
        CloseHelp,
        Say("Shortcuts: a tab of every key, from the keymap, and F5 runs"),
        Shortcuts,
        Wait(ms(300)),
        CheckTab("Shortcuts"),
        CheckShortcutsBound,
        // As Linux writes them; on a Mac, Ctrl is ⌘.
        CheckShortcut("Run your program", &["Ctrl+Enter", "F5"]),
        CheckShortcut("Undo", &["Ctrl+Z"]),
        CheckShortcut("Cut, copy and paste", &["Ctrl+X", "Ctrl+C", "Ctrl+V"]),
        CheckShortcut("Undo the last move", &["Backspace", "Ctrl+Z"]),
        CheckShortcut("Turn left", &["←"]),
        CheckShortcut("Pause, or carry on when paused", &["F6"]),
        CheckShortcut("Single step: one instruction, then pause", &["F10"]),
        CheckShortcut("Reset: stop, and put everything back", &["Shift+F5"]),
        Screenshot("shortcuts"),
        SetSpeed(Speed::Instant),
        Run("REPEAT 4 [FD 100 RT 90]", Speed::Instant),
        WaitFinished,
        Console("CS"),
        WaitFinished,
        CheckDrawn(0),
        // In the editor, where you'd press it. Pressed on the next frame,
        // so the run starts after a moment.
        EditorKeys("f5"),
        Wait(ms(300)),
        WaitFinished,
        CheckDrawn(4),
        Console("CS"),
        WaitFinished,
        EditorKeys("secondary-enter"),
        Wait(ms(300)),
        WaitFinished,
        CheckDrawn(4),
        // F6 pauses and carries on, F10 steps, Shift+F5 resets.
        Run("FD 300\nRT 90\nFD 300", Speed::Animated(50.0)),
        Wait(ms(1500)),
        EditorKeys("f6"),
        Wait(ms(300)),
        WaitPaused,
        CheckPaused(true),
        CheckHighlight("RT 90"),
        // After the toolbar has redrawn as Resume.
        Wait(ms(100)),
        HoverAt(323.0, 20.0),
        Wait(ms(1000)),
        Screenshot("resume-tooltip"),
        HoverAt(700.0, 400.0),
        EditorKeys("f10"),
        Wait(ms(300)),
        WaitPaused,
        CheckHighlight("FD 300"),
        EditorKeys("f6"),
        Wait(ms(300)),
        CheckPaused(false),
        CheckMoving,
        EditorKeys("shift-f5"),
        Wait(ms(300)),
        CheckPaused(false),
        CheckDrawn(0),
        Wait(ms(100)),
        HoverAt(228.0, 20.0),
        Wait(ms(1000)),
        Screenshot("run-tooltip"),
        HoverAt(700.0, 400.0),
        CloseHelp,
        Say("Console undo: an oops at the console, taken back"),
        Run("", Speed::Instant),
        WaitFinished,
        Console("FD 100"),
        WaitFinished,
        Console("RT 90"),
        WaitFinished,
        Console("FD 100"),
        WaitFinished,
        Console("RT 45"),
        WaitFinished,
        CheckTurtle(100.0, 100.0, 135.0),
        Screenshot("undo-before"),
        UndoConsole,
        WaitFinished,
        CheckTurtle(100.0, 100.0, 90.0),
        CheckDrawn(2),
        CheckConsole("Undid: RT 45"),
        Console("RT 90"),
        WaitFinished,
        Console("FD 100"),
        WaitFinished,
        CheckDrawn(3),
        Screenshot("undo-after"),
        Say("Console undo: Ctrl+Z with nothing typed, back past a CLEAN and a colour"),
        Console("SETPC \"orange CLEAN FD 50 RT 90 FD 50"),
        WaitFinished,
        CheckDrawn(2),
        CheckColours("PENCOLOR=orange FILLCOLOR=none BACKGROUND=0"),
        ConsoleKeys("secondary-z"),
        Wait(ms(200)),
        WaitFinished,
        CheckDrawn(3),
        CheckTurtle(100.0, 0.0, 180.0),
        CheckColours("PENCOLOR=7 FILLCOLOR=none BACKGROUND=0"),
        CheckConsole("Undid: SETPC \"orange CLEAN FD 50 RT 90 FD 50"),
        Say("Console undo: an entry that stopped with an error"),
        Console("FD 30 FD \"far"),
        WaitFinished,
        CheckDrawn(4),
        CheckTurtle(100.0, -30.0, 180.0),
        UndoConsole,
        WaitFinished,
        CheckDrawn(3),
        CheckTurtle(100.0, 0.0, 180.0),
        Say("Console undo: with something typed, Ctrl+Z undoes the typing"),
        ConsoleKeys("x"),
        Wait(ms(100)),
        CheckConsoleTyped("x"),
        ConsoleKeys("secondary-z"),
        Wait(ms(200)),
        CheckDrawn(3),
        CheckConsoleTyped(""),
        Say("Console undo: an entry that was stopped"),
        SetSpeed(Speed::Animated(50.0)),
        Console("FOREVER [FD 1 RT 1]"),
        Wait(ms(500)),
        EndRun,
        UndoConsole,
        WaitFinished,
        CheckDrawn(3),
        CheckTurtle(100.0, 0.0, 180.0),
        SetSpeed(Speed::Instant),
        Say("Teach Kame: the entries since the last Run become a procedure"),
        Console("FD"),
        WaitFinished,
        OpenTeach,
        Wait(ms(300)),
        TeachName("fd"),
        Wait(ms(100)),
        Screenshot("teach-name-taken"),
        TeachName("house"),
        // Untick the second and fourth, then Shift-click the second: the
        // second to the fourth are ticked again. The error stays unticked.
        TeachToggle(1, false),
        TeachToggle(3, false),
        TeachToggle(1, true),
        Wait(ms(500)),
        Screenshot("teach"),
        PressTeach,
        Wait(ms(300)),
        WaitFinished,
        CheckEditor("TO HOUSE\n  FD 100\n  RT 90\n  FD 100\n  RT 90\n  FD 100\nEND\n"),
        CheckConsole("Kame knows HOUSE now. It's at the end of your program."),
        CheckTurtle(100.0, 0.0, 180.0),
        Console("RT 45"),
        WaitFinished,
        Console("HOUSE"),
        WaitFinished,
        CheckDrawn(6),
        Screenshot("taught"),
        Say("Teach Kame: Undo goes back past the definition"),
        UndoConsole,
        WaitFinished,
        UndoConsole,
        WaitFinished,
        CheckDrawn(3),
        CheckTurtle(100.0, 0.0, 180.0),
        Say("Console undo: the background comes back"),
        Run("", Speed::Instant),
        WaitFinished,
        Console("FD 10"),
        WaitFinished,
        Console("SETBG 1 FD 10"),
        WaitFinished,
        CheckColours("PENCOLOR=7 FILLCOLOR=none BACKGROUND=1"),
        UndoConsole,
        WaitFinished,
        CheckBackground([0, 0, 0, 255]),
        CheckColours("PENCOLOR=7 FILLCOLOR=none BACKGROUND=0"),
        CheckDrawn(1),
        Say("Console undo: long entries are cut short, and Run leaves nothing to undo"),
        Run("", Speed::Instant),
        WaitFinished,
        CheckUndoLabel(""),
        CheckCanTeach(false),
        Console("TO TRI :n"),
        Console("REPEAT 3 [FD :n RT 120]"),
        Console("END"),
        WaitFinished,
        CheckUndoLabel("TO TRI :n…"),
        CheckCanTeach(false),
        Console("REPEAT 36 [FD 10 RT 10 FD 5]"),
        WaitFinished,
        CheckUndoLabel("REPEAT 36 [FD 10 RT 10…"),
        CheckCanTeach(true),
        Run("FD 10", Speed::Instant),
        WaitFinished,
        CheckUndoLabel(""),
        CheckCanTeach(false),
        Say("Console undo: with an example showing, your drawing comes back first"),
        Run("", Speed::Instant),
        WaitFinished,
        Console("FD 50"),
        WaitFinished,
        Console("RT 90 FD 50"),
        WaitFinished,
        CheckDrawn(2),
        OpenHelp("forward"),
        RunExample("forward", 0),
        WaitFinished,
        CheckDrawn(8),
        UndoConsole,
        WaitFinished,
        CheckDrawn(1),
        CheckTurtle(0.0, 50.0, 0.0),
        CloseHelp,
        Say("Teach Kame: rows that can't be ticked, notes, All and None"),
        Run("", Speed::Instant),
        WaitFinished,
        Console("TO TRI :n"),
        Console("REPEAT 3 [FD :n RT 120]"),
        Console("END"),
        WaitFinished,
        Console("TRI 40"),
        WaitFinished,
        Console("FD"),
        WaitFinished,
        Console("RT 30"),
        WaitFinished,
        OpenTeach,
        Wait(ms(800)),
        CheckTeachRows(
            "# TO TRI :n REPEAT 3 [FD :n RT 120] END | x TRI 40 (uses TRI) | - FD (FD needs 1 input, but got none) | x RT 30",
        ),
        CheckTeachPicture,
        TeachName("tri"),
        Wait(ms(100)),
        CheckTeachProblem("TRI is already defined at the console. Choose another name."),
        Screenshot("teach-rows"),
        TeachAll(false),
        CheckTeachRows(
            "# TO TRI :n REPEAT 3 [FD :n RT 120] END | - TRI 40 (uses TRI) | - FD (FD needs 1 input, but got none) | - RT 30",
        ),
        TeachAll(true),
        CheckTeachRows(
            "# TO TRI :n REPEAT 3 [FD :n RT 120] END | x TRI 40 (uses TRI) | x FD (FD needs 1 input, but got none) | x RT 30",
        ),
        Say("Teach Kame: Escape closes the dialog, and nothing changes"),
        Keys("escape"),
        Wait(ms(400)),
        CheckDialog(false),
        CheckEditor(""),
        CheckCanTeach(true),
        Say("Teach Kame: the editor's Undo takes the procedure back out"),
        Run("; mine", Speed::Instant),
        WaitFinished,
        Console("FD 20"),
        WaitFinished,
        OpenTeach,
        Wait(ms(300)),
        TeachName("stair"),
        PressTeach,
        Wait(ms(300)),
        WaitFinished,
        CheckDialog(false),
        CheckEditor("; mine\n\nTO STAIR\n  FD 20\nEND\n"),
        EditorKeys("secondary-z"),
        Wait(ms(300)),
        CheckEditor("; mine"),
        Say("Teach Kame: so does Copy to editor's, for definitions typed at the console"),
        Console("TO STEP2"),
        Console("FD 5"),
        Console("END"),
        WaitFinished,
        CopyDefinitions,
        Wait(ms(100)),
        // TRI too, from earlier: Copy to editor keeps definitions across a
        // Run, but only the newest of each name.
        CheckEditor("; mine\n\nTO TRI :n\nREPEAT 3 [FD :n RT 120]\nEND\n\nTO STEP2\nFD 5\nEND\n"),
        EditorKeys("secondary-z"),
        Wait(ms(300)),
        CheckEditor("; mine"),
        Say("Colours: the page, found by searching for color"),
        Run("FD 100", Speed::Instant),
        WaitFinished,
        Type("SETPC\nFD 100"),
        ShowPanel,
        SearchHelp("color"),
        Wait(ms(100)),
        CheckFirstResult("Colours"),
        SearchHelp(""),
        OpenHelp("colours"),
        Wait(ms(500)),
        CheckCurrentColour("\"#ffffff"),
        Screenshot("colours"),
        Say("Colours: the fields set the colour, and Copy to editor puts it at the cursor"),
        ColourField(4, "#ff9a26"),
        Wait(ms(100)),
        CheckCurrentColour("\"#ff9a26"),
        CursorAt("C\n"),
        CopyColour,
        Wait(ms(200)),
        CheckEditor("SETPC \"#ff9a26\nFD 100"),
        EditorKeys("secondary-z"),
        Wait(ms(300)),
        CheckEditor("SETPC\nFD 100"),
        ChooseForm("RGBA"),
        CheckCurrentColour("[255 154 38 255]"),
        Say("Colours: a field that doesn't make sense leaves the colour as it was"),
        ColourField(0, "300"),
        Wait(ms(100)),
        CheckColourProblem("R goes from 0 to 255."),
        CheckCurrentColour("[255 154 38 255]"),
        ScrollHelp(380.),
        Wait(ms(200)),
        Screenshot("colours-problem"),
        ColourField(0, "64"),
        Wait(ms(100)),
        CheckColourProblem(""),
        CheckCurrentColour("[64 154 38 255]"),
        Say("Colours: the wheel, the palette and the names"),
        WheelAt(0.5, 0.0),
        CheckCurrentColour("[154 0 0 255]"),
        PickPalette(4),
        CheckCurrentColour("4"),
        PickName("tomato"),
        CheckCurrentColour("\"tomato"),
        FilterNames("red"),
        ScrollHelp(1000.),
        Wait(ms(200)),
        Screenshot("colours-names"),
        CursorAt("C\n"),
        CopyColour,
        Wait(ms(200)),
        CheckEditor("SETPC \"tomato\nFD 100"),
        Run("SETPC \"tomato\nFD 100", Speed::Instant),
        WaitFinished,
        CheckColours("PENCOLOR=tomato FILLCOLOR=none BACKGROUND=0"),
        Screenshot("colours-copied"),
        CloseHelp,
        Say("Show thinking: an arc on a turn, and a ruler on a move"),
        SetThinking(true),
        // At 80 steps a second, FD 100 takes 1.25 s and RT 120 0.5 s.
        Run("FD 100 RT 120 FD 100", Speed::Animated(80.0)),
        Wait(ms(1500)),
        Screenshot("thinking-turn"),
        Wait(ms(900)),
        Screenshot("thinking-move"),
        SetSpeed(Speed::Instant),
        WaitFinished,
        // On a light background it's dark amber, so it still shows.
        Run("SETBG 7 SETPC 0 FD 100 RT 120", Speed::Animated(80.0)),
        Wait(ms(1500)),
        Screenshot("thinking-light"),
        SetSpeed(Speed::Instant),
        WaitFinished,
        Say("When a run ends, the highlight and Kame's thinking go"),
        Run("FD 20 RT 90", Speed::Animated(80.0)),
        Wait(ms(100)),
        CheckEditorHighlight(true),
        WaitFinished,
        Wait(ms(500)),
        CheckEditorHighlight(false),
        CheckDriveThinking("nothing"),
        Screenshot("run-ended"),
        SetThinking(false),
        Say("Drive Kame: the keys make console entries, and the same key joins up"),
        Run("", Speed::Instant),
        WaitFinished,
        Drive(true),
        CheckDriving(true),
        DriveKeys(&["up", "up", "up", "up", "up", "up", "up", "up", "up", "up"]),
        WaitDriven,
        CheckConsole("> FD 100"),
        CheckTurtle(0.0, 100.0, 0.0),
        // Each press draws its own line: ten for FD 100, which look as one.
        CheckDrawn(10),
        DriveKeys(&["right", "right", "right", "right", "right", "right"]),
        WaitDriven,
        CheckConsole("> RT 90"),
        CheckTurtle(0.0, 100.0, 90.0),
        DriveKeys(&["up", "up", "up", "up", "up", "up", "up", "up", "up", "up"]),
        WaitDriven,
        CheckTurtle(100.0, 100.0, 90.0),
        DriveKeys(&["right", "right", "right"]),
        WaitDriven,
        CheckUndoLabel("RT 45"),
        Wait(ms(100)),
        Screenshot("driving"),
        Say("Drive Kame: Backspace takes back the whole entry, and Space lifts the pen"),
        DriveKeys(&["backspace"]),
        WaitDriven,
        CheckTurtle(100.0, 100.0, 90.0),
        CheckUndoLabel("FD 100"),
        DriveKeys(&["space", "up", "space"]),
        WaitDriven,
        CheckDrawn(20),
        CheckTurtle(110.0, 100.0, 90.0),
        CheckConsole("> PU"),
        CheckConsole("> PD"),
        Say("Drive Kame: Teach Kame what you drove"),
        OpenTeach,
        Wait(ms(300)),
        TeachName("corner"),
        PressTeach,
        Wait(ms(300)),
        WaitDriven,
        CheckEditor("TO CORNER\n  FD 100\n  RT 90\n  FD 100\n  PU\n  FD 10\n  PD\nEND\n"),
        Say("Drive Kame: Esc and Run stop driving"),
        DriveKeys(&["escape"]),
        CheckDriving(false),
        Drive(true),
        Run("FD 1", Speed::Instant),
        WaitFinished,
        CheckDriving(false),
        Say("Drive Kame: the steps, and presses waiting their turn"),
        Run("", Speed::Instant),
        WaitFinished,
        Drive(true),
        DriveSteps(50.0, 90.0),
        DriveKeys(&["up", "up", "up"]),
        CheckDriveQueue(2),
        WaitDriven,
        CheckDriveQueue(0),
        CheckConsole("> FD 150"),
        CheckTurtle(0.0, 150.0, 0.0),
        DriveKeys(&["right"]),
        WaitDriven,
        CheckConsole("> RT 90"),
        Screenshot("driving-steps"),
        Say("Drive Kame: typing at the console ends a driven entry"),
        DriveSteps(10.0, 15.0),
        DriveKeys(&["up"]),
        WaitDriven,
        Console("PRINT 1"),
        WaitFinished,
        DriveKeys(&["up"]),
        WaitDriven,
        CheckUndoLabel("FD 10"),
        CheckTurtle(20.0, 150.0, 90.0),
        Say("Drive Kame: the last entry's thought fades as the next starts"),
        DriveKeys(&["right"]),
        // Before its first frame, the turn hasn't started; the move fades.
        CheckDriveThinking("nothing, fading move 10"),
        WaitDriven,
        Wait(ms(500)),
        CheckDriveThinking("turn 15"),
        Say("Drive Kame: a held key builds up no queue, and letting go stops Kame"),
        DriveSteps(50.0, 90.0),
        DriveKeysDown(&["up", "up", "up", "up", "up", "up"]),
        CheckDriveQueue(1),
        DriveKeyUp("up"),
        CheckDriveQueue(0),
        WaitDriven,
        CheckConsole("> FD 50"),
        // Still held when the press ends: its repeat plays, and only that.
        DriveKeysDown(&["up", "up", "up", "up", "up", "up"]),
        WaitDriven,
        CheckConsole("> FD 150"),
        DriveKeyUp("up"),
        // Holding Space flips the pen once.
        DriveKeysDown(&["space", "space", "space"]),
        WaitDriven,
        CheckConsole("> PU"),
        DriveKeyUp("space"),
        DriveKeys(&["space"]),
        WaitDriven,
        CheckConsole("> PD"),
        // Holding Backspace takes back one entry.
        DriveKeysDown(&["backspace", "backspace", "backspace"]),
        WaitDriven,
        CheckUndoLabel("PU"),
        DriveKeyUp("backspace"),
        DriveKeys(&["escape"]),
        Quit,
    ]
}

pub struct Autotest {
    steps: Vec<Step>,
    next: usize,
    waiting: Option<(Step, Instant)>,
    log: String,
    failures: usize,
    dir: PathBuf,
    screenshots: usize,
    trace: Option<Vec<String>>,
    track: Option<Vec<(Instant, f64)>>,
    /// The metrics, highlight changes and console lines when `MeasureRenders` ran.
    measuring: Option<(Snapshot, u64, usize)>,
    highlights: u64,
}

impl Autotest {
    pub fn from_env() -> Option<Autotest> {
        let value = std::env::var("PENDOWN_AUTOTEST").ok()?;
        let dir = if value == "1" {
            PathBuf::from("target/autotest")
        } else {
            PathBuf::from(value)
        };
        std::fs::create_dir_all(&dir).ok();
        let steps = script();
        // PENDOWN_AUTOTEST_FROM="Phase 5" starts at the first section whose
        // title has that in it.
        let next = std::env::var("PENDOWN_AUTOTEST_FROM")
            .ok()
            .and_then(|from| {
                steps
                    .iter()
                    .position(|s| matches!(s, Step::Say(text) if text.contains(from.as_str())))
            })
            .unwrap_or(0);
        Some(Autotest {
            steps,
            next,
            waiting: None,
            log: String::new(),
            failures: 0,
            dir,
            screenshots: 0,
            trace: None,
            track: None,
            measuring: None,
            highlights: 0,
        })
    }
}

impl App {
    fn say(&mut self, line: impl Into<String>) {
        let line = line.into();
        println!("{line}");
        if let Some(auto) = &mut self.autotest {
            auto.log.push_str(&line);
            auto.log.push('\n');
        }
    }

    fn check(&mut self, ok: bool, what: String) {
        self.say(format!("  {} {what}", if ok { "ok  " } else { "FAIL" }));
        if !ok {
            self.autotest
                .as_mut()
                .expect("running the autotest")
                .failures += 1;
        }
    }

    /// Called at every frame that plays the animation.
    pub(crate) fn on_frame(&mut self, now: Instant) {
        let y = self.animator.as_ref().map_or(0.0, |a| a.pos.y);
        if let Some(track) = self.autotest.as_mut().and_then(|a| a.track.as_mut()) {
            track.push((now, y));
        }
    }

    /// Called whenever the highlight moves.
    pub(crate) fn on_highlight(&mut self) {
        if let Some(auto) = &mut self.autotest {
            auto.highlights += 1;
        }
        let text = self.highlighted();
        if let Some(trace) = self.autotest.as_mut().and_then(|a| a.trace.as_mut()) {
            trace.push(text);
        }
    }

    /// The editor text the highlight is on. Console entries aren't in the
    /// editor, so their spans don't index `run_text`.
    fn highlighted(&self) -> String {
        self.animator
            .as_ref()
            .and_then(|a| a.highlight)
            .filter(|s| s.source == SourceId::Editor)
            .map_or(String::new(), |s| self.run_text[s.range()].to_string())
    }

    /// Takes the session's next steps, until one has to wait. Returns
    /// whether it took any, so the window should render.
    pub(crate) fn drive_autotest(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.autotest.is_none() {
            return false;
        }
        let mut took = false;
        loop {
            let auto = self.autotest.as_mut().unwrap();
            if let Some((step, since)) = auto.waiting.clone() {
                let done = match step {
                    Step::Wait(d) => since.elapsed() >= d,
                    Step::WaitDriven => {
                        (self.animator.as_ref().is_none_or(|a| a.finished())
                            && self
                                .driving
                                .as_ref()
                                .is_none_or(|d| d.presses.waiting() == 0))
                            || since.elapsed() > Duration::from_secs(20)
                    }
                    Step::WaitFinished => {
                        self.animator.as_ref().is_none_or(|a| a.finished())
                            || since.elapsed() > Duration::from_secs(60)
                    }
                    Step::WaitPaused => {
                        self.animator
                            .as_ref()
                            .is_none_or(|a| a.mode() == Mode::Paused || a.finished())
                            || since.elapsed() > Duration::from_secs(10)
                    }
                    Step::CheckStopsWithin(limit) => {
                        let exited = self.run.as_ref().and_then(|r| r.exited());
                        match (exited, self.stop_at) {
                            (Some(exited), Some(stop)) => {
                                let took = exited.saturating_duration_since(stop);
                                self.check(
                                    took <= limit,
                                    format!(
                                        "the interpreter stopped {:.3} ms after Stop",
                                        took.as_secs_f64() * 1e3
                                    ),
                                );
                                true
                            }
                            _ if since.elapsed() > Duration::from_secs(2) => {
                                self.check(false, "the interpreter didn't stop within 2 s".into());
                                true
                            }
                            _ => false,
                        }
                    }
                    _ => true,
                };
                if !done {
                    return took;
                }
                self.autotest.as_mut().unwrap().waiting = None;
                continue;
            }
            let Some(step) = auto.steps.get(auto.next).cloned() else {
                return took;
            };
            auto.next += 1;
            took = true;
            match step.clone() {
                Step::Say(text) => self.say(format!("\n{text}")),
                Step::Run(program, speed) => {
                    self.set_speed(speed, window, cx);
                    self.editor.update(cx, |state, cx| {
                        state.set_value(program.to_string(), window, cx)
                    });
                    self.start(DEFAULT_CHANNEL, cx);
                }
                Step::Wait(_)
                | Step::WaitFinished
                | Step::WaitDriven
                | Step::WaitPaused
                | Step::CheckStopsWithin(_) => {
                    self.autotest.as_mut().unwrap().waiting = Some((step, Instant::now()));
                }
                Step::SetSpeed(speed) => self.set_speed(speed, window, cx),
                Step::Pause => self.animator.as_mut().map_or((), |a| a.pause()),
                // As the toolbar does, so a changed program replays.
                Step::Resume => self.resume(cx),
                Step::SingleStep => self.single_step(cx),
                Step::EndRun => self.stop(cx),
                Step::Screenshot(name) => self.screenshot(name),
                Step::Track => self.autotest.as_mut().unwrap().track = Some(Vec::new()),
                Step::CheckSpeeds(at, before, after) => {
                    let track = self
                        .autotest
                        .as_mut()
                        .unwrap()
                        .track
                        .take()
                        .unwrap_or_default();
                    self.check_speeds(&track, at, before, after);
                }
                Step::Trace => self.autotest.as_mut().unwrap().trace = Some(Vec::new()),
                Step::CheckTrace(expected) => {
                    let trace = self
                        .autotest
                        .as_mut()
                        .unwrap()
                        .trace
                        .take()
                        .unwrap_or_default();
                    self.check(
                        trace == expected,
                        format!("the highlight went through {trace:?}"),
                    );
                }
                Step::CheckHighlight(expected) => {
                    let text = self.highlighted();
                    self.check(text == expected, format!("the highlight is on {text:?}"));
                }
                Step::CheckMoving => {
                    let moving = self
                        .animator
                        .as_ref()
                        .is_some_and(|a| a.partial(1.0).is_some());
                    self.check(moving, "the turtle is part-way through the move".into());
                }
                Step::CheckEditorHighlight(shown) => {
                    let found = !self.highlight_ranges(cx).is_empty();
                    self.check(
                        found == shown,
                        format!(
                            "the editor's highlight is {}",
                            if found { "shown" } else { "hidden" }
                        ),
                    );
                }
                Step::CheckEditable(editable) => {
                    let readonly = self.editor_pane.read(cx).readonly();
                    self.check(
                        readonly != editable,
                        format!(
                            "the editor is {}",
                            if readonly { "read-only" } else { "editable" }
                        ),
                    );
                }
                Step::CheckPaused(expected) => {
                    let paused = self.paused();
                    self.check(
                        paused == expected,
                        format!(
                            "the run is {}",
                            if paused { "paused" } else { "not paused" }
                        ),
                    );
                }
                Step::ToggleBreakpoint(line) => {
                    self.gutter
                        .update(cx, |gutter, cx| gutter.toggle(line - 1, cx));
                }
                Step::CheckBreakpoints(expected) => {
                    let lines: Vec<usize> =
                        self.gutter.read(cx).lines.iter().map(|l| l + 1).collect();
                    self.check(lines == expected, format!("breakpoints on lines {lines:?}"));
                }
                Step::ScrollEditorTo(line) => {
                    self.editor.update(cx, |state, cx| {
                        let height = state.line_height().unwrap_or(px(20.));
                        let offset = state.scroll_offset();
                        state.set_scroll_offset(point(offset.x, -height * (line - 1) as f32), cx);
                    });
                }
                Step::Reset => self.reset(cx),
                Step::HoverAt(x, y) => {
                    window.dispatch_event(
                        PlatformInput::MouseMove(MouseMoveEvent {
                            position: point(px(x), px(y)),
                            pressed_button: None,
                            modifiers: Modifiers::default(),
                        }),
                        cx,
                    );
                }
                Step::CheckButtons(label, can_reset) => {
                    let found = if self.running() && self.editor_run {
                        "Restart"
                    } else {
                        "Run"
                    };
                    let resettable = !self.at_start;
                    self.check(
                        found == label && resettable == can_reset,
                        format!(
                            "the toolbar shows {found}, and Reset is {}",
                            if resettable { "enabled" } else { "greyed out" }
                        ),
                    );
                }
                Step::CheckCurrentLine(expected) => {
                    let line = self.gutter.read(cx).current().map(|l| l + 1);
                    self.check(
                        line == expected,
                        format!("the gutter's arrow is on line {line:?}"),
                    );
                }
                Step::RunAsIs(speed) => {
                    self.set_speed(speed, window, cx);
                    self.start(DEFAULT_CHANNEL, cx);
                }
                Step::CheckConsoleExactly(expected) => {
                    let lines: Vec<&str> = self.console.iter().map(|l| l.as_ref()).collect();
                    self.check(lines == expected, format!("the console holds {lines:?}"));
                }
                Step::CheckNotMoving => {
                    let moving = self.animator.as_ref().is_some_and(|a| a.in_progress());
                    self.check(!moving, "nothing is left part-way".into());
                }
                Step::CheckNothingAhead => {
                    let (ahead, text) = (
                        self.animator.as_ref().map_or(0, |a| a.lines_ahead()),
                        self.highlighted(),
                    );
                    self.check(
                        ahead == 0,
                        format!("paused at {text:?} with {ahead} lines run ahead"),
                    );
                }
                Step::CheckFinishedWithin(limit) => {
                    let took = match (&self.run, self.finished_at) {
                        (Some(run), Some(at)) => Some(at - run.started),
                        _ => None,
                    };
                    let ok = took.is_some_and(|t| t <= limit);
                    let what = took.map_or("didn't finish".into(), |t| {
                        format!("finished in {:.2} s", t.as_secs_f64())
                    });
                    let detail = format!(
                        "{what}, {} lines drawn, {} events",
                        self.canvas.borrow().drawing.segments(),
                        self.events
                    );
                    self.check(ok, detail);
                }
                Step::CheckFramesAtMost(limit) => {
                    let p50 = self.frame_intervals.percentile(50).unwrap_or_default();
                    let summary = self.frame_intervals.summary();
                    self.check(p50 <= limit, format!("frame intervals {summary}"));
                }
                Step::Console(line) => {
                    self.type_into_console(line, window, cx);
                }
                Step::ConsoleKeys(keys) => self.press_in_console(keys, window, cx),
                Step::UndoConsole => self.undo_console(cx),
                Step::OpenTeach => self.open_teach(window, cx),
                Step::TeachAll(ticked) => {
                    if let Some(dialog) = self.teach_dialog.clone() {
                        dialog.update(cx, |dialog, cx| dialog.tick_all(ticked, cx));
                    }
                }
                Step::CheckTeachRows(expected) => {
                    let rows = self
                        .teach_dialog
                        .as_ref()
                        .map_or(String::new(), |d| d.read(cx).describe());
                    self.check(rows == expected, format!("the dialog's rows are {rows:?}"));
                }
                Step::CheckTeachProblem(expected) => {
                    let problem = self
                        .teach_dialog
                        .as_ref()
                        .and_then(|d| d.read(cx).name_problem(cx))
                        .unwrap_or_default();
                    self.check(
                        problem == expected,
                        format!("the name's problem is {problem:?}"),
                    );
                }
                Step::CheckTeachPicture => {
                    let drawn = self
                        .teach_dialog
                        .as_ref()
                        .is_some_and(|d| d.read(cx).has_drawing());
                    self.check(drawn, format!("the dialog's picture is drawn: {drawn}"));
                }
                Step::CheckDialog(open) => {
                    let found = window.has_active_dialog(cx);
                    self.check(found == open, format!("a dialog is open: {found}"));
                }
                Step::CheckUndoLabel(expected) => {
                    let found = self
                        .to_undo()
                        .map(pendown::console::label)
                        .unwrap_or_default();
                    self.check(found == expected, format!("Undo names {found:?}"));
                }
                Step::CheckCanTeach(expected) => {
                    let found = self.can_teach();
                    self.check(found == expected, format!("Teach Kame is enabled: {found}"));
                }
                Step::Keys(keys) => self.press(keys, window, cx),
                Step::ScrollLesson(y) => {
                    self.help_panel.update(cx, |p, cx| {
                        p.lesson_scrolls[p.lesson].set_offset(point(px(0.), px(-y)));
                        cx.notify();
                    });
                }
                Step::ScrollHelp(y) => {
                    self.help_panel.update(cx, |p, cx| {
                        p.page_scroll.set_offset(point(px(0.), px(-y)));
                        cx.notify();
                    });
                }
                Step::Drive(on) => {
                    if self.driving.is_some() != on {
                        self.toggle_driving(window, cx);
                    }
                }
                Step::DriveKeys(keys) => {
                    // Through the drive key table, as a real key would be.
                    for key in keys {
                        let keystroke = crate::driving::keystroke(key, cx);
                        let handled = self.drive_keystroke(&keystroke, window, cx);
                        self.check(handled, format!("{key} is a drive key"));
                        self.drive_key_up(&keystroke);
                    }
                }
                Step::DriveKeysDown(keys) => {
                    for key in keys {
                        let keystroke = crate::driving::keystroke(key, cx);
                        let handled = self.drive_keystroke(&keystroke, window, cx);
                        self.check(handled, format!("{key} is a drive key"));
                    }
                }
                Step::DriveKeyUp(key) => {
                    self.drive_key_up(&crate::driving::keystroke(key, cx));
                }
                Step::DriveSteps(moves, turns) => {
                    self.drive_steps = pendown::drive::Steps { moves, turns };
                }
                Step::CheckDriveQueue(expected) => {
                    let found = self.driving.as_ref().map_or(0, |d| d.presses.waiting());
                    self.check(found == expected, format!("{found} presses waiting"));
                }
                Step::CheckDriveThinking(expected) => {
                    let found = self.thinking_described();
                    self.check(found == expected, format!("Kame is thinking: {found}"));
                }
                Step::CheckDriving(expected) => {
                    let found = self.driving.is_some();
                    self.check(found == expected, format!("driving: {found}"));
                }
                Step::SetThinking(on) => {
                    if self.show_thinking != on {
                        self.toggle_thinking(cx);
                    }
                }
                Step::ShowPanel => {
                    if !self.help_open {
                        self.toggle_help(cx);
                    }
                }
                Step::ColourField(i, value) => {
                    if let Some(page) = self.help_panel.read(cx).colours.clone() {
                        let field = page.read(cx).fields[i].clone();
                        field.update(cx, |f, cx| f.set_value(value.to_string(), window, cx));
                        // Setting a field's text isn't typing, so say it was.
                        page.update(cx, |p, cx| p.typed(i, window, cx));
                    }
                }
                Step::WheelAt(x, y) => {
                    if let Some(page) = self.help_panel.read(cx).colours.clone() {
                        page.update(cx, |p, cx| p.pick_on_wheel(x, y, window, cx));
                    }
                }
                Step::PickPalette(n) => {
                    if let Some(page) = self.help_panel.read(cx).colours.clone() {
                        page.update(cx, |p, cx| p.pick_palette(n, window, cx));
                    }
                }
                Step::PickName(name) => {
                    if let Some(page) = self.help_panel.read(cx).colours.clone() {
                        let found = pendown::colours::names_matching(name)
                            .into_iter()
                            .find(|(n, _)| *n == name);
                        if let Some((name, rgba)) = found {
                            page.update(cx, |p, cx| p.pick_name(name, rgba, window, cx));
                        }
                    }
                }
                Step::ChooseForm(label) => {
                    if let Some(page) = self.help_panel.read(cx).colours.clone() {
                        page.update(cx, |p, cx| {
                            let form = pendown::colours::forms(p.rgba())
                                .into_iter()
                                .find(|f| f.label() == label);
                            if let Some(form) = form {
                                p.choose(form, cx);
                            }
                        });
                    }
                }
                Step::CopyColour => {
                    if let Some(page) = self.help_panel.read(cx).colours.clone() {
                        // After this update, since copying updates the window.
                        window.defer(cx, move |window, cx| {
                            page.update(cx, |p, cx| p.copy(window, cx));
                        });
                    }
                }
                Step::CheckCurrentColour(expected) => {
                    let found = self
                        .help_panel
                        .read(cx)
                        .colours
                        .as_ref()
                        .map(|p| p.read(cx).text())
                        .unwrap_or_default();
                    self.check(found == expected, format!("the current colour is {found}"));
                }
                Step::CheckColourProblem(expected) => {
                    let found = self
                        .help_panel
                        .read(cx)
                        .colours
                        .as_ref()
                        .and_then(|p| p.read(cx).problem.clone())
                        .map(|(_, why)| why)
                        .unwrap_or_default();
                    self.check(
                        found == expected,
                        format!("the fields' problem is {found:?}"),
                    );
                }
                Step::FilterNames(text) => {
                    if let Some(page) = self.help_panel.read(cx).colours.clone() {
                        let filter = page.read(cx).filter.clone();
                        filter.update(cx, |f, cx| f.set_value(text.to_string(), window, cx));
                    }
                }
                Step::CopyDefinitions => self.copy_definitions(window, cx),
                Step::EditorKeys(keys) => self.press_in_editor(keys, window, cx),
                Step::CheckBackground(expected) => {
                    let found = self.canvas.borrow().drawing.background();
                    self.check(found == expected, format!("the background is {found:?}"));
                }
                Step::TeachName(name) => {
                    if let Some(dialog) = self.teach_dialog.clone() {
                        let input = dialog.read(cx).name.clone();
                        input.update(cx, |input, cx| input.set_value(name, window, cx));
                        dialog.update(cx, |_, cx| cx.notify());
                    }
                }
                Step::TeachToggle(i, shift) => {
                    if let Some(dialog) = self.teach_dialog.clone() {
                        dialog.update(cx, |dialog, cx| dialog.toggle(i, shift, cx));
                    }
                }
                Step::PressTeach => {
                    if let Some(dialog) = self.teach_dialog.clone() {
                        // After this update, since teaching updates the window.
                        window.defer(cx, move |window, cx| {
                            dialog.update(cx, |dialog, cx| dialog.teach(window, cx));
                        });
                    }
                }
                Step::CheckConsoleTyped(expected) => {
                    let typed = self.console_typed(cx);
                    self.check(
                        typed == expected,
                        format!("the console's line holds {typed:?}"),
                    );
                }
                Step::SaveTo(name) => {
                    let path = self.autotest.as_ref().unwrap().dir.join(name);
                    self.save_to(&path, cx);
                }
                Step::OpenFrom(name) => {
                    let path = self.autotest.as_ref().unwrap().dir.join(name);
                    self.open_path(&path, window, cx);
                }
                Step::ExportTo(name) => {
                    let path = self.autotest.as_ref().unwrap().dir.join(name);
                    self.export_to(&path, crate::files::Picture::Png, cx);
                }
                Step::CursorAt(marker) | Step::F1At(marker) => {
                    let text = self.editor.read(cx).value().to_string();
                    let offset = text.find(marker).map_or(0, |i| i + 1);
                    let at = crate::app::position(&text, offset);
                    self.editor
                        .update(cx, |state, cx| state.set_cursor_position(at, window, cx));
                    if matches!(step, Step::F1At(_)) {
                        self.show_help(window, cx);
                    }
                }
                Step::CheckHelpPage(expected) => {
                    let page = self.help_panel.read(cx).page;
                    let title = match page {
                        crate::help_panel::Page::Command(i) => pendown_logo::reference::reference()
                            .commands[i]
                            .title
                            .clone(),
                        crate::help_panel::Page::Contents => "Contents".to_string(),
                    };
                    self.check(
                        self.help_open && title == expected,
                        format!("the help panel shows {title}"),
                    );
                }
                Step::SearchHelp(query) => {
                    let search = self.help_panel.read(cx).search.clone();
                    search.update(cx, |s, cx| s.set_value(query.to_string(), window, cx));
                    self.help_panel.update(cx, |_, cx| cx.notify());
                }
                Step::CheckFirstResult(expected) => {
                    let query = self.help_panel.read(cx).query(cx);
                    let r = pendown_logo::reference::reference();
                    let first = r
                        .search(&query, false)
                        .first()
                        .map(|&i| r.commands[i].title.clone())
                        .unwrap_or_default();
                    self.check(
                        first == expected,
                        format!("the first result for {query:?} is {first}"),
                    );
                }
                Step::OpenHelp(name) => {
                    self.help_panel
                        .update(cx, |p, cx| p.open_name(name, window, cx));
                }
                Step::HelpContents => {
                    self.help_panel.update(cx, |p, cx| {
                        p.go(crate::help_panel::Page::Contents, window, cx)
                    });
                }
                Step::RunExample(name, n) => {
                    let i = pendown_logo::reference::reference()
                        .find_index(name)
                        .expect("a help page");
                    self.run_example(i, n, cx);
                }
                Step::CheckExampleHighlight(expected) => {
                    let running = self.help_panel.read(cx).running.clone();
                    let text = running
                        .as_ref()
                        .and_then(|r| r.highlight.clone())
                        .map_or(String::new(), |range| {
                            self.run_text.get(range).unwrap_or_default().to_string()
                        });
                    self.check(
                        text == expected,
                        format!("the example's highlight is on {text:?}"),
                    );
                }
                Step::BackToMyDrawing => self.back_to_my_drawing(cx),
                Step::CopyExample(name, n) => {
                    let r = pendown_logo::reference::reference();
                    let code = r.find(name).expect("a help page").examples[n].code.clone();
                    self.copy_to_editor(&code, window, cx);
                }
                Step::CheckEditorEnds(expected) => {
                    let text = self.editor.read(cx).value().to_string();
                    let tail = &text[text.len().saturating_sub(expected.len() + 20)..];
                    self.check(
                        text.ends_with(expected),
                        format!("the editor ends with {tail:?}"),
                    );
                }
                Step::CheckErrorHelp(expected) => {
                    let r = pendown_logo::reference::reference();
                    let title = self
                        .error_help
                        .map_or(String::new(), |i| r.commands[i].title.clone());
                    self.check(
                        title == expected,
                        format!("the console offers help on {title:?}"),
                    );
                    if let Some(i) = self.error_help {
                        self.open_help(i, window, cx);
                    }
                }
                Step::Lesson(l) => {
                    if !self.help_open {
                        self.toggle_help(cx);
                    }
                    self.help_panel.update(cx, |p, cx| {
                        p.tab = crate::help_panel::Tab::Tutorial;
                        p.lesson = l;
                        cx.notify();
                    });
                }
                Step::LoadLesson(lesson, program) => self.proceed(
                    crate::files::After::Load(crate::help_panel::Source::Lesson {
                        lesson,
                        program,
                    }),
                    window,
                    cx,
                ),
                Step::RunLesson(lesson, program) => {
                    self.run_source(crate::help_panel::Source::Lesson { lesson, program }, cx)
                }
                Step::Gallery => {
                    if !self.help_open {
                        self.toggle_help(cx);
                    }
                    self.help_panel.update(cx, |p, cx| {
                        p.tab = crate::help_panel::Tab::Gallery;
                        cx.notify();
                    });
                }
                Step::CheckGalleryDrawn => {
                    let (drawn, all) = self.help_panel.read(cx).gallery_pictures_drawn();
                    self.check(
                        drawn == all,
                        format!("{drawn} of {all} gallery pictures drawn"),
                    );
                }
                Step::RunGallery(title) => {
                    let source = gallery_entry(title).source;
                    self.run_source(source, cx);
                }
                Step::LoadGallery(title) => {
                    let source = gallery_entry(title).source;
                    self.proceed(crate::files::After::Load(source), window, cx);
                }
                Step::GalleryFrom(title) => {
                    let from = gallery_entry(title).from();
                    self.help_panel
                        .update(cx, |p, cx| p.open_from(from, window, cx));
                }
                Step::CheckShowing(expected) => {
                    let showing = self
                        .example
                        .as_ref()
                        .map(|e| e.source.describe())
                        .unwrap_or_default();
                    self.check(showing == expected, format!("the banner says {showing:?}"));
                }
                Step::ScrollGallery(y) => {
                    self.help_panel.update(cx, |p, cx| {
                        p.gallery_scroll.set_offset(point(px(0.), px(-y)));
                        cx.notify();
                    });
                }
                Step::CheckGalleryScroll(y) => {
                    let found = -f32::from(self.help_panel.read(cx).gallery_scroll.offset().y);
                    self.check(
                        (found - y).abs() < 1.0,
                        format!("the Gallery is scrolled {found:.0} down"),
                    );
                }
                Step::Shortcuts => {
                    if !self.help_open {
                        self.toggle_help(cx);
                    }
                    self.help_panel.update(cx, |p, cx| {
                        p.tab = crate::help_panel::Tab::Shortcuts;
                        cx.notify();
                    });
                }
                Step::CheckShortcutsBound => {
                    for row in crate::shortcuts::groups().iter().flat_map(|g| &g.rows) {
                        let found = crate::shortcuts::labels(row, window, cx);
                        let expected = crate::shortcuts::expected(row);
                        self.check(
                            found.len() == expected,
                            format!("“{}” shows {found:?}, {expected} expected", row.what),
                        );
                    }
                }
                Step::CheckShortcut(what, expected) => {
                    let found = crate::shortcuts::groups()
                        .iter()
                        .flat_map(|g| &g.rows)
                        .find(|r| r.what == what)
                        .map(|row| crate::shortcuts::labels(row, window, cx))
                        .unwrap_or_default();
                    self.check(found == expected, format!("“{what}” shows {found:?}"));
                }
                Step::CheckTab(expected) => {
                    let tab = format!("{:?}", self.help_panel.read(cx).tab);
                    self.check(tab == expected, format!("the panel shows {tab}"));
                }
                Step::CloseHelp => {
                    if self.help_open {
                        self.toggle_help(cx);
                    }
                }
                Step::Type(text) => {
                    // Like typing: an edit the user could undo, which sends
                    // a change event. `set_value` sends none.
                    self.editor.update(cx, |state, cx| {
                        state.replace_all(text.to_string(), window, cx)
                    });
                }
                Step::CheckTitle(expected) => {
                    self.update_title(window, cx);
                    let title = self.title.clone();
                    self.check(title == expected, format!("the title is {title:?}"));
                }
                Step::CheckDiagnostics(expected) => {
                    let found: Vec<String> = self.editor.update(cx, |state, _| {
                        state.diagnostics_mut().map_or(Vec::new(), |set| {
                            set.iter()
                                .map(|d| {
                                    // The message: the rest is the syntax
                                    // and example.
                                    let message = d.diagnostic.message.lines().next();
                                    message.unwrap_or_default().to_string()
                                })
                                .collect()
                        })
                    });
                    self.check(found == expected, format!("the editor shows {found:?}"));
                }
                Step::CheckEditor(expected) => {
                    let text = self.editor.read(cx).value().to_string();
                    self.check(text == expected, format!("the editor holds {text:?}"));
                }
                Step::CheckExported(name) => {
                    let path = self.autotest.as_ref().unwrap().dir.join(name);
                    let size = {
                        let canvas = self.canvas.borrow();
                        (canvas.drawing.width(), canvas.drawing.height())
                    };
                    let image = image::open(&path).map(|i| i.to_rgba8());
                    let found = image.as_ref().map(|i| i.dimensions());
                    let drawn = image
                        .as_ref()
                        .map_or(0, |i| i.pixels().filter(|p| p.0 != [0, 0, 0, 255]).count());
                    let ok = found.as_ref().is_ok_and(|s| *s == size) && drawn > 0;
                    self.check(ok, format!("exported a {found:?} PNG with {drawn} drawn pixels, the canvas is {size:?}"));
                }
                Step::CheckExportedSvg(name) => {
                    let path = self.autotest.as_ref().unwrap().dir.join(name);
                    let svg = std::fs::read_to_string(&path).unwrap_or_default();
                    // The background, then a line for each mark.
                    let marks = svg
                        .lines()
                        .filter(|l| {
                            l.starts_with('<')
                                && !l.starts_with("</")
                                && !l.starts_with("<svg")
                                && !l.starts_with("<g")
                        })
                        .count()
                        - 1;
                    let drawn = self.canvas.borrow().drawing.segments();
                    self.check(
                        svg.starts_with("<svg ") && marks == drawn,
                        format!("exported an SVG with {marks} marks, and {drawn} were drawn"),
                    );
                }
                Step::CheckDrawn(expected) => {
                    let drawn = self.canvas.borrow().drawing.segments();
                    self.check(drawn == expected, format!("{drawn} lines and shapes drawn"));
                }
                Step::CheckTurtle(x, y, heading) => {
                    let (fx, fy, found) = self
                        .animator
                        .as_ref()
                        .map_or((0.0, 0.0, 0.0), |a| (a.pos.x, a.pos.y, a.heading));
                    let ok = (fx - x).abs() < 0.01
                        && (fy - y).abs() < 0.01
                        && (found - heading).abs() < 0.01;
                    self.check(
                        ok,
                        format!("the turtle is at {fx:.1}, {fy:.1}, heading {found:.1}"),
                    );
                }
                Step::CheckConsole(expected) => {
                    let found = self.console.iter().any(|l| l.as_ref() == expected);
                    self.check(found, format!("the console shows {expected:?}"));
                }
                Step::MeasureRenders => {
                    // A focused editor or console redraws to blink its caret,
                    // which isn't what this counts: an earlier section may
                    // have left one focused.
                    window.blur(cx);
                    pendown_logo::metrics::set_enabled(true);
                    let start = (
                        self.snapshot(),
                        self.autotest.as_ref().unwrap().highlights,
                        self.console.len(),
                    );
                    self.autotest.as_mut().unwrap().measuring = Some(start);
                }
                Step::CheckRenders(per_second) => {
                    let auto = self.autotest.as_mut().unwrap();
                    let (start, highlights, lines) =
                        auto.measuring.take().expect("MeasureRenders came first");
                    let highlights = auto.highlights - highlights;
                    let printed = self.console.len().saturating_sub(lines) as u64;
                    let i = self.snapshot().since(&start);
                    let rate = i.renders as f64 / i.wall.as_secs_f64();
                    // A frame or two more, for the renders that start and
                    // end the measurement.
                    self.check(
                        rate <= per_second && i.renders <= i.frames + 2,
                        format!(
                            "{} renders at {rate:.0}/s, for {} animation frames",
                            i.renders, i.frames
                        ),
                    );
                    self.check(
                        i.editor_renders <= highlights + 2 && i.console_renders <= printed + 2,
                        format!(
                            "the editor rendered {} times for {highlights} highlight changes, the console {} times for {printed} lines",
                            i.editor_renders, i.console_renders
                        ),
                    );
                }
                Step::CheckPanels(expected) => {
                    let shown = self.panels(cx);
                    self.check(shown == expected, format!("the panels show {shown:?}"));
                }
                Step::CheckPanelsDrawn => {
                    let watch = self.watched.read(cx).watch.clone();
                    let same =
                        |drawn: &Option<std::sync::Arc<pendown_logo::Watch>>| match (drawn, &watch)
                        {
                            (Some(a), Some(b)) => std::sync::Arc::ptr_eq(a, b),
                            (None, None) => true,
                            _ => false,
                        };
                    let counters = same(&self.counters_pane.read(cx).drawn);
                    let variables = same(&self.variables_pane.read(cx).drawn);
                    self.check(
                        counters && variables,
                        format!(
                            "the counters drew the latest: {counters}, the variables: {variables}"
                        ),
                    );
                }
                Step::CheckColours(expected) => {
                    let watch = self.watched.read(cx).watch.clone();
                    let shown = watch.map_or(String::new(), |w| {
                        w.colours
                            .iter()
                            .map(|c| format!("{}={}", c.name, c.value))
                            .collect::<Vec<_>>()
                            .join(" ")
                    });
                    self.check(shown == expected, format!("the colours are {shown:?}"));
                }
                Step::CheckPanelsStart(expected) => {
                    let shown = self.panels(cx);
                    self.check(
                        shown.starts_with(expected),
                        format!("the panels show {shown:?}"),
                    );
                }
                Step::ToggleMetrics => self.toggle_metrics(window, cx),
                Step::CheckOverlay(shown) => {
                    let lines = self.overlay.as_ref().map(|o| o.lines.clone());
                    let ok = match &lines {
                        Some(lines) => {
                            shown
                                && lines
                                    .first()
                                    .is_some_and(|l| l.contains(" renders/s, UI CPU "))
                        }
                        None => !shown,
                    };
                    self.check(ok, format!("the overlay shows {lines:?}"));
                }
                Step::Quit => {
                    let auto = self.autotest.as_ref().unwrap();
                    let failures = auto.failures;
                    let mut log = auto.log.clone();
                    let _ = writeln!(log, "\n{failures} checks failed");
                    let path = auto.dir.join("autotest.log");
                    std::fs::write(&path, log).ok();
                    println!(
                        "\n{failures} checks failed; the log is in {}",
                        path.display()
                    );
                    std::process::exit(if failures == 0 { 0 } else { 1 });
                }
            }
        }
    }

    /// What the counters and variables panels show, in short: each counter,
    /// with `*` on the one `REPCOUNT` outputs, the globals, then each scope.
    fn panels(&self, cx: &gpui_kit::App) -> String {
        let Some(watch) = self.watched.read(cx).watch.clone() else {
            return String::new();
        };
        let mut parts: Vec<String> = watch
            .counters
            .iter()
            .map(|c| {
                let of = c.of.map_or(String::new(), |of| format!(" of {of}"));
                format!(
                    "{} {}{of}{}",
                    c.name,
                    c.count,
                    if c.repcount { "*" } else { "" }
                )
            })
            .collect();
        for scope in &watch.scopes {
            let vars: Vec<String> = scope
                .variables
                .iter()
                .map(|v| format!("{}={}", v.name, v.value.as_deref().unwrap_or("-")))
                .collect();
            if !scope.name.is_empty() {
                parts.push(format!("{}: {}", scope.name, vars.join(" ")));
            } else if !vars.is_empty() {
                parts.push(vars.join(" "));
            }
        }
        parts.join(" | ")
    }

    fn check_speeds(&mut self, track: &[(Instant, f64)], at: Duration, before: f64, after: f64) {
        let Some(&(start, _)) = track.first() else {
            self.check(false, "no positions recorded".into());
            return;
        };
        // Average speeds over 0.4 s either side of the change, leaving out
        // the frames around it.
        let speed = |from: Duration, to: Duration| {
            let window: Vec<_> = track
                .iter()
                .filter(|(t, _)| (from..to).contains(&(*t - start)))
                .collect();
            match (window.first(), window.last()) {
                (Some(a), Some(b)) if b.0 > a.0 => (b.1 - a.1) / (b.0 - a.0).as_secs_f64(),
                _ => 0.0,
            }
        };
        let gap = Duration::from_millis(50);
        let measured_before = speed(at - Duration::from_millis(450), at - gap);
        let measured_after = speed(at + gap, at + Duration::from_millis(450));
        let close = |measured: f64, expected: f64| (measured - expected).abs() <= expected * 0.1;
        self.check(
            close(measured_before, before) && close(measured_after, after),
            format!(
                "the turtle moved at {measured_before:.0} steps/s, then {measured_after:.0} steps/s"
            ),
        );
    }

    fn screenshot(&mut self, name: &str) {
        let auto = self.autotest.as_mut().unwrap();
        auto.screenshots += 1;
        let path = auto.dir.join(format!("{}-{name}.png", auto.screenshots));
        let title = self.title.replace('\'', "");
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "xwd -silent -name '{title}' | convert xwd:- '{}'",
                path.display()
            ))
            .status();
        let ok = status.is_ok_and(|s| s.success());
        self.say(format!(
            "  screenshot {}{}",
            path.display(),
            if ok {
                ""
            } else {
                " (failed: needs xwd and ImageMagick)"
            }
        ));
    }
}

/// The gallery's entry with this title.
fn gallery_entry(title: &str) -> &'static pendown::gallery::Entry {
    pendown::gallery::gallery()
        .iter()
        .flat_map(|s| &s.entries)
        .find(|e| e.title == title)
        .expect("a gallery entry")
}
