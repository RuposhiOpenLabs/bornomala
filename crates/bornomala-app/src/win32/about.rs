//! Bornomala Authentic Desi About & Credits Window
//!
//! Styled in authentic Desi Bengali theme: Forest Green (#066a4a), Crimson Red (#e02f2d),
//! and warm Parchment (#fef7de / #fbf3dc), matching the Bornomala logo and cultural aesthetic.

use super::api::*;
use super::assets::{self, IconId};

static mut ABOUT_HWND: HWND = 0;

const ID_BTN_ABOUT_GITHUB: usize = 5001;
const ID_BTN_ABOUT_FEEDBACK: usize = 5002;
const ID_BTN_ABOUT_CLOSE: usize = 5003;

pub struct AboutWindow;

impl AboutWindow {
    pub fn open() {
        unsafe {
            if ABOUT_HWND != 0 && IsWindowVisible(ABOUT_HWND) != 0 {
                ShowWindow(ABOUT_HWND, SW_RESTORE);
                SetForegroundWindow(ABOUT_HWND);
                return;
            }

            let hinst = GetModuleHandleW(std::ptr::null());
            let class_name = to_wide("BornomalaAboutClass");

            let mut wc: WNDCLASSEXW = std::mem::zeroed();
            wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
            wc.lpfnWndProc = Some(about_wnd_proc);
            wc.hInstance = hinst;
            wc.hCursor = LoadCursorW(0, 32512 as *const u16);
            wc.hbrBackground = CreateSolidBrush(rgb(251, 243, 220)); // Parchment #fbf3dc
            wc.lpszClassName = class_name.as_ptr();

            RegisterClassExW(&wc);

            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            let width = 480;
            let height = 440;
            let x = (screen_w - width) / 2;
            let y = (screen_h - height) / 2;

            let title = to_wide("বর্ণমালা পরিচিতি — About Bornomala");
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
                x,
                y,
                width,
                height,
                0,
                0,
                hinst,
                std::ptr::null_mut(),
            );

            ABOUT_HWND = hwnd;
            ShowWindow(hwnd, SW_SHOW);
            SetForegroundWindow(hwnd);
        }
    }
}

unsafe extern "system" fn about_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let hinst = GetModuleHandleW(std::ptr::null());

            let font_btn = CreateFontW(
                -12, 0, 0, 0, FW_BOLD, 0, 0, 0, DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
                to_wide("Nirmala UI").as_ptr(),
            );

            // Action Buttons
            let btn_github = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(),
                to_wide("🌐 গিটহাব (GitHub)").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                24, 385, 130, 32, hwnd, ID_BTN_ABOUT_GITHUB, hinst, std::ptr::null_mut(),
            );
            SendMessageW(btn_github, WM_SETFONT, font_btn, 1);

            let btn_mail = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(),
                to_wide("📧 সাপোর্ট ও ফিডব্যাক").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                164, 385, 160, 32, hwnd, ID_BTN_ABOUT_FEEDBACK, hinst, std::ptr::null_mut(),
            );
            SendMessageW(btn_mail, WM_SETFONT, font_btn, 1);

            let btn_close = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(),
                to_wide("✕ বন্ধ করুন").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON,
                334, 385, 106, 32, hwnd, ID_BTN_ABOUT_CLOSE, hinst, std::ptr::null_mut(),
            );
            SendMessageW(btn_close, WM_SETFONT, font_btn, 1);

            0
        }

        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let color_green = rgb(6, 106, 74);
            let color_red = rgb(224, 47, 45);
            let color_parchment = rgb(251, 243, 220);
            let color_card_border = rgb(6, 106, 74);

            let mut client_rc: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut client_rc);
            let w = client_rc.right - client_rc.left;
            let h = client_rc.bottom - client_rc.top;

            let mem_dc = CreateCompatibleDC(hdc);
            let mem_bmp = CreateCompatibleBitmap(hdc, w, h);
            let _old_bmp = SelectObject(mem_dc, mem_bmp);

            // Background Fill
            let bg_brush = CreateSolidBrush(color_parchment);
            FillRect(mem_dc, &client_rc, bg_brush);
            DeleteObject(bg_brush);

            // 1. Forest Green Header Banner
            let header_brush = CreateSolidBrush(color_green);
            let header_rc = RECT { left: 0, top: 0, right: w, bottom: 74 };
            FillRect(mem_dc, &header_rc, header_brush);
            DeleteObject(header_brush);

            // Crimson Red Circle emblem behind logo
            let red_brush = CreateSolidBrush(color_red);
            let red_pen = CreatePen(PS_SOLID, 1, color_red);
            let old_brush = SelectObject(mem_dc, red_brush);
            let old_pen = SelectObject(mem_dc, red_pen);
            Ellipse(mem_dc, 18, 12, 68, 62);
            SelectObject(mem_dc, old_brush);
            SelectObject(mem_dc, old_pen);
            DeleteObject(red_brush);
            DeleteObject(red_pen);

            // Draw Bornomala Logo in header
            assets::draw_icon(mem_dc, IconId::Logo, 24, 18);

            // Header Titles
            SetBkMode(mem_dc, TRANSPARENT);
            SetTextColor(mem_dc, rgb(254, 247, 222)); // Cream font

            let title_font = CreateFontW(
                -19, 0, 0, 0, FW_BOLD, 0, 0, 0, DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
                to_wide("Nirmala UI").as_ptr(),
            );
            let old_f = SelectObject(mem_dc, title_font);
            let mut t_rect = RECT { left: 82, top: 15, right: w - 20, bottom: 42 };
            DrawTextW(mem_dc, to_wide("বর্ণমালা — Bornomala v0.1.0").as_ptr(), -1, &mut t_rect, DT_LEFT | DT_SINGLELINE);

            let sub_font = CreateFontW(
                -12, 0, 0, 0, FW_NORMAL, 0, 0, 0, DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
                to_wide("Nirmala UI").as_ptr(),
            );
            SelectObject(mem_dc, sub_font);
            let mut st_rect = RECT { left: 82, top: 42, right: w - 20, bottom: 64 };
            DrawTextW(mem_dc, to_wide("১০০% মুক্ত ও আধুনিক বাংলা টাইপিং স্যুট").as_ptr(), -1, &mut st_rect, DT_LEFT | DT_SINGLELINE);

            // 2. Tagline Card (Rounded box with Crimson Red quote)
            let border_pen = CreatePen(PS_SOLID, 1, color_card_border);
            let old_p = SelectObject(mem_dc, border_pen);
            let null_brush = GetStockObject(5 /* NULL_BRUSH */);
            let old_b = SelectObject(mem_dc, null_brush);

            RoundRect(mem_dc, 20, 86, w - 20, 134, 10, 10);

            let quote_font = CreateFontW(
                -13, 0, 0, 0, FW_BOLD, 0, 0, 0, DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
                to_wide("Nirmala UI").as_ptr(),
            );
            SelectObject(mem_dc, quote_font);
            SetTextColor(mem_dc, color_red);
            let mut q_rect = RECT { left: 30, top: 96, right: w - 30, bottom: 124 };
            DrawTextW(mem_dc, to_wide("“আপনার মাতৃভাষার জন্য কোনো লাইসেন্স কি-র প্রয়োজন নেই।”").as_ptr(), -1, &mut q_rect, DT_CENTER | DT_SINGLELINE);

            // 3. Information Card (Rounded box with details)
            RoundRect(mem_dc, 20, 144, w - 20, 365, 10, 10);
            SelectObject(mem_dc, old_b);
            SelectObject(mem_dc, old_p);
            DeleteObject(border_pen);

            // Details List
            let item_label_font = CreateFontW(
                -12, 0, 0, 0, FW_BOLD, 0, 0, 0, DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
                to_wide("Nirmala UI").as_ptr(),
            );
            let item_val_font = CreateFontW(
                -12, 0, 0, 0, FW_NORMAL, 0, 0, 0, DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
                to_wide("Nirmala UI").as_ptr(),
            );

            let draw_detail = |label: &str, val: &str, y_pos: i32| {
                SelectObject(mem_dc, item_label_font);
                SetTextColor(mem_dc, color_green);
                let mut l_rc = RECT { left: 34, top: y_pos, right: 175, bottom: y_pos + 20 };
                DrawTextW(mem_dc, to_wide(label).as_ptr(), -1, &mut l_rc, DT_LEFT | DT_SINGLELINE);

                SelectObject(mem_dc, item_val_font);
                SetTextColor(mem_dc, rgb(30, 40, 35));
                let mut v_rc = RECT { left: 180, top: y_pos, right: w - 34, bottom: y_pos + 20 };
                DrawTextW(mem_dc, to_wide(val).as_ptr(), -1, &mut v_rc, DT_LEFT | DT_SINGLELINE);
            };

            draw_detail("প্রকাশক (Publisher):", "রূপসী ওপেন ল্যাবস (Ruposhi Open Labs)", 160);
            draw_detail("ওয়েবসাইট:", "github.com/RuposhiOpenLabs/bornomala", 188);
            draw_detail("প্রধান ডেভেলপার:", "Ryan Shelby (MdSagorMunshi)", 216);
            draw_detail("লাইসেন্স:", "GNU General Public License v3.0 (GPLv3)", 244);
            draw_detail("নিরাপত্তা ও গোপনীয়তা:", "১০০% অফলাইন, জিরো কী-লগিং, জিরো ট্র্যাকিং", 272);
            draw_detail("সমর্থিত ইনপুট মোড:", "ইউনিকোড, বিজয় ANSI, ফোনেটিক, ইংরেজি", 300);
            draw_detail("দ্বিমুখী রূপান্তর:", "ইউনিকোড ⇄ বিজয় এএনএসআই কনভার্টার", 328);

            // Blit to screen
            BitBlt(hdc, 0, 0, w, h, mem_dc, 0, 0, 0x00CC0020);

            SelectObject(mem_dc, old_f);
            DeleteObject(title_font);
            DeleteObject(sub_font);
            DeleteObject(quote_font);
            DeleteObject(item_label_font);
            DeleteObject(item_val_font);
            DeleteObject(mem_bmp);
            DeleteDC(mem_dc);

            EndPaint(hwnd, &ps);
            0
        }

        WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => {
            let hdc_ctrl = wparam as HDC;
            SetBkMode(hdc_ctrl, TRANSPARENT);
            SetTextColor(hdc_ctrl, rgb(6, 106, 74));
            let parchment_brush = CreateSolidBrush(rgb(251, 243, 220));
            parchment_brush as LRESULT
        }

        WM_COMMAND => {
            let id = wparam & 0xFFFF;
            match id {
                ID_BTN_ABOUT_GITHUB => {
                    let open_op = to_wide("open");
                    let url = to_wide("https://github.com/RuposhiOpenLabs/bornomala");
                    ShellExecuteW(hwnd, open_op.as_ptr(), url.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL);
                }
                ID_BTN_ABOUT_FEEDBACK => {
                    let open_op = to_wide("open");
                    let mailto = to_wide("mailto:ryn@disr.it?subject=Bornomala%20Feedback");
                    ShellExecuteW(hwnd, open_op.as_ptr(), mailto.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL);
                }
                ID_BTN_ABOUT_CLOSE => {
                    ShowWindow(hwnd, SW_HIDE);
                    DestroyWindow(hwnd);
                    ABOUT_HWND = 0;
                }
                _ => {}
            }
            0
        }

        WM_CLOSE => {
            ShowWindow(hwnd, SW_HIDE);
            DestroyWindow(hwnd);
            ABOUT_HWND = 0;
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
