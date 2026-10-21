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
