//! The help panel's Colours page: the forms, a colour picker with a wheel,
//! the palette and the names as swatches, and the current colour, which
//! Copy to editor puts at the editor's cursor.

use std::{
    cell::Cell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

use gpui_kit::component::{
    ActiveTheme, Disableable as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};
use gpui_kit::{prelude::FluentBuilder as _, *};
use pendown::colours::{
    Form, Hsva, forms, names_matching, parse_channel, parse_hex, text, wheel_colour, wheel_point,
};
use pendown_logo::PALETTE;

use crate::app::App as PendownWindow;

/// The wheel's size, in logical pixels.
const WHEEL: f32 = 168.;

/// What set the colour, which decides its form.
#[derive(Clone, Copy)]
enum Source {
    Picker,
    Palette(u8),
    Name(&'static str),
}

/// Pictures made from pixels, by their size in device pixels and what
/// they're of, as Help's pictures are kept.
type Images = Arc<Mutex<HashMap<(i32, i32, u32), Arc<RenderImage>>>>;

/// The R, G, B, A and hex fields' labels.
const FIELDS: [&str; 5] = ["R", "G", "B", "A", "Hex"];

pub struct ColoursPage {
    app: WeakEntity<PendownWindow>,
    pub(crate) colour: Hsva,
    pub(crate) form: Form,
    /// The form the picker gives: RGB, RGBA or Hex, whichever was chosen last.
    picker_form: Form,
    pub(crate) fields: Vec<Entity<InputState>>,
    /// The field that doesn't make sense, and why.
    pub(crate) problem: Option<(usize, String)>,
    /// The fields are being set to the colour, not typed in.
    syncing: bool,
    pub(crate) filter: Entity<InputState>,
    images: Images,
    wheel_bounds: Rc<Cell<Bounds<Pixels>>>,
    brightness_bounds: Rc<Cell<Bounds<Pixels>>>,
    opacity_bounds: Rc<Cell<Bounds<Pixels>>>,
    _subscriptions: Vec<Subscription>,
}

/// A colour for gpui.
fn hsla([r, g, b, a]: [u8; 4]) -> Hsla {
    gpui_kit::Rgba {
        r: r as f32 / 255.,
        g: g as f32 / 255.,
        b: b as f32 / 255.,
        a: a as f32 / 255.,
    }
    .into()
}

/// A picture made from pixels at the size it's shown, kept in `images` by
/// its size and `what`. `pixel` gives each pixel as premultiplied BGRA, from
/// its position and the picture's size, in device pixels.
fn pixels_image(
    images: Images,
    what: u32,
    width: f32,
    height: f32,
    pixel: impl Fn(u32, u32, u32, u32) -> [u8; 4] + 'static,
) -> Img {
    img(ImageSource::Custom(Arc::new(move |window, _| {
        let scale = window.scale_factor();
        let (w, h) = (
            (width * scale).round().max(1.) as u32,
            (height * scale).round().max(1.) as u32,
        );
        let key = (w as i32, h as i32, what);
        let mut images = images.lock().unwrap();
        if let Some(image) = images.get(&key) {
            return Some(Ok(image.clone()));
        }
        let mut bytes = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                bytes.extend(pixel(x, y, w, h));
            }
        }
        let buffer = image::RgbaImage::from_raw(w, h, bytes).expect("buffer matches size");
        let image = Arc::new(RenderImage::new([image::Frame::new(buffer)]));
        // Only the newest wheels are wanted: drop the rest now and then.
        if images.len() > 16 {
            images.clear();
        }
        images.insert(key, image.clone());
        Some(Ok(image))
    })))
    .w(px(width))
    .h(px(height))
}

/// A checkerboard, to show how see-through a colour is.
fn checker(images: Images, width: f32, height: f32) -> Img {
    pixels_image(images, u32::MAX, width, height, |x, y, _, _| {
        let light = ((x / 8) + (y / 8)) % 2 == 0;
        if light {
            [255, 255, 255, 255]
        } else {
            [216, 212, 212, 255]
        }
    })
}

/// A swatch of `rgba` over a checkerboard, `size` square.
fn swatch(images: Images, rgba: [u8; 4], size: f32, cx: &App) -> Div {
    div()
        .relative()
        .flex_none()
        .size(px(size))
        .rounded_md()
        .overflow_hidden()
        .border_1()
        .border_color(cx.theme().border)
        .child(checker(images, size, size).absolute())
        .child(div().absolute().size_full().bg(hsla(rgba)))
}

/// Where `position` is in `bounds`, from 0 to 1 across and down.
fn fraction(bounds: Bounds<Pixels>, position: Point<Pixels>) -> (f32, f32) {
    let x = (position.x - bounds.origin.x).as_f32() / bounds.size.width.as_f32().max(1.);
    let y = (position.y - bounds.origin.y).as_f32() / bounds.size.height.as_f32().max(1.);
    (x, y)
}

/// Keeps an element's bounds as it's laid out, for turning the mouse's
/// position into a colour.
fn track_bounds(cell: Rc<Cell<Bounds<Pixels>>>) -> impl IntoElement {
    canvas(move |bounds, _, _| cell.set(bounds), |_, _, _, _| {})
        .absolute()
        .size_full()
}

impl ColoursPage {
    pub fn new(
        app: WeakEntity<PendownWindow>,
        start: [u8; 4],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> ColoursPage {
        let mut subscriptions = Vec::new();
        let fields: Vec<_> = (0..FIELDS.len())
            .map(|i| {
                let field = cx.new(|cx| InputState::new(window, cx));
                subscriptions.push(cx.subscribe_in(
                    &field,
                    window,
                    move |this: &mut ColoursPage, _, event, window, cx| {
                        if let InputEvent::Change = event {
                            this.typed(i, window, cx);
                        }
                    },
                ));
                field
            })
            .collect();
        let filter =
            cx.new(|cx| InputState::new(window, cx).placeholder("Filter: blue, light, sea…"));
        subscriptions.push(cx.subscribe(&filter, |_, _, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                cx.notify();
            }
        }));
        let mut page = ColoursPage {
            app,
            colour: Hsva::from_rgba(start),
            form: Form::Hex,
            picker_form: Form::Hex,
            fields,
            problem: None,
            syncing: false,
            filter,
            images: Images::default(),
            wheel_bounds: Rc::default(),
            brightness_bounds: Rc::default(),
            opacity_bounds: Rc::default(),
            _subscriptions: subscriptions,
        };
        page.sync_fields(None, window, cx);
        page
    }

    pub(crate) fn rgba(&self) -> [u8; 4] {
        self.colour.rgba()
    }

    /// What Copy to editor puts in: the colour in its form.
    pub(crate) fn text(&self) -> String {
        text(self.form, self.rgba())
    }

    /// Sets the colour, in the form its source gives it.
    fn set(&mut self, colour: Hsva, source: Source, window: &mut Window, cx: &mut Context<Self>) {
        self.colour = colour;
        self.form = match source {
            Source::Picker => self.picker_form,
            Source::Palette(n) => Form::Palette(n),
            Source::Name(name) => Form::Name(name),
        };
        self.problem = None;
        self.sync_fields(None, window, cx);
        cx.notify();
    }

    pub(crate) fn pick_palette(&mut self, n: u8, window: &mut Window, cx: &mut Context<Self>) {
        let colour = Hsva::from_rgba(PALETTE[n as usize]);
        self.set(colour, Source::Palette(n), window, cx);
    }

    pub(crate) fn pick_name(
        &mut self,
        name: &'static str,
        rgba: [u8; 4],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set(Hsva::from_rgba(rgba), Source::Name(name), window, cx);
    }

    /// Chooses the form to copy in. RGB, RGBA and Hex are kept for the
    /// picker too.
    pub(crate) fn choose(&mut self, form: Form, cx: &mut Context<Self>) {
        if matches!(form, Form::Rgb | Form::Rgba | Form::Hex) {
            self.picker_form = form;
        }
        self.form = form;
        cx.notify();
    }

    /// Puts the colour in the fields, but `except` the one being typed in.
    fn sync_fields(&mut self, except: Option<usize>, window: &mut Window, cx: &mut Context<Self>) {
        let [r, g, b, a] = self.rgba();
        let hex = text(Form::Hex, [r, g, b, a])
            .trim_start_matches('"')
            .to_string();
        let values = [
            r.to_string(),
            g.to_string(),
            b.to_string(),
            a.to_string(),
            hex,
        ];
        self.syncing = true;
        for (i, value) in values.into_iter().enumerate() {
            if Some(i) != except {
                self.fields[i].update(cx, |field, cx| field.set_value(value, window, cx));
            }
        }
        self.syncing = false;
    }

    /// A field was typed in: the colour follows if it makes sense.
    pub(crate) fn typed(&mut self, i: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.syncing {
            return;
        }
        let value = self.fields[i].read(cx).value().to_string();
        let parsed = if i == 4 {
            parse_hex(&value)
        } else {
            parse_channel(&value, FIELDS[i]).map(|n| {
                let mut rgba = self.rgba();
                rgba[i] = n;
                rgba
            })
        };
        match parsed {
            Ok(rgba) if rgba == self.rgba() => self.problem = None,
            Ok(rgba) => {
                // Opacity alone doesn't change the hue, saturation or
                // brightness, so keep them: grey keeps its hue.
                let colour = if rgba[..3] == self.rgba()[..3] {
                    Hsva {
                        a: rgba[3],
                        ..self.colour
                    }
                } else {
                    Hsva::from_rgba(rgba)
                };
                self.colour = colour;
                self.form = self.picker_form;
                self.problem = None;
                self.sync_fields(Some(i), window, cx);
            }
            Err(why) => self.problem = Some((i, why)),
        }
        cx.notify();
    }

    fn wheel_at(&mut self, position: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        let (x, y) = fraction(self.wheel_bounds.get(), position);
        self.pick_on_wheel(x as f64, y as f64, window, cx);
    }

    /// Picks the colour at a point on the wheel, from 0 to 1 across and down.
    pub(crate) fn pick_on_wheel(
        &mut self,
        x: f64,
        y: f64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (h, s) = wheel_colour(x * 2. - 1., y * 2. - 1.);
        // Picking on the wheel at black would show nothing changing.
        let v = if self.colour.v < 0.05 {
            1.
        } else {
            self.colour.v
        };
        let colour = Hsva {
            h,
            s,
            v,
            ..self.colour
        };
        self.set(colour, Source::Picker, window, cx);
    }

    fn brightness_at(
        &mut self,
        position: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (x, _) = fraction(self.brightness_bounds.get(), position);
        let colour = Hsva {
            v: x.clamp(0., 1.) as f64,
            ..self.colour
        };
        self.set(colour, Source::Picker, window, cx);
    }

    fn opacity_at(&mut self, position: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        let (x, _) = fraction(self.opacity_bounds.get(), position);
        let colour = Hsva {
            a: (x.clamp(0., 1.) * 255.).round() as u8,
            ..self.colour
        };
        self.set(colour, Source::Picker, window, cx);
    }

    /// Copy to editor: the colour goes at the editor's cursor.
    pub(crate) fn copy(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.text();
        self.app
            .update(cx, |app, cx| app.insert_at_cursor(&text, window, cx))
            .ok();
    }

    /// The forms, each with a swatch.
    pub fn forms_table(&self, cx: &App) -> AnyElement {
        let theme = cx.theme();
        let tomato = [255, 99, 71, 255];
        let half = [255, 99, 71, 128];
        let rows: [(&str, [u8; 4], &str); 7] = [
            ("Palette number", [255, 0, 0, 255], "4"),
            ("Name", tomato, "\"tomato"),
            ("Red, green, blue", tomato, "[255 99 71]"),
            ("…and opacity", half, "[255 99 71 128]"),
            ("Hex", tomato, "\"#ff6347"),
            ("Short hex", [255, 102, 68, 255], "\"#f64"),
            ("Hex, with opacity", half, "\"#ff634780"),
        ];
        v_flex()
            .children(rows.into_iter().map(|(what, rgba, code)| {
                h_flex()
                    .gap_2()
                    .py_1()
                    .border_b_1()
                    .border_color(theme.border.opacity(0.5))
                    .child(swatch(self.images.clone(), rgba, 20., cx))
                    .child(
                        div()
                            .w(px(120.))
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(what),
                    )
                    .child(
                        div()
                            .font_family(theme.mono_font_family.clone())
                            .text_xs()
                            .child(code),
                    )
            }))
            .child(
                div()
                    .mt_2()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child("Numbers in a list go from 0 to 255. Opacity 0 can't be seen, and 255 is solid."),
            )
            .into_any_element()
    }

    /// The wheel, the brightness and opacity bars, and the fields.
    pub fn picker(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let rgba = self.rgba();
        let solid = [rgba[0], rgba[1], rgba[2], 255];
        let Hsva { h, s, v, a } = self.colour;
        let (mx, my) = wheel_point(h, s);
        let marker = |x: f32, y: f32| {
            div()
                .absolute()
                .left(px(x - 7.))
                .top(px(y - 7.))
                .size(px(14.))
                .rounded_full()
                .border_2()
                .border_color(white())
                .shadow_sm()
                .bg(hsla(solid))
        };
        let brightness = (v * 1000.).round() as u32;
        let wheel = div()
            .id("colour-wheel")
            .relative()
            .flex_none()
            .size(px(WHEEL))
            .cursor_crosshair()
            .child(track_bounds(self.wheel_bounds.clone()))
            .child(pixels_image(
                self.images.clone(),
                brightness,
                WHEEL,
                WHEEL,
                move |x, y, w, _| {
                    let r = w as f64 / 2.;
                    let (dx, dy) = ((x as f64 + 0.5 - r) / r, (y as f64 + 0.5 - r) / r);
                    let coverage = ((1. - dx.hypot(dy)) * r + 0.5).clamp(0., 1.);
                    if coverage == 0. {
                        return [0; 4];
                    }
                    let (h, s) = wheel_colour(dx, dy);
                    let [r, g, b, _] = Hsva { h, s, v, a: 255 }.rgba();
                    let pre = |c: u8| (c as f64 * coverage).round() as u8;
                    [pre(b), pre(g), pre(r), (255. * coverage).round() as u8]
                },
            ))
            .child(marker(
                WHEEL / 2. * (1. + mx as f32),
                WHEEL / 2. * (1. + my as f32),
            ))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, e: &MouseDownEvent, window, cx| {
                    this.wheel_at(e.position, window, cx)
                }),
            )
            .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, window, cx| {
                if e.pressed_button == Some(MouseButton::Left) {
                    this.wheel_at(e.position, window, cx)
                }
            }));

        let full = Hsva {
            v: 1.,
            a: 255,
            ..self.colour
        }
        .rgba();
        let handle = |at: f32| {
            div()
                .absolute()
                .left(relative(at))
                .ml(px(-7.))
                .top(px(-1.))
                .size(px(14.))
                .rounded_full()
                .border_2()
                .border_color(white())
                .shadow_sm()
                .bg(hsla(rgba))
        };
        let brightness_bar = h_flex()
            .gap_2()
            .text_xs()
            .text_color(theme.muted_foreground)
            .child(div().w(px(64.)).child("Brightness"))
            .child(
                div()
                    .id("brightness")
                    .relative()
                    .flex_1()
                    .h(px(12.))
                    .rounded_full()
                    .cursor_pointer()
                    .bg(linear_gradient(
                        90.,
                        linear_color_stop(hsla([0, 0, 0, 255]), 0.),
                        linear_color_stop(hsla(full), 1.),
                    ))
                    .child(track_bounds(self.brightness_bounds.clone()))
                    .child(handle(v as f32))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, e: &MouseDownEvent, window, cx| {
                            this.brightness_at(e.position, window, cx)
                        }),
                    )
                    .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, window, cx| {
                        if e.pressed_button == Some(MouseButton::Left) {
                            this.brightness_at(e.position, window, cx)
                        }
                    })),
            );
        let clear = [solid[0], solid[1], solid[2], 0];
        let opacity_bar = h_flex()
            .gap_2()
            .text_xs()
            .text_color(theme.muted_foreground)
            .child(div().w(px(64.)).child("Opacity"))
            .child(
                div()
                    .id("opacity")
                    .relative()
                    .flex_1()
                    .h(px(12.))
                    .cursor_pointer()
                    .child(track_bounds(self.opacity_bounds.clone()))
                    .child(
                        div()
                            .absolute()
                            .size_full()
                            .rounded_full()
                            .overflow_hidden()
                            .child(checker(self.images.clone(), 240., 12.)),
                    )
                    .child(
                        div()
                            .absolute()
                            .size_full()
                            .rounded_full()
                            .bg(linear_gradient(
                                90.,
                                linear_color_stop(hsla(clear), 0.),
                                linear_color_stop(hsla(solid), 1.),
                            )),
                    )
                    .child(handle(a as f32 / 255.))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, e: &MouseDownEvent, window, cx| {
                            this.opacity_at(e.position, window, cx)
                        }),
                    )
                    .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, window, cx| {
                        if e.pressed_button == Some(MouseButton::Left) {
                            this.opacity_at(e.position, window, cx)
                        }
                    })),
            );

        let field = |i: usize| {
            let wrong = self.problem.as_ref().is_some_and(|(p, _)| *p == i);
            v_flex()
                .gap(px(2.))
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(FIELDS[i])
                .child(
                    Input::new(&self.fields[i])
                        .small()
                        .font_family(theme.mono_font_family.clone())
                        .when(wrong, |input| input.border_color(theme.danger)),
                )
        };
        v_flex()
            .gap_3()
            .child(
                h_flex().gap_3().items_start().child(wheel).child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .gap_2()
                        .child(
                            h_flex()
                                .gap_2()
                                .child(field(0).flex_1())
                                .child(field(1).flex_1()),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .child(field(2).flex_1())
                                .child(field(3).flex_1()),
                        )
                        .child(field(4)),
                ),
            )
            .when_some(self.problem.as_ref(), |el, (_, why)| {
                el.child(div().text_xs().text_color(theme.danger).child(why.clone()))
            })
            .child(brightness_bar)
            .child(opacity_bar)
            .into_any_element()
    }

    /// The 16 palette colours, by number.
    pub fn palette(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        div()
            .grid()
            .grid_cols(8)
            .gap_x_1p5()
            .gap_y_2()
            .children(PALETTE.iter().enumerate().map(|(n, rgba)| {
                let selected = self.form == Form::Palette(n as u8);
                v_flex()
                    .id(("palette", n))
                    .items_center()
                    .gap(px(2.))
                    .cursor_pointer()
                    .child(
                        div()
                            .w_full()
                            .h(px(28.))
                            .rounded_md()
                            .border_1()
                            .border_color(theme.border)
                            .when(selected, |d| d.border_2().border_color(theme.foreground))
                            .bg(hsla(*rgba)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_family(theme.mono_font_family.clone())
                            .text_color(theme.muted_foreground)
                            .child(n.to_string()),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.pick_palette(n as u8, window, cx)
                    }))
            }))
            .into_any_element()
    }

    /// The names, in colour order, with the filter.
    pub fn names(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let filter = self.filter.read(cx).value().to_string();
        let names = names_matching(&filter);
        let count = names.len();
        v_flex()
            .gap_2()
            .child(Input::new(&self.filter).small().cleanable(true))
            .when(!filter.trim().is_empty(), |el| {
                el.child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(match count {
                            0 => format!("No name has \"{}\" in it.", filter.trim()),
                            1 => format!("1 name has \"{}\" in it.", filter.trim()),
                            n => format!("{n} names have \"{}\" in it.", filter.trim()),
                        }),
                )
            })
            .child(
                div()
                    .grid()
                    .grid_cols(3)
                    .gap_x_1()
                    .children(names.into_iter().map(|(name, rgba)| {
                        let selected = self.form == Form::Name(name);
                        h_flex()
                            .id(SharedString::from(format!("name-{name}")))
                            .gap_1p5()
                            .min_w_0()
                            .px_1()
                            .py(px(3.))
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|d| d.bg(theme.muted))
                            .when(selected, |d| {
                                d.bg(theme.muted).border_1().border_color(theme.foreground)
                            })
                            .child(
                                div()
                                    .flex_none()
                                    .size(px(16.))
                                    .rounded_sm()
                                    .border_1()
                                    .border_color(theme.border)
                                    .bg(hsla(rgba)),
                            )
                            .child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_xs()
                                    .font_family(theme.mono_font_family.clone())
                                    .child(name),
                            )
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.pick_name(name, rgba, window, cx)
                            }))
                    })),
            )
            .into_any_element()
    }

    /// The current colour, its form, and Copy to editor, at the bottom of
    /// the page. Copy to editor is off while a program runs, since the
    /// editor is read-only then, but not while it's paused (ADR 0028).
    pub fn bar(&self, running: bool, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let rgba = self.rgba();
        let copy = Button::new("copy-colour")
            .primary()
            .label("Copy to editor")
            .disabled(running)
            .when(running, |b| b.tooltip("Pause the program to change it"))
            .on_click(cx.listener(|this, _, window, cx| this.copy(window, cx)));
        v_flex()
            .flex_none()
            .gap_2()
            .px_3()
            .py_2()
            .border_t_1()
            .border_color(theme.border)
            .bg(theme.muted.opacity(0.5))
            .child(
                h_flex()
                    .gap_3()
                    .child(swatch(self.images.clone(), rgba, 36., cx))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.muted_foreground)
                                    .child("CURRENT COLOUR"),
                            )
                            .child(
                                div()
                                    .font_family(theme.mono_font_family.clone())
                                    .text_sm()
                                    .truncate()
                                    .child(self.text()),
                            ),
                    )
                    .child(copy),
            )
            .child(
                h_flex()
                    .gap_1()
                    .flex_wrap()
                    .children(forms(rgba).into_iter().map(|form| {
                        let button =
                            Button::new(SharedString::from(format!("form-{}", form.label())))
                                .xsmall()
                                .label(form.label());
                        // The chosen form is solid, so it stands out.
                        if self.form == form {
                            button.primary()
                        } else {
                            button.outline()
                        }
                        .on_click(cx.listener(move |this, _, _, cx| this.choose(form, cx)))
                    })),
            )
            .into_any_element()
    }
}
