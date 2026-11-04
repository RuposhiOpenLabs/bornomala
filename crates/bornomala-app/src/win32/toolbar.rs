//! Floating Toolbar for Bornomala — Authentic SVG Redesign.
//! Renders 100% faithful to the user's prototype design using pre-rasterized
//! 32-bit alpha-blended vector assets from /home/ryan/Desktop/tmp/bornomaldata/.

use super::api::*;
use super::assets::{self, IconId};
use super::converter::ConverterWindow;
use bornomala_engine::InputScheme;
use std::sync::atomic::{AtomicU32, Ordering};

pub const TOOLBAR_WIDTH: i32 = 366;
pub const TOOLBAR_HEIGHT: i32 = 36;

pub static CURRENT_SCHEME: AtomicU32 = AtomicU32::new(3); // 0=Unicode, 1=Ansi, 2=Phonetic, 3=PassThrough (Default: English)
static HOVERED_BTN: AtomicU32 = AtomicU32::new(0); // 0=None, 1=Pill, 2=Tray, 3=Menu, 4=Close

static mut TOOLBAR_HWND: HWND = 0;
static mut FONT_TITLE: HFONT = 0;
static mut FONT_PILL: HFONT = 0;

// Coordinates of interactive elements matching prototype layout
const RECT_SEL_PILL:    RECT = RECT { left: 136, top: 5, right: 248, bottom: 31 };
const RECT_BTN_TRAY:    RECT = RECT { left: 260, top: 6, right: 284, bottom: 30 };
const RECT_BTN_MENU:    RECT = RECT { left: 288, top: 6, right: 312, bottom: 30 };
const RECT_BTN_CLOSE:   RECT = RECT { left: 316, top: 6, right: 340, bottom: 30 };

// Command IDs
const ID_MODE_UNICODE: usize = 101;
const ID_MODE_ANSI: usize = 102;
const ID_MODE_PHONETIC: usize = 103;
const ID_MODE_ENGLISH: usize = 104;

const ID_MENU_CONVERTER: usize = 201;
const ID_MENU_TRAY: usize = 202;
const ID_MENU_KEYBOARD: usize = 203;
const ID_MENU_SETTINGS: usize = 204;
const ID_MENU_ABOUT: usize = 205;
const ID_MENU_UPDATE: usize = 206;
const ID_MENU_EXIT: usize = 207;

#[allow(dead_code)]
pub struct FloatingToolbar {
    pub hwnd: HWND,
}

impl FloatingToolbar {
    pub fn create(hinstance: HINSTANCE) -> Option<Self> {
        unsafe {
            let class_name = to_wide("BornomalaToolbarClass");
            let mut wc: WNDCLASSEXW = std::mem::zeroed();
            wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
            wc.lpfnWndProc = Some(toolbar_wnd_proc);
            wc.hInstance = hinstance;
            wc.hCursor = LoadCursorW(0, 32512 as *const u16);
            wc.hbrBackground = CreateSolidBrush(rgb(251, 243, 220)); // Parchment ink: #fbf3dc
            wc.lpszClassName = class_name.as_ptr();

            RegisterClassExW(&wc);

            let font_name = to_wide("Nirmala UI");
            FONT_TITLE = CreateFontW(
                -13, 0, 0, 0, FW_BOLD, 0, 0, 0,
                DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
                font_name.as_ptr(),
            );
            FONT_PILL = CreateFontW(
                -11, 0, 0, 0, FW_BOLD, 0, 0, 0,
                DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
                font_name.as_ptr(),
            );

            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let x = (screen_w - TOOLBAR_WIDTH) / 2;
            let y = 14;

            let title = to_wide("Bornomala Toolbar");
            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_POPUP,
                x,
                y,
                TOOLBAR_WIDTH,
                TOOLBAR_HEIGHT,
                0,
                0,
                hinstance,
                std::ptr::null_mut(),
            );

            if hwnd == 0 {
                return None;
            }

            let rgn = CreateRoundRectRgn(0, 0, TOOLBAR_WIDTH + 1, TOOLBAR_HEIGHT + 1, 18, 18);
            SetWindowRgn(hwnd, rgn, TRUE);

            TOOLBAR_HWND = hwnd;

            Some(Self { hwnd })
        }
    }

    pub fn show() {
        unsafe {
            if TOOLBAR_HWND != 0 {
                ShowWindow(TOOLBAR_HWND, SW_SHOW);
                SetWindowPos(
                    TOOLBAR_HWND,
                    HWND_TOPMOST,
                    0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
                );
                InvalidateRect(TOOLBAR_HWND, std::ptr::null(), TRUE);
                UpdateWindow(TOOLBAR_HWND);

                // User opened the toolbar; persist preference so it doesn't stay hidden
                let mut cfg = bornomala_engine::AppConfig::load();
                if cfg.toolbar_minimized {
                    cfg.toolbar_minimized = false;
                    let _ = cfg.save();
                }
            }
        }
    }

    pub fn get_hwnd() -> HWND {
        unsafe { TOOLBAR_HWND }
    }

    pub fn set_scheme(scheme: InputScheme) {
        let code = match scheme {
            InputScheme::ClassicUnicode => 0,
            InputScheme::ClassicAnsi => 1,
            InputScheme::Phonetic => 2,
            InputScheme::PassThrough => 3,
        };
        CURRENT_SCHEME.store(code, Ordering::SeqCst);
        unsafe {
            if TOOLBAR_HWND != 0 {
                InvalidateRect(TOOLBAR_HWND, std::ptr::null(), TRUE);
            }
        }
    }

    pub fn get_scheme() -> InputScheme {
        match CURRENT_SCHEME.load(Ordering::SeqCst) {
            1 => InputScheme::ClassicAnsi,
            2 => InputScheme::Phonetic,
            3 => InputScheme::PassThrough,
            _ => InputScheme::ClassicUnicode,
        }
    }

    pub fn toggle_visibility() {
        unsafe {
            if TOOLBAR_HWND != 0 {
                if IsWindowVisible(TOOLBAR_HWND) != 0 {
                    super::minimize_toolbar_to_tray();
                } else {
                    Self::show();
                }
            }
        }
    }

    pub fn minimize() {
        unsafe {
            if TOOLBAR_HWND != 0 {
                ShowWindow(TOOLBAR_HWND, SW_HIDE);
            }
        }
    }
}

unsafe extern "system" fn toolbar_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let color_green = rgb(6, 106, 74);    // #066a4a
            let color_paper = rgb(251, 243, 220); // #fbf3dc

            // 1. Fill Background
            let bg_brush = CreateSolidBrush(color_paper);
            let mut client_rc: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut client_rc);
            FillRect(hdc, &client_rc, bg_brush);
            DeleteObject(bg_brush);

            // 2. Outer 2px Green Pill Border
            let border_pen = CreatePen(PS_SOLID, 2, color_green);
            let parchment_brush = CreateSolidBrush(color_paper);
            let old_brush = SelectObject(hdc, parchment_brush);
            let old_pen = SelectObject(hdc, border_pen);
            RoundRect(hdc, 1, 1, TOOLBAR_WIDTH - 1, TOOLBAR_HEIGHT - 1, 18, 18);

            // 3. Left Ornament from authentic ornament.svg
            assets::draw_icon(hdc, IconId::OrnamentLeft, 12, 6);

            // 4. Authentic Logo from logo.svg
            assets::draw_icon(hdc, IconId::Logo, 28, 6);

            // 5. Divider 1 (1px green vertical line)
            let div_pen = CreatePen(PS_SOLID, 1, color_green);
            SelectObject(hdc, div_pen);
            MoveToEx(hdc, 60, 7, std::ptr::null_mut());
            LineTo(hdc, 60, 29);

            // 6. Wordmark "বর্ণমালা"
            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, color_green);
            SelectObject(hdc, FONT_TITLE);
            let mut word_rc = RECT { left: 66, top: 4, right: 132, bottom: 32 };
            let wide_word = to_wide("বর্ণমালা");
            DrawTextW(hdc, wide_word.as_ptr(), -1, &mut word_rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

            // 7. Language Selector Pill [ Flag | Mode ∨ ]
            SelectObject(hdc, parchment_brush);
            let pill_pen = CreatePen(PS_SOLID, 2, color_green);
            SelectObject(hdc, pill_pen);
            RoundRect(hdc, RECT_SEL_PILL.left, RECT_SEL_PILL.top, RECT_SEL_PILL.right, RECT_SEL_PILL.bottom, 10, 10);
            DeleteObject(pill_pen);

            let current = CURRENT_SCHEME.load(Ordering::SeqCst);

            // Draw Flag / En Badge inside Pill
            if current == 3 {
                assets::draw_icon(hdc, IconId::FlagEn, 142, 11);
            } else {
                assets::draw_icon(hdc, IconId::FlagBd, 142, 11);
            }

            // Mode Label text
            let mode_text = match current {
                0 => "Unicode",
                1 => "Bijoy ANSI",
                2 => "Phonetic",
                _ => "English",
            };
            SetTextColor(hdc, color_green);
            SelectObject(hdc, FONT_PILL);
            let mut label_rc = RECT { left: 166, top: 5, right: 232, bottom: 31 };
            let wide_mode = to_wide(mode_text);
            DrawTextW(hdc, wide_mode.as_ptr(), -1, &mut label_rc, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

            // Down chevron ∨
            let chev_pen = CreatePen(PS_SOLID, 2, color_green);
            SelectObject(hdc, chev_pen);
            MoveToEx(hdc, 236, 16, std::ptr::null_mut());
            LineTo(hdc, 239, 19);
            LineTo(hdc, 242, 16);
            DeleteObject(chev_pen);

            // 8. Divider 2 (1px green vertical line)
            SelectObject(hdc, div_pen);
            MoveToEx(hdc, 254, 7, std::ptr::null_mut());
            LineTo(hdc, 254, 29);

            // 9. Tool Buttons (SendToTray, Menu, Close)
            let hover_id = HOVERED_BTN.load(Ordering::Relaxed);
            let draw_hover_bg = |rc: &RECT, id: u32| {
                if hover_id == id {
                    let (bg_col, border_col) = if id == 4 {
                        (rgb(253, 235, 235), rgb(240, 180, 180)) // Gentle crimson tint on close hover
                    } else {
                        (rgb(238, 228, 202), rgb(215, 202, 175)) // Warm parchment hover
                    };
                    let hover_brush = CreateSolidBrush(bg_col);
                    let hover_pen = CreatePen(PS_SOLID, 1, border_col);
                    let prev_brush = SelectObject(hdc, hover_brush);
                    let prev_pen = SelectObject(hdc, hover_pen);
                    RoundRect(hdc, rc.left, rc.top, rc.right, rc.bottom, 6, 6);
                    SelectObject(hdc, prev_brush);
                    SelectObject(hdc, prev_pen);
                    DeleteObject(hover_brush);
                    DeleteObject(hover_pen);
                }
            };

            // Send to Tray tool (24x24)
            draw_hover_bg(&RECT_BTN_TRAY, 2);
            assets::draw_icon(hdc, IconId::SendToTray, 263, 9);

            // Menu tool (24x24)
            draw_hover_bg(&RECT_BTN_MENU, 3);
            assets::draw_icon(hdc, IconId::Menu, 291, 9);

            // Close tool (24x24)
            draw_hover_bg(&RECT_BTN_CLOSE, 4);
            assets::draw_icon(hdc, IconId::Close, 319, 9);

            // 10. Right Ornament from authentic ornament.svg (mirrored with red center)
            assets::draw_icon(hdc, IconId::OrnamentRight, 346, 6);

            // Cleanup GDI objects
            SelectObject(hdc, old_pen);
            SelectObject(hdc, old_brush);
            DeleteObject(border_pen);
            DeleteObject(parchment_brush);
            DeleteObject(div_pen);

            EndPaint(hwnd, &ps);
            0
        }

        WM_MOUSEMOVE => {
            let x = (lparam & 0xFFFF) as i32;
            let y = ((lparam >> 16) & 0xFFFF) as i32;
            let pt = POINT { x, y };

            let in_rc = |rc: &RECT| -> bool {
                pt.x >= rc.left && pt.x <= rc.right && pt.y >= rc.top && pt.y <= rc.bottom
            };

            let new_hover = if in_rc(&RECT_SEL_PILL) {
                1
            } else if in_rc(&RECT_BTN_TRAY) {
                2
            } else if in_rc(&RECT_BTN_MENU) {
                3
            } else if in_rc(&RECT_BTN_CLOSE) {
                4
            } else {
                0
            };

            let old_hover = HOVERED_BTN.swap(new_hover, Ordering::SeqCst);
            if old_hover != new_hover {
                InvalidateRect(hwnd, std::ptr::null(), TRUE);
            }

            let mut tme: TRACKMOUSEEVENT = std::mem::zeroed();
            tme.cbSize = std::mem::size_of::<TRACKMOUSEEVENT>() as u32;
            tme.dwFlags = TME_LEAVE;
            tme.hwndTrack = hwnd;
            TrackMouseEvent(&mut tme);

            0
        }

        WM_MOUSELEAVE => {
            HOVERED_BTN.store(0, Ordering::SeqCst);
            InvalidateRect(hwnd, std::ptr::null(), TRUE);
            0
        }

        WM_LBUTTONDOWN => {
            let x = (lparam & 0xFFFF) as i32;
            let y = ((lparam >> 16) & 0xFFFF) as i32;
            let pt = POINT { x, y };

            let in_rect = |rc: &RECT| -> bool {
                pt.x >= rc.left && pt.x <= rc.right && pt.y >= rc.top && pt.y <= rc.bottom
            };

            // 1. Clicked Language Selector Pill: Open Dropdown Scheme Menu
            if in_rect(&RECT_SEL_PILL) {
                let mut screen_pt = POINT { x: RECT_SEL_PILL.left, y: RECT_SEL_PILL.bottom + 2 };
                ClientToScreen(hwnd, &mut screen_pt);

                let hmenu = CreatePopupMenu();
                let current = CURRENT_SCHEME.load(Ordering::SeqCst);

                let add_menu_item = |menu: HMENU, id: usize, label: &str, is_checked: bool| {
                    let mut flags = MF_STRING;
                    if is_checked {
                        flags |= MF_CHECKED;
                    }
                    let wide = to_wide(label);
                    AppendMenuW(menu, flags, id, wide.as_ptr());
                };

                add_menu_item(hmenu, ID_MODE_UNICODE, "🇧🇩  ইউনিকোড (Unicode)\tCtrl+Alt+V", current == 0);
                add_menu_item(hmenu, ID_MODE_ANSI, "🇧🇩  বিজয় এএনএসআই (Bijoy ANSI)\tCtrl+Alt+B", current == 1);
                add_menu_item(hmenu, ID_MODE_PHONETIC, "🇧🇩  ফোনেটিক (Avro Phonetic)\tF12 / Ctrl+Space", current == 2);
                AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
                add_menu_item(hmenu, ID_MODE_ENGLISH, "🇬🇧  English (Pass-through)\tCtrl+Alt+E", current == 3);

                SetForegroundWindow(hwnd);
                TrackPopupMenu(hmenu, 0, screen_pt.x, screen_pt.y, 0, hwnd, std::ptr::null());
                DestroyMenu(hmenu);
                return 0;
            }

            // 2. Clicked Send to Tray button
            if in_rect(&RECT_BTN_TRAY) {
                super::minimize_toolbar_to_tray();
                return 0;
            }

            // 3. Clicked Menu button (☰) — Features Popup Menu
            if in_rect(&RECT_BTN_MENU) {
                let mut screen_pt = POINT { x: RECT_BTN_MENU.left, y: RECT_BTN_MENU.bottom + 4 };
                ClientToScreen(hwnd, &mut screen_pt);

                let hmenu = CreatePopupMenu();
                let add_menu_item = |menu: HMENU, id: usize, label: &str| {
                    let wide = to_wide(label);
                    AppendMenuW(menu, MF_STRING, id, wide.as_ptr());
                };
                let add_sep = |menu: HMENU| {
                    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
                };

                add_menu_item(hmenu, ID_MENU_KEYBOARD, "⌨  কীবোর্ড লেআউট গাইড (Layout Guide)...");
                add_menu_item(hmenu, ID_MENU_SETTINGS, "⚙  কনফিগারেশন ও সেটিংস (Settings)...");
                add_menu_item(hmenu, ID_MENU_ABOUT, "ℹ  বর্ণমালা পরিচিতি (About Bornomala)...");
                add_sep(hmenu);
                add_menu_item(hmenu, ID_MENU_CONVERTER, "🔄  বাংলা রূপান্তরকারী (Converter)...");
                add_menu_item(hmenu, ID_MENU_TRAY, "⬇  টাস্কবারে পাঠান (Send to system tray)");
                add_menu_item(hmenu, ID_MENU_UPDATE, "🔄  আপডেট পরীক্ষা করুন (Check for Updates)...");
                add_sep(hmenu);
                add_menu_item(hmenu, ID_MENU_EXIT, "✕  প্রস্থান (Exit Bornomala)");

                SetForegroundWindow(hwnd);
                TrackPopupMenu(hmenu, 0, screen_pt.x, screen_pt.y, 0, hwnd, std::ptr::null());
                DestroyMenu(hmenu);
                return 0;
            }

            // 4. Clicked Close button (✕) — Clean Exit
            if in_rect(&RECT_BTN_CLOSE) {
                PostQuitMessage(0);
                return 0;
            }

            // Clicked background: Smooth window dragging!
            ReleaseCapture();
            SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as WPARAM, 0);
            0
        }

        WM_RBUTTONUP => {
            let mut screen_pt: POINT = std::mem::zeroed();
            GetCursorPos(&mut screen_pt);

            let hmenu = CreatePopupMenu();
            let current = CURRENT_SCHEME.load(Ordering::SeqCst);

            let add_menu_item = |menu: HMENU, id: usize, label: &str, is_checked: bool| {
                let mut flags = MF_STRING;
                if is_checked {
                    flags |= MF_CHECKED;
                }
                let wide = to_wide(label);
                AppendMenuW(menu, flags, id, wide.as_ptr());
            };
            let add_sep = |menu: HMENU| {
                AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
            };

            add_menu_item(hmenu, ID_MODE_UNICODE, "🇧🇩  ইউনিকোড (Unicode)\tCtrl+Alt+V", current == 0);
            add_menu_item(hmenu, ID_MODE_ANSI, "🇧🇩  বিজয় এএনএসআই (Bijoy ANSI)\tCtrl+Alt+B", current == 1);
            add_menu_item(hmenu, ID_MODE_PHONETIC, "🇧🇩  ফোনেটিক (Avro Phonetic)\tF12 / Ctrl+Space", current == 2);
            add_sep(hmenu);
            add_menu_item(hmenu, ID_MODE_ENGLISH, "🇬🇧  English (Pass-through)\tCtrl+Alt+E", current == 3);
            add_sep(hmenu);
            add_menu_item(hmenu, ID_MENU_KEYBOARD, "⌨  কীবোর্ড লেআউট গাইড (Layout Guide)...", false);
            add_menu_item(hmenu, ID_MENU_SETTINGS, "⚙  কনফিগারেশন ও সেটিংস (Settings)...", false);
            add_menu_item(hmenu, ID_MENU_ABOUT, "ℹ  বর্ণমালা পরিচিতি (About Bornomala)...", false);
            add_sep(hmenu);
            add_menu_item(hmenu, ID_MENU_CONVERTER, "🔄  বাংলা রূপান্তরকারী (Converter)...", false);
            add_menu_item(hmenu, ID_MENU_TRAY, "⬇  টাস্কবারে পাঠান (Send to system tray)", false);
            add_menu_item(hmenu, ID_MENU_UPDATE, "🔄  আপডেট পরীক্ষা করুন (Check for Updates)...", false);
            add_sep(hmenu);
            add_menu_item(hmenu, ID_MENU_EXIT, "✕  প্রস্থান (Exit Bornomala)", false);

            SetForegroundWindow(hwnd);
            TrackPopupMenu(hmenu, 0, screen_pt.x, screen_pt.y, 0, hwnd, std::ptr::null());
            DestroyMenu(hmenu);
            0
        }

        WM_COMMAND => {
            let cmd_id = wparam & 0xFFFF;
            match cmd_id {
                ID_MODE_UNICODE => {
                    FloatingToolbar::set_scheme(InputScheme::ClassicUnicode);
                }
                ID_MODE_ANSI => {
                    FloatingToolbar::set_scheme(InputScheme::ClassicAnsi);
                }
                ID_MODE_PHONETIC => {
                    FloatingToolbar::set_scheme(InputScheme::Phonetic);
                }
                ID_MODE_ENGLISH => {
                    FloatingToolbar::set_scheme(InputScheme::PassThrough);
                }
                ID_MENU_CONVERTER => {
                    ConverterWindow::open();
                }
                ID_MENU_TRAY => {
                    super::minimize_toolbar_to_tray();
                }
                ID_MENU_KEYBOARD => {
                    let title = to_wide("বর্ণমালা — কীবোর্ড লেআউট গাইড");
                    let msg = to_wide(
                        "বর্ণমালা কীবোর্ড শর্টকাটসমূহ:\n\n\
                        • Ctrl + Alt + V : ক্লাসিক ইউনিকোড মোড\n\
                        • Ctrl + Alt + B : বিজয় এএনএসআই মোড (SutonnyMJ)\n\
                        • F12 বা Ctrl + Space : ফোনেটিক বাংলা মোড\n\
                        • Ctrl + Alt + E : ইংরেজি (Pass-through)\n\n\
                        যুক্তবর্ণ লেখার নিয়ম:\n\
                        প্রথম বর্ণ + G (হসন্ত) + দ্বিতীয় বর্ণ\n\
                        উদাহরণ: k + g + k = ক্ক, k + g + t = ক্ত"
                    );
                    MessageBoxW(hwnd, msg.as_ptr(), title.as_ptr(), 0x00000040);
                }
                ID_MENU_SETTINGS => {
                    super::settings::SettingsWindow::open();
                }
                ID_MENU_ABOUT => {
                    super::about::AboutWindow::open();
                }
                ID_MENU_UPDATE => {
                    super::updater::trigger_manual_update_check("0.1.0");
                }
                ID_MENU_EXIT => {
                    PostQuitMessage(0);
                }
                _ => {}
            }
            0
        }

        WM_DESTROY => {
            if FONT_TITLE != 0 { DeleteObject(FONT_TITLE); FONT_TITLE = 0; }
            if FONT_PILL != 0 { DeleteObject(FONT_PILL); FONT_PILL = 0; }
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
