# Phase 5: help, examples and tutorial

A side panel with tabs for Help and Tutorial. Help is a reference for every implemented command, generated from `pendown-logo.kdl`, with our own descriptions and runnable examples. The tutorial is six lessons with Kame. Errors for obsolete commands say why, and errors link to help. A tree-sitter grammar for Logo comes too. See "Phase 5" in `implementation-plan.md`, and the mockups linked from it.

There are no new Logo commands.

## What "done" looks like

- **The side panel:**
  - on the right of the window, beside the canvas, resizable, with tabs for Help and Tutorial;
  - the toolbar's **Help** button shows and hides it, and **F1** opens it on help for the word under the editor's cursor.
- **Help:**
  - Back and Forward, Contents, and a search box;
  - pages for every implemented command, with its section, names, usage, our description, examples with pictures, and "See also";
  - the contents by chapter and section, with counts, and our extensions tagged;
  - search by name and alias;
  - obsolete commands on request, greyed, with their reasons;
  - following the cursor, with Pin;
  - Syntax pages for `TO … END`, `:name`, `"word`, lists, arrays, parentheses, comments and the operators.
- **Descriptions:** a `help` node for every phase 1 and 2 command and every syntax entry, written for our dialect, in the markup below.
- **Examples:** at least one `example` for every phase 1 and 2 command and syntax entry.
  - **Run:** an example runs on the main canvas, highlighted in the panel. The user's own drawing, turtle and panels come back with **Back to my drawing**, or when they Run.
  - **Copy to editor:** adds it to the end of the program, and Undo takes it out again.
- **Examples are tests:** every example, and every code block in the tutorial, runs without errors, and its picture matches a saved PNG.
- **Tutorial:** six lessons as markdown files, with Load into editor and Run on their code.
- **Obsolete commands:** running one says why it isn't in Pendown. The reasons are reworded to read as errors.
- **Help from errors:** after an error about a command, the console's title bar has a **Help on X** button that opens its page.
- **A tree-sitter grammar** and a Zed extension, in a folder of their own, ready to publish as their own repository.

## Decisions

- **The reference lives in `pendown-logo`,** as `pendown_logo::reference`. It embeds `pendown-logo.kdl` and parses it with the `kdl` crate, once, when first needed. The interpreter uses it for obsolete commands' errors, and the GUI uses it for the panel. So the reasons have one source, and `kdl` becomes a normal dependency of `pendown-logo`, not just a test one.
- **The kdl's new nodes,** on a command:
  - `help """..."""`: our description;
  - `example """...""" title="..."`, any number: a runnable program;
  - `see "name" "name"`: more "See also" than its section gives, when it helps.
- **Syntax entries** are in a `syntax` node at the top level of the kdl, beside `logo`, each with `eg`, `help` and `example`, and `names` for what the cursor can be on, such as `:`, `[` or `+`.
- **Markup:** paragraphs, `code`, `**bold**`, `*italics*`, lists starting `- `, fenced code blocks, and `[NAME]`, which links to a command or syntax page. `pendown::markup` parses it, and a test checks every link.
- **Pictures are drawn when shown, not stored in the app.** The panel runs each example headless on a background thread, with `pendown-cli`'s renderer, and caches the picture. The examples test compares the same pictures with PNGs saved in `crates/pendown/tests/help-pictures/`.
  - The pictures are fitted to the drawing, so a small one isn't lost in the whole turtle area. That's a new `fit` option on the renderer.
  - `RANDOM` uses seed 1 in pictures, as in the tests.
- **An example runs on a session of its own** (`run::start`), so the user's interpreter, with its procedures and variables, is untouched. The window keeps the user's run, animator and drawing while it runs, and swaps them back.
- **The example's picture stays when it ends,** until **Back to my drawing** or Run. The plan said the user's drawing comes back as soon as it ends, but then you'd never see what the example drew.
- **The highlight goes to the panel,** on the example's code, while an example runs: its spans point into the example's text.
- **Errors carry the command they're about,** as `Error::command`, set by the primitives' error helpers, and by the arity and "I don't know how to" errors.
- **The console is a text area,** so its lines can't hold links. A "Help on X" button in its title bar serves the last error instead.
- **The tutorial** is `tutorial/NN-name.md` at the repo root, embedded with `include_str!`. A lesson's title is its first heading.
- **Following the cursor:** the window watches the editor, which notifies on every cursor move, and finds the word under the cursor with `pendown_logo::syntax`'s tokens.
- **The grammar** is in `tree-sitter-logo/` in this repo for now, with its Zed extension in `tree-sitter-logo/zed/`. Publishing it as its own repository is for the user to do, or to ask for.

## How the work is split

| Issue | What | Blocked by |
|---|---|---|
| 01 | `pendown_logo::reference`: the kdl's model, lookup and search | |
| 02 | Obsolete commands' errors, the reasons reworded, and `Error::command` | 01 |
| 03 | Markup, and `help` and `example` for every command and syntax entry | 01 |
| 04 | Examples are tests: fitted pictures, saved PNGs | 03 |
| 05 | The side panel: Help's pages, contents, search, following the cursor | 03 |
| 06 | Running examples, and Copy to editor | 05 |
| 07 | Help on errors | 02, 05 |
| 08 | The tutorial | 05, 06 |
| 09 | Checks in the scripted session | 06, 07, 08 |
| 10 | The tree-sitter grammar and Zed extension | |
| 11 | Phase 5 check | all others |

## Testing

- **Reference tests:** every implemented command has a page, with help and an example; lookup by every alias; search; obsolete commands and their reasons; syntax entries.
- **Markup tests,** and a test that every `[NAME]` link resolves.
- **Interpreter tests** of obsolete commands' errors, and of `Error::command`.
- **The examples test:** every example and tutorial code block runs cleanly, and its fitted picture matches the saved PNG.
- **The scripted session:** F1 on a word, search, an example run and Back to my drawing, Help on an error, a tutorial lesson loaded and run, with screenshots.
- **The grammar's own tests,** in tree-sitter's corpus format.

## Out of scope

- Help for phase 6 and 7 commands, which come with those phases.
- Translations.
- Publishing the grammar's repository, and the Zed extension, without the user's go-ahead.
