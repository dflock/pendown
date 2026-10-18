# 19. The project is called Pendown, and the tutorial's turtle is Kame

Status: Accepted
Date: 2026-10-18

## Context

The project needed a name. Names used by other Logo projects or products were ruled out: Turtle (a Rust turtle-graphics crate), Rusty-Turtle, Turtle Logo, Logo Worlds, Papert, Terrapin, Turtle Art and Mindstorms. `pendown` was free on crates.io.

## Decision

- **The project and app are called Pendown.** `PENDOWN` is the Logo command that makes the turtle start drawing, which suits a tool for creating new programs.
- **Crates**, in a Cargo workspace:
  - `pendown`: the GUI app, the main thing users run. Its window title is "Pendown".
  - `pendown-cli`: the CLI, which renders programs to files.
  - `pendown-logo`: the interpreter library, with no GUI code (ADR 0009). Earlier documents call this "the logo crate".
- **The turtle character in the tutorial and help is called Kame,** Japanese for turtle. Lessons can talk about what Kame does: "Tell Kame to go forward 100 steps".

## Consequences

- The binaries are named after their crates: `pendown` for the GUI and `pendown-cli` for the CLI.
- In Rust code the library is imported as `pendown_logo`.
- `kame` is taken on crates.io (an SSH toolkit), but Kame is only a character, not a crate.
- The repository directory is still called `logo-rs`; renaming it is optional.
