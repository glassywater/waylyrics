use crate::EXCLUDED_REGEXES;

use gtk::prelude::*;
use gtk::Label;

use super::window;

#[cfg(target_os = "windows")]
pub(super) fn set_click_pass_through(window: &window::Window, enabled: bool) {
    use std::ffi::c_void;

    fn set_window_click_through(hwnd: *mut c_void, enabled: bool) {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE,
        };
        let hwnd = HWND(hwnd as _);

        const WS_EX_TRANSPARENT: isize = 0x00000020;
        const WS_EX_LAYERED: isize = 0x00080000;
        unsafe {
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            if enabled {
                SetWindowLongPtrW(
                    hwnd,
                    GWL_EXSTYLE,
                    ex_style | WS_EX_TRANSPARENT | WS_EX_LAYERED,
                );
            } else {
                SetWindowLongPtrW(
                    hwnd,
                    GWL_EXSTYLE,
                    ex_style & !WS_EX_TRANSPARENT & !WS_EX_LAYERED,
                );
            }
        }
    }

    let Some(surface) = window.surface().and_downcast::<gdk4_win32::Win32Surface>() else {
        return;
    };

    let handle = surface.handle().0;

    set_window_click_through(handle, enabled);
}

#[cfg(not(target_os = "windows"))]
pub(super) fn set_click_pass_through(window: &window::Window, enabled: bool) {
    use gtk::cairo::{RectangleInt, Region};
    use gtk::subclass::prelude::*;

    let obj = window;
    let Some(surface) = obj.surface() else {
        return;
    };

    if enabled {
        if !window.is_decorated() {
            surface.set_input_region(Some(&Region::create_rectangle(&RectangleInt::new(
                0, 0, 0, 0,
            ))));
        } else {
            let headerbar = &window.imp().headerbar;
            let allocation = headerbar.allocation();

            surface.set_input_region(Some(&Region::create_rectangle(&RectangleInt::new(
                allocation.x(),
                allocation.y(),
                allocation.width(),
                allocation.height(),
            ))));
        }
    } else {
        surface.set_input_region(Some(&Region::create_rectangle(&RectangleInt::new(
            0,
            0,
            i32::MAX,
            i32::MAX,
        ))));
    }
}

/// set css style for waylyrics
/// As said in [GTK+ doc], gtk constructs style from the lower priority ones to the upper ones,
/// We set priority as `STYLE_PROVIDER_PRIORITY + 1` to override user theme
///
/// [GTK+ doc]: https://docs.gtk.org/gtk4/type_func.StyleContext.add_provider_for_display.html#parameters
pub fn merge_css(css: &str) {
    use gtk::gdk::Display as GdkDisplay;
    use gtk::CssProvider;
    use std::cell::RefCell;

    thread_local! {
        static LATEST_PROVIDER: RefCell<Option<CssProvider>> = const { RefCell::new(None) };
    }
    let css_provider = CssProvider::new();
    css_provider.load_from_data(css);
    let display = GdkDisplay::default().expect("Could not connect to a display.");
    LATEST_PROVIDER.with_borrow_mut(|provider| {
        if let Some(provider) = provider.take() {
            gtk::style_context_remove_provider_for_display(&display, &provider);
        }
    });

    gtk::style_context_add_provider_for_display(
        &display,
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER + 1,
    );
    LATEST_PROVIDER.with_borrow_mut(|provider| {
        *provider = Some(css_provider);
    });
}

fn has_filtered_word(text: &str) -> bool {
    EXCLUDED_REGEXES.with_borrow(|regex_set| regex_set.is_match(text))
}

pub fn setup_label(label: &Label, hide_filtered_words: bool) {
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::Word);

    if hide_filtered_words {
        label.connect_label_notify(|label| {
            let text = label.label();
            let visible = !has_filtered_word(&text) && !text.is_empty();
            label.set_visible(visible);
        });
    } else {
        label.connect_label_notify(|label| {
            let visible = !label.label().is_empty();
            label.set_visible(visible);
        });
    }
}

/// 绘制带 ruby 注音的文本
///
/// 在 Cairo 上下文中绘制日文歌词，汉字上方显示平假名注音
pub fn draw_ruby_text(
    cr: &gtk::cairo::Context,
    width: i32,
    height: i32,
    ruby_line: &crate::lyric_providers::RubyLine,
    base_font_size: f64,
    ruby_font_size: f64,
    font_family: &str,
    text_color: (f64, f64, f64),
    ruby_color: (f64, f64, f64),
) {
    let layout = pangocairo::functions::create_layout(cr);

    // 计算总宽度用于居中
    let full_text: String = ruby_line.segments.iter().map(|s| s.text.as_str()).collect();
    let base_desc = pango::FontDescription::from_string(&format!("{} {}", font_family, base_font_size));
    layout.set_font_description(Some(&base_desc));
    layout.set_text(&full_text);
    let (total_w, _) = layout.pixel_size();

    let mut current_x = (width as f64 - total_w as f64) / 2.0;
    let base_y = height as f64 / 2.0;

    for segment in &ruby_line.segments {
        // 测量基础文字宽度
        let base_desc = pango::FontDescription::from_string(&format!("{} {}", font_family, base_font_size));
        layout.set_font_description(Some(&base_desc));
        layout.set_text(&segment.text);
        let (base_w, _base_h) = layout.pixel_size();

        // 绘制基础文字
        cr.set_source_rgb(text_color.0, text_color.1, text_color.2);
        cr.move_to(current_x, base_y);
        pangocairo::functions::show_layout(cr, &layout);

        // 如果有注音，在上方绘制
        if let Some(ref reading) = segment.reading {
            let ruby_desc = pango::FontDescription::from_string(&format!("{} {}", font_family, ruby_font_size));
            layout.set_font_description(Some(&ruby_desc));
            layout.set_text(reading);
            let (ruby_w, ruby_h) = layout.pixel_size();

            // 居中对齐注音到基础上方
            let ruby_x = current_x + (base_w as f64 - ruby_w as f64) / 2.0;
            let ruby_y = base_y - ruby_h as f64 - 2.0;

            cr.set_source_rgb(ruby_color.0, ruby_color.1, ruby_color.2);
            cr.move_to(ruby_x, ruby_y);
            pangocairo::functions::show_layout(cr, &layout);

            // 重置属性
            layout.set_attributes(None);
        }

        current_x += base_w as f64;
    }
}
