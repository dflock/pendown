# Phase 1 check: examples, HiDPI and Wayland

Status: ready-for-human
Type: task
Blocked by: 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14, 15

## What

The end-of-phase check, before phase 2:

- **Programs:** write a handful of Logo programs that use every phase-1 command. Run them in the GUI at several speeds and with the CLI, and add them as picture tests.
- **Untested platforms** (spikes 01, 02 and 04):
  - a real HiDPI display, where the pixel buffer is twice the size;
  - Wayland;
  - a GPU other than an Intel HD 530.
  
  Check the speed, the memory and how the drawing looks.
- **The kdl:** every phase-1 command in `pendown-logo.kdl` is implemented, and a test checks that the interpreter's command table and the kdl agree.
- **Docs:** update `README.md` and `implementation-plan.md` with anything that changed.

## Done when

The programs run as expected everywhere tested, and anything found is fixed or filed as an issue.

## Comments

### 2026-10-21: the parts an agent can do

- **Programs:** `examples/phase-1/` has four programs: `garden`, `edges`, `clean` and `numbers`.
  - Between them they use every phase-1 command. `tests/kdl.rs` in `pendown-logo` checks this, using `syntax::highlight` to find the commands they call.
  - `pendown-cli`'s `phase_1_examples` test draws each one at 400 × 400 with seed 1, and compares the result with `tests/pictures/example-NAME.png`, and its printed output with `example-NAME.txt`.
- **The kdl:**
  - `tests/kdl.rs` checks that the interpreter's command table and the kdl's phase-1 commands agree exactly, names and aliases both.
  - It found one difference: `WORD` was implemented, by spike 03, but marked phase 2. **Decided:** it's now phase 1, so phase 1 is 90 commands and phase 2 is 87. The plan says so.
- **Docs:**
  - `README.md` now mentions the examples and the scripted session.
  - The plan's CLI section and phase counts are up to date.
  - ADRs 0001–0019 were updated as each issue changed them.

### Still for a human

- Try the app: `cargo run --release -p pendown`. Run, Pause, Step, Step instruction, Stop and the speed slider, on the examples and your own programs.
- **The platform dialogs:** Open, Save As and Export PNG, and the "Save the changes?" prompt on New, Open and closing the window. The scripted session can't drive them.
- **Untested platforms:** a real HiDPI display (the pixel buffer twice the size), Wayland, and a GPU other than the Intel HD 530. Check the speed, the memory and how the drawing looks.
