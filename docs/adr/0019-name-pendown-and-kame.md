# 19. The project is called Pendown, and the tutorial's turtle is Kame

Status: Accepted
Date: 2026-10-18. Updated 2026-10-21 with `pendown-render`.

## Context

The project needed a name. Names used by other Logo projects or products were ruled out: Turtle (a Rust turtle-graphics crate), Rusty-Turtle, Turtle Logo, Logo Worlds, Papert, Terrapin, Turtle Art and Mindstorms. `pendown` was free on crates.io.

## Decision

- **The project and app are called Pendown.** `PENDOWN` is the Logo command that makes the turtle start drawing, which suits a tool for creating new programs.
- **Crates**, in a Cargo workspace:
  - `pendown`: the GUI app, the main thing users run. Its window title is "Pendown".
  - `pendown-cli`: the CLI, which renders programs to files.
  - `pendown-logo`: the interpreter library, with no GUI code (ADR 0009). Earlier documents call this "the logo crate".
  - `pendown-render`: the display list and the pixel buffer, drawn with vello_cpu (ADRs 0003 and 0010), and PNG encoding. It has no gpui code, so the GUI and the CLI share it, as ADR 0003 requires. It can't live in `pendown-logo`, which has no drawing code, or in `pendown`, which the CLI shouldn't depend on.
- **Layout:** the workspace's `Cargo.toml` is at the repo root, with the crates in `crates/`. The spikes in `spikes/` are kept as a record, outside the workspace.
- **The turtle character in the tutorial and help is called Kame,** Japanese for turtle. Lessons can talk about what Kame does: "Tell Kame to go forward 100 steps".

## Consequences

- The binaries are named after their crates: `pendown` for the GUI and `pendown-cli` for the CLI.
- In Rust code the library is imported as `pendown_logo`.
- `kame` is taken on crates.io (an SSH toolkit), but Kame is only a character, not a crate.
- The repository directory is still called `logo-rs`; renaming it is optional.
