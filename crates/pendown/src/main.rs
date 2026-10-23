//! Pendown: a Logo playground where you can watch the turtle think.

mod app;
mod autotest;
mod canvas;
mod files;
mod help_panel;
mod highlighter;
mod icons;
mod panes;
mod watching;

use gpui_kit::*;

fn main() {
    gpui_kit::application().with_assets(icons::AppAssets).run(|cx| {
        gpui_kit::init(cx);
        files::bind_keys(cx);
        // On Linux the UI font is often a variable one, such as Ubuntu, and
        // gpui draws only its regular weight, so bold text isn't. Noto Sans
        // has a file for each weight, and most distributions install it.
        #[cfg(target_os = "linux")]
        if cx.text_system().all_font_names().iter().any(|f| f == "Noto Sans") {
            cx.global_mut::<gpui_kit::component::Theme>().font_family = "Noto Sans".into();
        }
        // Closing the window ends the app.
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions { title: Some("Pendown".into()), ..Default::default() }),
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, size(px(1400.), px(900.)), cx))),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, |window, cx| cx.new(|cx| app::App::new(window, cx)))
            .expect("can open the window");
        cx.activate(true);
    });
}
