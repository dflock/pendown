# The Gallery tab

Status: resolved
Type: task
Blocked by: 01, 02

## What

A third tab in the side panel, after Help and Tutorial (`Tab::Gallery`), as the mockup shows (linked in this issue's comments).

- **Sections** in `gallery.kdl`'s order, each with a heading, then its cards two to a row. A narrow panel goes to one a row.
- **A card:**
  - the thumbnail, square, the picture drawn with `HelpPanel::picture` and scaled to fit, with a blank square while it draws;
  - the title, and under it where it's from ("Help: REPEAT", "Lesson 6", "Gallery"), a link to that help page or lesson where there is one;
  - **Load**, with "Load into editor" as its tooltip, and **Run**, as issue 02 does them. While the user's program runs, they're off, as the help panel's buttons are.
  - While its example is on the canvas, the card's picture is outlined.
- **Drawing the thumbnails:** in the background, as the help panel's pictures are, fitted to the drawing, starting when the tab first opens, in order down the page. The help panel draws its pictures 84 px square (`PICTURE_SIZE`); a card's is about twice that, so draw them at the card's size, keyed by size in the cache, or lines will be blurry or too thin.
- **Scrolling:** the tab keeps its scroll position when you switch tabs and back.
- **Follow the cursor** (Help's behaviour) doesn't switch to the Gallery.

## Done when

- The scripted session opens the Gallery, screenshots it, runs a card's example, goes back to the drawing, and loads one into the editor.
- The screenshot shows the thumbnails drawn, not blank.
- Opening the tab doesn't slow the window: the session's render counts and frame intervals pass.

## Comments

2026-10-31: Mockup: https://claude.ai/artifact/24LiwBQn1efdn9BuSf5jbk. It uses the real pictures from `crates/pendown/tests/help-pictures/`. The user chose the grid, two a row, over a list of one a row.

2026-10-31: Done. Cards are 160 px pictures in a wrapping row: two a row at the panel's usual width, one when it's narrow. The pictures are the help panel's SVGs, rendered at the card's size by `svg_image`, so they're sharp without drawing them again at a different size. Load and Run stay on while the user's program runs, as the lessons' buttons do: both stop it first. (The help panel turns buttons off only on the Colours page.) The scripted session checks all 38 pictures are drawn, Run and Back to my drawing, Load, the scroll kept across tabs, and the from-links.
