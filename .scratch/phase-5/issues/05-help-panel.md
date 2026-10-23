# The side panel and Help

Status: done
Type: task
Blocked by: 03

## What

A resizable panel on the right with Help and Tutorial tabs, shown by the toolbar's Help button and F1. Help has Back and Forward, Contents, search, command pages with pictures and See also, obsolete commands on request, and follows the editor's cursor, with Pin.

## Done when

The panel works as in the mockups, and screenshots show it.

## Comments

### 2026-10-23: done

`help_panel.rs` is the panel, a cached view on the right of the window, in its own resizable layout (`main-help`, so each layout keeps its sizes). The toolbar's Help button shows it, and F1 opens it on the word under the cursor. Help has Back and Forward, Contents, search, and command pages: the section as links, the names, the usage without the manual's notes, our help, the examples with their pictures, and See also. Obsolete commands, on request, have a "Not in Pendown" note. Links in the text are gpui's `InteractiveText`, in the theme's blue: its link colour is black.

Following the cursor: the window observes the editor, which notifies when the cursor moves, and `pendown::help::word_at` finds the page for the token there, using the highlighter's tokens: `:size` shows Variables, a number Numbers, and a procedure of the program nothing. Clicking to a page shows the strip as just following; Back, Forward and Pin pin the page.

Pictures are drawn with `pendown_cli::render` on a background task the first time a page shows them, and kept. An example that draws nothing shows what it printed instead.

Left out: ↑ and ↓ through the search results. Enter opens the first.
