# 32. What Pendown remembers is in config.kdl

Status: Accepted
Date: 2026-11-04

## Context

Pendown started the same way every time, so you had to put back the window, the panes, the speed and your place in the tutorial. The Help tab also had a checkbox, Show obsolete commands, that few people need and that took up room on every page.

## Decision

- **One file, `config.kdl`,** in Pendown's settings folder. It's KDL because the help and the gallery are, and because people can read and edit it.
- **It has what's remembered and what's only set there.** The folder last opened from or saved to, the window's size and state, the editor, console and help pane sizes, the speed, and the side panel's tab, page, search, pin, lesson and scroll are remembered. `show-obsolete` is only set in the file, and replaces the checkbox.
- **Saved when the window closes,** not as things change. A crash loses that session's layout, which is cheap to put back, and nothing writes to disk during a drag or a run. The exception is the last folder, saved as soon as a file is opened or saved: that's rare, and it's already writing a file.
- **Saving only changes the remembered nodes.** It reads the file again first, so comments, `show-obsolete` and edits made while Pendown ran are all kept. A file that isn't valid KDL is left alone, with a warning, since writing over it would lose what someone was editing.
- **A wrong value is ignored or limited,** and panes are cut down to fit the window, so a hand edit or a smaller screen can't leave a pane out of reach.
- **The window's position isn't remembered,** only its size. A position can be off-screen when a monitor has gone, and it opens centred instead.
- **The console gets a divider.** It was a fixed height, so there was nothing to remember.
- **The canvas takes up changes in the window's size.** The editor, help panel and console stay as they were left. Scaled with the window, as they were, they'd grow each time a maximized window opened, as it opens at its restore size and then grows.
- **`--no-config` and `--config FILE`,** for testing and for keeping settings apart. The scripted session uses no file unless given one, as it clicks by position and expects the default layout, and it mustn't change the user's settings.

## Consequences

- The last folder used to be a file of its own, `last-folder`, written at once and shared with the scripted session, which kept overwriting it. It's now kept in full, in this file. A full path in the old file moves in, and the file goes after the first save.
- The program in the editor isn't remembered, so a restored help page that was following the cursor shows until the cursor moves to another word.
- The window's restore size is tracked as it changes, because X11 reports a maximized window's bounds as its current size, and may do so before saying it's maximized. X11 ignores a window opened maximized or fullscreen, so on every platform it opens at its restore size, and then becomes that.
- Only the tab that was showing keeps its scroll. A page is too short for its scroll until its pictures are drawn, so the panel goes on scrolling it until they are.
