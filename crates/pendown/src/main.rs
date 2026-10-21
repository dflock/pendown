//! Pendown: a Logo playground where you can watch the turtle think.

mod app;
mod autotest;
mod canvas;
mod highlighter;

use gpui_kit::*;

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);
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
