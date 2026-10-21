# Templates and the higher-order commands

Status: ready-for-agent
Type: task
Blocked by: 04

## What

- **Templates** in UCBLogo's four forms: procedure names, explicit slots (`?`, `?1`, `?REST`, `#`), lambdas `[[x] ...]` and procedure text `[[x] [...] [...]]`.
- **`APPLY`, `INVOKE`, `FOREACH`, `MAP`, `MAP.SE`, `FILTER`, `FIND` and `REDUCE`,** with several data inputs where the manual allows them, and words as data.
- **Errors:** the wrong number of inputs for a template, a `FILTER` template that doesn't output `TRUE` or `FALSE`, data inputs of different lengths.

## Done when

Each command has tests of its behaviour and its errors in `pendown-logo`, or the GUI and CLI tests cover it.

## Comments
