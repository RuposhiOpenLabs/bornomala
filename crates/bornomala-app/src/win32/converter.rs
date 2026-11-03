//! Dedicated Native Windows GUI for Bornomala Converter.
//! Provides high-speed, 100% offline bidirectional conversion between
//! Unicode Bangla and SutonnyMJ ANSI, with clipboard integration and Nirmala UI font.

use super::api::*;
use bornomala_engine::converter::BanglaConverter;
use std::sync::atomic::{AtomicBool, Ordering};

pub const CONV_WIDTH: i32 = 740;
pub const CONV_HEIGHT: i32 = 620;

static IS_STANDALONE: AtomicBool = AtomicBool::new(false);

// Child control IDs
const IDC_EDIT_UNICODE: usize = 101;
const IDC_EDIT_ANSI: usize = 102;
const IDC_BTN_U2A: usize = 201;
const IDC_BTN_A2U: usize = 202;
const IDC_BTN_COPY_U: usize = 203;
const IDC_BTN_COPY_A: usize = 204;
const IDC_BTN_CLEAR: usize = 205;
const IDC_LABEL_STATUS: usize = 301;

static mut CONVERTER_HWND: HWND = 0;
static mut EDIT_U_HWND: HWND = 0;
static mut EDIT_A_HWND: HWND = 0;
static mut STATUS_HWND: HWND = 0;
static mut CONV_FONT: HFONT = 0;

pub struct ConverterWindow;

impl ConverterWindow {
    pub fn open_standalone() {
        IS_STANDALONE.store(true, Ordering::SeqCst);
        Self::open();
    }

    pub fn open() {
        unsafe {
            if CONVERTER_HWND != 0 {
                ShowWindow(CONVERTER_HWND, SW_RESTORE);
                SetForegroundWindow(CONVERTER_HWND);
                return;
            }

            let hinst = GetModuleHandleW(std::ptr::null());
            let class_name = to_wide("BornomalaConverterClass");
            let mut wc: WNDCLASSEXW = std::mem::zeroed();
            wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
            wc.lpfnWndProc = Some(converter_wnd_proc);
            wc.hInstance = hinst;
            wc.hCursor = LoadCursorW(0, 32512 as *const u16);
            wc.hbrBackground = CreateSolidBrush(rgb(248, 250, 252)); // Light slate
            wc.lpszClassName = class_name.as_ptr();

            RegisterClassExW(&wc);

            let font_name = to_wide("Nirmala UI");
            CONV_FONT = CreateFontW(
                -13, 0, 0, 0, FW_NORMAL, 0, 0, 0,
                DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
                font_name.as_ptr(),
            );

            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            let x = (screen_w - CONV_WIDTH) / 2;
            let y = (screen_h - CONV_HEIGHT) / 2;

            let title = to_wide("বর্ণমালা রূপান্তরকারী — Bornomala Bangla Converter");
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                x,
                y,
                CONV_WIDTH,
                CONV_HEIGHT,
                0,
                0,
                hinst,
                std::ptr::null_mut(),
            );

            CONVERTER_HWND = hwnd;
            ShowWindow(hwnd, SW_SHOW);
            SetForegroundWindow(hwnd);
        }
    }
}

unsafe extern "system" fn converter_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let hinst = GetModuleHandleW(std::ptr::null());
            let static_cls = to_wide("STATIC");
            let edit_cls = to_wide("EDIT");
            let btn_cls = to_wide("BUTTON");

            // 1. Top Section (Unicode)
            let lbl_u_text = to_wide("ইউনিকোড বাংলা টেক্সট (ওয়েবসাইট, ফেসবুক, অফিস ইত্যাদি):");
            let lbl_u = CreateWindowExW(
                0, static_cls.as_ptr(), lbl_u_text.as_ptr(),
                WS_CHILD | WS_VISIBLE,
                16, 12, 700, 20,
                hwnd, 0, hinst, std::ptr::null_mut(),
            );
            SendMessageW(lbl_u, WM_SETFONT, CONV_FONT as WPARAM, 1);

            let edit_u = CreateWindowExW(
                WS_EX_CLIENTEDGE, edit_cls.as_ptr(), std::ptr::null(),
                WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_BORDER | ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN,
                16, 36, 692, 180,
                hwnd, IDC_EDIT_UNICODE, hinst, std::ptr::null_mut(),
            );
            SendMessageW(edit_u, WM_SETFONT, CONV_FONT as WPARAM, 1);
            EDIT_U_HWND = edit_u;

            // 2. Action Buttons Row
            let btn_u2a_txt = to_wide("⬇ ইউনিকোড ➔ বিজয় ANSI");
            let btn_u2a = CreateWindowExW(
                0, btn_cls.as_ptr(), btn_u2a_txt.as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                16, 226, 180, 32,
                hwnd, IDC_BTN_U2A, hinst, std::ptr::null_mut(),
            );
            SendMessageW(btn_u2a, WM_SETFONT, CONV_FONT as WPARAM, 1);

            let btn_a2u_txt = to_wide("⬆ বিজয় ANSI ➔ ইউনিকোড");
            let btn_a2u = CreateWindowExW(
                0, btn_cls.as_ptr(), btn_a2u_txt.as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                204, 226, 180, 32,
                hwnd, IDC_BTN_A2U, hinst, std::ptr::null_mut(),
            );
            SendMessageW(btn_a2u, WM_SETFONT, CONV_FONT as WPARAM, 1);

            let btn_copy_u_txt = to_wide("📋 ইউনিকোড কপি");
            let btn_copy_u = CreateWindowExW(
                0, btn_cls.as_ptr(), btn_copy_u_txt.as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                392, 226, 100, 32,
                hwnd, IDC_BTN_COPY_U, hinst, std::ptr::null_mut(),
            );
            SendMessageW(btn_copy_u, WM_SETFONT, CONV_FONT as WPARAM, 1);

            let btn_copy_a_txt = to_wide("📋 ANSI কপি");
            let btn_copy_a = CreateWindowExW(
                0, btn_cls.as_ptr(), btn_copy_a_txt.as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
500, 226, 100, 32,
                hwnd, IDC_BTN_COPY_A, hinst, std::ptr::null_mut(),
            );
            SendMessageW(btn_copy_a, WM_SETFONT, CONV_FONT as WPARAM, 1);

            let btn_clr_txt = to_wide("🗑 মুছুন");
            let btn_clr = CreateWindowExW(
                0, btn_cls.as_ptr(), btn_clr_txt.as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                608, 226, 100, 32,
                hwnd, IDC_BTN_CLEAR, hinst, std::ptr::null_mut(),
            );
            SendMessageW(btn_clr, WM_SETFONT, CONV_FONT as WPARAM, 1);

            // 3. Bottom Section (ANSI)
            let lbl_a_text = to_wide("বিজয় এএনএসআই টেক্সট (SutonnyMJ ANSI — প্রেস, প্রিন্ট, ফটোশপ, ইলাস্ট্রেটর ইত্যাদি):");
            let lbl_a = CreateWindowExW(
                0, static_cls.as_ptr(), lbl_a_text.as_ptr(),
                WS_CHILD | WS_VISIBLE,
                16, 270, 700, 20,
                hwnd, 0, hinst, std::ptr::null_mut(),
            );
            SendMessageW(lbl_a, WM_SETFONT, CONV_FONT as WPARAM, 1);

            let edit_a = CreateWindowExW(
                WS_EX_CLIENTEDGE, edit_cls.as_ptr(), std::ptr::null(),
                WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_BORDER | ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN,
                16, 294, 692, 180,
                hwnd, IDC_EDIT_ANSI, hinst, std::ptr::null_mut(),
            );
            SendMessageW(edit_a, WM_SETFONT, CONV_FONT as WPARAM, 1);
            EDIT_A_HWND = edit_a;

            // 4. Status Bar
            let status_text = to_wide("প্রস্তুত (Ready) | Ruposhi Open Labs — https://github.com/RuposhiOpenLabs | Dev: Ryan Shelby");
            let lbl_status = CreateWindowExW(
                0, static_cls.as_ptr(), status_text.as_ptr(),
                WS_CHILD | WS_VISIBLE,
                16, 488, 700, 24,
                hwnd, IDC_LABEL_STATUS, hinst, std::ptr::null_mut(),
            );
            SendMessageW(lbl_status, WM_SETFONT, CONV_FONT as WPARAM, 1);
            STATUS_HWND = lbl_status;

            0
        }

        WM_COMMAND => {
            let id = wparam & 0xFFFF;
            match id {
                IDC_BTN_U2A => {
                    // Convert Unicode -> ANSI
                    let unicode_text = get_edit_text(EDIT_U_HWND);
                    let conv = BanglaConverter::new();
                    let (ansi_result, _unmapped) = conv.unicode_to_ansi(&unicode_text);
                    set_edit_text(EDIT_A_HWND, &ansi_result);

                    let status = format!("সফলভাবে ইউনিকোড থেকে ANSI রূপান্তর সম্পন্ন ({} অক্ষর)।", ansi_result.chars().count());
                    let w_status = to_wide(&status);
                    SetWindowTextW(STATUS_HWND, w_status.as_ptr());
                }

                IDC_BTN_A2U => {
                    // Convert ANSI -> Unicode
                    let ansi_text = get_edit_text(EDIT_A_HWND);
                    let conv = BanglaConverter::new();
                    let unicode_result = conv.ansi_to_unicode(&ansi_text);
                    set_edit_text(EDIT_U_HWND, &unicode_result);

                    let status = format!("সফলভাবে ANSI থেকে ইউনিকোড রূপান্তর সম্পন্ন ({} অক্ষর)।", unicode_result.chars().count());
                    let w_status = to_wide(&status);
                    SetWindowTextW(STATUS_HWND, w_status.as_ptr());
                }

                IDC_BTN_COPY_U => {
                    let text = get_edit_text(EDIT_U_HWND);
                    set_clipboard_text(hwnd, &text);
                    let w_status = to_wide("ইউনিকোড টেক্সট ক্লিপবোর্ডে কপি করা হয়েছে।");
                    SetWindowTextW(STATUS_HWND, w_status.as_ptr());
                }

                IDC_BTN_COPY_A => {
                    let text = get_edit_text(EDIT_A_HWND);
                    set_clipboard_text(hwnd, &text);
                    let w_status = to_wide("ANSI টেক্সট ক্লিপবোর্ডে কপি করা হয়েছে।");
                    SetWindowTextW(STATUS_HWND, w_status.as_ptr());
                }

                IDC_BTN_CLEAR => {
                    let empty = to_wide("");
                    SetWindowTextW(EDIT_U_HWND, empty.as_ptr());
                    SetWindowTextW(EDIT_A_HWND, empty.as_ptr());
                    let w_status = to_wide("উভয় টেক্সট বক্স মুছে ফেলা হয়েছে।");
                    SetWindowTextW(STATUS_HWND, w_status.as_ptr());
                }

                _ => {}
            }
            0
        }

        WM_CLOSE => {
            if IS_STANDALONE.load(Ordering::SeqCst) {
                DestroyWindow(hwnd);
            } else {
                ShowWindow(hwnd, SW_HIDE);
            }
            0
        }

        WM_DESTROY => {
            CONVERTER_HWND = 0;
            EDIT_U_HWND = 0;
            EDIT_A_HWND = 0;
            STATUS_HWND = 0;
            if IS_STANDALONE.load(Ordering::SeqCst) {
                PostQuitMessage(0);
            }
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn get_edit_text(edit_hwnd: HWND) -> String {
    let len = GetWindowTextLengthW(edit_hwnd);
    if len <= 0 {
        return String::new();
    }
    let mut buf: Vec<u16> = vec![0; (len + 1) as usize];
    GetWindowTextW(edit_hwnd, buf.as_mut_ptr(), len + 1);
    buf.pop(); // Remove null terminator
    String::from_utf16_lossy(&buf)
}

unsafe fn set_edit_text(edit_hwnd: HWND, text: &str) {
    let wide = to_wide(text);
    SetWindowTextW(edit_hwnd, wide.as_ptr());
}

unsafe fn set_clipboard_text(owner: HWND, text: &str) {
    let wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    let bytes = wide.len() * 2;

    let hmem = GlobalAlloc(GMEM_MOVEABLE, bytes);
    if hmem == 0 {
        return;
    }

    let ptr = GlobalLock(hmem);
    if !ptr.is_null() {
        std::ptr::copy_nonoverlapping(wide.as_ptr() as *const u8, ptr as *mut u8, bytes);
        GlobalUnlock(hmem);

        if OpenClipboard(owner) != 0 {
            EmptyClipboard();
            SetClipboardData(CF_UNICODETEXT, hmem);
            CloseClipboard();
        }
    }
}
