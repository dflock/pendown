# Help on errors

Status: done
Type: task
Blocked by: 02, 05

## What

After an error with a `command`, the console's title bar has a Help on X button that opens its page, obsolete ones included.

## Done when

The button appears after an error and opens the page.

## Comments

### 2026-10-23: done

`show_errors` keeps the help page of the first error whose command has one (`App::error_help`), and the console's title bar shows **Help on FORWARD** for it, by the page's title, so `FD` shows FORWARD. Pressing it opens the panel on that page, pinned. A new run clears it. It works for obsolete commands, whose pages show the reason and the usage without the manual's text. `(EDIT)` in a usage is EDIT, not a name `EDIT)`.
