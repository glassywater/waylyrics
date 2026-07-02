pub mod search_window;
mod window;

use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{Application, Label};
pub use window::Window;

use crate::app::utils::set_click_pass_through;
use crate::{config, DEFAULT_TEXT};

const WINDOW_MIN_HEIGHT: i32 = 120;

pub mod actions;
pub mod dialog;
pub mod utils;

/// 当前正在显示的 ruby 行索引
pub static CURRENT_RUBY_INDEX: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub fn build_main_window(
    app: &Application,
    enable_filter_regex: bool,
    cache_lyrics: bool,
    length_toleration_ms: u128,
    show_default_text_on_idle: bool,
    show_lyric_on_pause: bool,
    font_family: &str,
    #[cfg(feature = "layer-shell")] layer_shell: bool,
    #[cfg(feature = "layer-shell")] layer_shell_anchor: crate::config::LayerShellAnchor,
) -> Window {
    let window = Window::new(
        app,
        cache_lyrics,
        length_toleration_ms,
        show_default_text_on_idle,
        show_lyric_on_pause,
    );

    #[cfg(feature = "layer-shell")]
    if layer_shell {
        use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};

        LayerShell::init_layer_shell(&window);
        // wlr-layer-shell-unstable-v1 version 4 needed, see
        // https://wayland.app/protocols/wlr-layer-shell-unstable-v1#compositor-support
        // and https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:enum:keyboard_interactivity
        LayerShell::set_keyboard_mode(&window, KeyboardMode::OnDemand);
        LayerShell::set_layer(&window, Layer::Overlay);

        layer_shell_anchor.setup(&window);
    }

    window.set_size_request(500, WINDOW_MIN_HEIGHT);
    window.set_title(Some(DEFAULT_TEXT));
    window.set_icon_name(Some(crate::APP_ID_FIXED));
    window.present();

    let above_label = Label::builder()
        .label("Waylyrics")
        .name("above")
        .vexpand(true)
        .hexpand(false)
        .build();
    let below_label = Label::builder()
        .label("")
        .name("below")
        .vexpand(true)
        .hexpand(false)
        .visible(false)
        .build();

    for label in [&above_label, &below_label] {
        utils::setup_label(label, enable_filter_regex);
    }

    // 创建 ruby 注音绘制区域
    let ruby_drawing_area = gtk::DrawingArea::builder()
        .name("ruby")
        .vexpand(true)
        .hexpand(false)
        .content_width(500)
        .content_height(100)
        .visible(false)
        .build();

    // 设置 draw 回调
    let font_family_clone = font_family.to_string();
    ruby_drawing_area.set_draw_func(move |_area, cr, width, height| {
        // 获取当前的 ruby 数据并绘制
        crate::sync::LYRIC.with_borrow(|lyric_state| {
            if let Some(ref ruby_lines) = lyric_state.ruby {
                let idx = CURRENT_RUBY_INDEX.load(std::sync::atomic::Ordering::Relaxed);
                let ruby_line = ruby_lines.get(idx).or_else(|| ruby_lines.first());
                if let Some(ruby_line) = ruby_line {
                    utils::draw_ruby_text(
                        cr,
                        width,
                        height,
                        ruby_line,
                        24.0, // 基础字号
                        12.0, // 注音字号
                        &font_family_clone,
                        (1.0, 1.0, 1.0), // 白色文本
                        (0.7, 0.7, 0.7), // 浅灰色注音
                    );
                }
            }
        });
    });

    // 存储 DrawingArea 引用
    let _ = window.imp().ruby_drawing_area.set(ruby_drawing_area.clone());

    let verical_box = gtk::Box::builder()
        .name("lyrics-box")
        .baseline_position(gtk::BaselinePosition::Center)
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .vexpand(true)
        .hexpand(false)
        .build();

    verical_box.insert_child_after(&above_label, gtk::Box::NONE);
    verical_box.insert_child_after(&ruby_drawing_area, Some(&above_label));
    verical_box.insert_child_after(&below_label, Some(&ruby_drawing_area));

    window.set_child(Some(&verical_box));

    let align = window.imp().lyric_align.get();
    set_lyric_align(&window, align);

    window.connect_decorated_notify(|window| {
        crate::log::debug!("triggered decorated signal");
        let clickthrough = window.imp().clickthrough.get();
        set_click_pass_through(window, clickthrough)
    });

    window.set_icon_name(Some(crate::APP_ID_FIXED));

    window
}

pub fn set_lyric_align(window: &Window, align: config::Align) -> Option<()> {
    let vbox: gtk::Box = window.child()?.downcast().ok()?;
    vbox.set_halign(align.into());

    let labels = get_labels(&vbox)?;
    for label in labels {
        label.set_halign(align.into());
        label.set_justify(align.into());
    }

    // 更新 ruby drawing area 的对齐
    if let Some(ruby_da) = window.imp().ruby_drawing_area.get() {
        ruby_da.set_halign(align.into());
    }

    window.imp().lyric_align.set(align);
    Some(())
}

fn get_labels(vbox: &gtk::Box) -> Option<[Label; 2]> {
    let above_label: Label = vbox.first_child()?.downcast().ok()?;
    let below_label: Label = vbox.last_child()?.downcast().ok()?;
    Some([above_label, below_label])
}

pub fn get_label(window: &Window, position: &str) -> Label {
    let Some(vbox) = window.child().and_then(|c| c.downcast::<gtk::Box>().ok()) else {
        panic!("initialization failed!")
    };
    get_labels(&vbox)
        .expect("cannot find labels")
        .into_iter()
        .find(|label| label.widget_name() == position)
        .unwrap()
}

/// 获取 ruby 注音绘制区域
pub fn get_ruby_drawing_area(window: &Window) -> Option<&gtk::DrawingArea> {
    window.imp().ruby_drawing_area.get()
}
