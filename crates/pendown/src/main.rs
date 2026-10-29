//! Pendown: a Logo playground where you can watch the turtle think.

// On Windows, a release build opens no console window beside the app. Debug
// builds keep it, for what Pendown prints while it's being worked on.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod autotest;
mod canvas;
mod colours_page;
mod driving;
mod files;
mod help_panel;
mod highlighter;
mod icons;
mod panes;
mod teach_dialog;
mod watching;

use std::borrow::Cow;

use gpui_kit::*;

/// Noto Sans, under the SIL Open Font License: see `fonts/OFL.txt`.
const FONTS: [&[u8]; 4] = [
    include_bytes!("../fonts/NotoSans-Regular.ttf"),
    include_bytes!("../fonts/NotoSans-Italic.ttf"),
    include_bytes!("../fonts/NotoSans-Bold.ttf"),
    include_bytes!("../fonts/NotoSans-BoldItalic.ttf"),
];

fn main() {
    gpui_kit::application()
        .with_assets(icons::AppAssets)
        .run(|cx| {
            gpui_kit::init(cx);
            files::bind_keys(cx);
            // The UI font is built in, so it looks the same everywhere. The
            // system's is often a variable font, such as Ubuntu on Linux, and gpui
            // draws only its regular weight, so bold text isn't.
            cx.text_system()
                .add_fonts(FONTS.iter().map(|&f| Cow::Borrowed(f)).collect())
                .expect("the built-in fonts load");
            cx.global_mut::<gpui_kit::component::Theme>().font_family = "Noto Sans".into();
            // Closing the window ends the app.
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            let options = WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some("Pendown".into()),
                    ..Default::default()
                }),
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1400.), px(900.)),
                    cx,
                ))),
                ..Default::default()
            };
            gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| app::App::new(window, cx))
            })
            .expect("can open the window");
            cx.activate(true);
        });
}
