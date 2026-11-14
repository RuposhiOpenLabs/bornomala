//! Dedicated Native Windows GUI for Bornomala Converter.
//!
//! Authentic Desi Bengali Theme: Forest Green (#066a4a), Crimson Red (#e02f2d),
//! and warm Heritage Parchment (#fbf3dc), matching Bornomala's signature aesthetic.
//!
//! Provides:
//! 1. High-speed bidirectional text converter (Unicode <-> SutonnyMJ ANSI).
//! 2. Universal document converter for DOCX, PDF, and TXT preserving 100% layout and styles.

use super::api::*;
use bornomala_engine::converter::BanglaConverter;
use bornomala_engine::doc_converter::{
    convert_file, default_output_path, ConversionDirection, DocumentFormat,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

pub const CONV_WIDTH: i32 = 840;
pub const CONV_HEIGHT: i32 = 720;

static IS_STANDALONE: AtomicBool = AtomicBool::new(false);

// Child control IDs
const IDC_TAB_TEXT: usize = 110;
const IDC_TAB_DOC: usize = 111;

// Text conversion IDs
const IDC_EDIT_UNICODE: usize = 101;
const IDC_EDIT_ANSI: usize = 102;
const IDC_BTN_U2A: usize = 201;
const IDC_BTN_A2U: usize = 202;
const IDC_BTN_COPY_U: usize = 203;
const IDC_BTN_COPY_A: usize = 204;
const IDC_BTN_CLEAR: usize = 205;
const IDC_LBL_U: usize = 206;
const IDC_LBL_A: usize = 207;

// Document conversion IDs
const IDC_DOC_SRC_LBL: usize = 401;
const IDC_DOC_SRC_EDIT: usize = 402;
const IDC_DOC_SRC_BTN: usize = 403;
const IDC_DOC_DIR_LBL: usize = 404;
const IDC_DOC_RADIO_A2U: usize = 405;
const IDC_DOC_RADIO_U2A: usize = 406;
const IDC_DOC_OUT_LBL: usize = 407;
const IDC_DOC_OUT_EDIT: usize = 408;
const IDC_DOC_OUT_BTN: usize = 409;
const IDC_DOC_CONVERT_BTN: usize = 410;
const IDC_DOC_RESULT_LBL: usize = 411;
const IDC_DOC_OPEN_FOLDER: usize = 412;
const IDC_DOC_OPEN_FILE: usize = 413;
const IDC_DOC_FORMAT_BADGE: usize = 414;

// Status Bar
const IDC_LABEL_STATUS: usize = 301;

// Global Handles
static mut CONVERTER_HWND: HWND = 0;
static mut CURRENT_TAB: usize = 0; // 0 = Text, 1 = Document

// Fonts & Brushes
static mut FONT_TITLE: HFONT = 0;
static mut FONT_SUBTITLE: HFONT = 0;
static mut FONT_LABEL_BOLD: HFONT = 0;
static mut FONT_REGULAR: HFONT = 0;
static mut FONT_BTN: HFONT = 0;
static mut FONT_TEXT_BOX: HFONT = 0;
static mut BRUSH_BG: HBRUSH = 0;
static mut BRUSH_WHITE: HBRUSH = 0;

// Text HWNDs
static mut EDIT_U_HWND: HWND = 0;
static mut EDIT_A_HWND: HWND = 0;
static mut BTN_U2A_HWND: HWND = 0;
static mut BTN_A2U_HWND: HWND = 0;
static mut BTN_COPY_U_HWND: HWND = 0;
static mut BTN_COPY_A_HWND: HWND = 0;
static mut BTN_CLEAR_HWND: HWND = 0;
static mut LBL_U_HWND: HWND = 0;
static mut LBL_A_HWND: HWND = 0;

// Doc HWNDs
static mut DOC_SRC_LBL_HWND: HWND = 0;
static mut DOC_SRC_EDIT_HWND: HWND = 0;
static mut DOC_SRC_BTN_HWND: HWND = 0;
static mut DOC_DIR_LBL_HWND: HWND = 0;
static mut DOC_RADIO_A2U_HWND: HWND = 0;
static mut DOC_RADIO_U2A_HWND: HWND = 0;
static mut DOC_OUT_LBL_HWND: HWND = 0;
static mut DOC_OUT_EDIT_HWND: HWND = 0;
static mut DOC_OUT_BTN_HWND: HWND = 0;
static mut DOC_CONVERT_BTN_HWND: HWND = 0;
static mut DOC_RESULT_LBL_HWND: HWND = 0;
static mut DOC_OPEN_FOLDER_HWND: HWND = 0;
static mut DOC_OPEN_FILE_HWND: HWND = 0;
static mut DOC_FORMAT_BADGE_HWND: HWND = 0;

// Status HWND
static mut STATUS_HWND: HWND = 0;
static mut LAST_CONVERTED_OUTPUT: Option<PathBuf> = None;

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
            BRUSH_BG = CreateSolidBrush(rgb(251, 243, 220)); // Heritage Parchment #fbf3dc
            BRUSH_WHITE = CreateSolidBrush(rgb(255, 255, 255));
            wc.hbrBackground = BRUSH_BG;
            wc.lpszClassName = class_name.as_ptr();

            RegisterClassExW(&wc);

            let font_name = to_wide("Nirmala UI");
            FONT_TITLE = CreateFontW(
                -17,
                0,
                0,
                0,
                FW_BOLD,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                0,
                0,
                CLEARTYPE_QUALITY,
                0,
                font_name.as_ptr(),
            );
            FONT_SUBTITLE = CreateFontW(
                -12,
                0,
                0,
                0,
                FW_NORMAL,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                0,
                0,
                CLEARTYPE_QUALITY,
                0,
                font_name.as_ptr(),
            );
            FONT_LABEL_BOLD = CreateFontW(
                -13,
                0,
                0,
                0,
                FW_BOLD,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                0,
                0,
                CLEARTYPE_QUALITY,
                0,
                font_name.as_ptr(),
            );
            FONT_REGULAR = CreateFontW(
                -13,
                0,
                0,
                0,
                FW_NORMAL,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                0,
                0,
                CLEARTYPE_QUALITY,
                0,
                font_name.as_ptr(),
            );
            FONT_BTN = CreateFontW(
                -12,
                0,
                0,
                0,
                FW_BOLD,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                0,
                0,
                CLEARTYPE_QUALITY,
                0,
                font_name.as_ptr(),
            );
            FONT_TEXT_BOX = CreateFontW(
                -15,
                0,
                0,
                0,
                FW_NORMAL,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                0,
                0,
                CLEARTYPE_QUALITY,
                0,
                font_name.as_ptr(),
            );

            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            let x = (screen_w - CONV_WIDTH) / 2;
            let y = (screen_h - CONV_HEIGHT) / 2;

            let title = to_wide("বর্ণমালা রূপান্তরকারী — Bornomala Universal Converter");
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE,
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

            // -------------------------------------------------------------
            // Tabs Bar: Text Converter vs Document Converter
            // -------------------------------------------------------------
            let btn_tab_text = CreateWindowExW(
                0,
                btn_cls.as_ptr(),
                to_wide("📝 টেক্সট রূপান্তর (Live Text)").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                20,
                80,
                240,
                34,
                hwnd,
                IDC_TAB_TEXT,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(btn_tab_text, WM_SETFONT, FONT_BTN as WPARAM, 1);

            let btn_tab_doc = CreateWindowExW(
                0,
                btn_cls.as_ptr(),
                to_wide("📄 নথি ও ফাইল রূপান্তর (DOCX, DOC, PDF, TXT)").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                268,
                80,
                340,
                34,
                hwnd,
                IDC_TAB_DOC,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(btn_tab_doc, WM_SETFONT, FONT_BTN as WPARAM, 1);

            // -------------------------------------------------------------
            // TAB 1: TEXT CONVERSION CONTROLS
            // -------------------------------------------------------------
            let lbl_u = CreateWindowExW(
                0,
                static_cls.as_ptr(),
                to_wide("ইউনিকোড বাংলা টেক্সট (ওয়েবসাইট, অফিস, ব্রাউজার, সামাজিক মাধ্যম):").as_ptr(),
                WS_CHILD | WS_VISIBLE,
                20,
                124,
                600,
                20,
                hwnd,
                IDC_LBL_U,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(lbl_u, WM_SETFONT, FONT_LABEL_BOLD as WPARAM, 1);
            LBL_U_HWND = lbl_u;

            let btn_copy_u = CreateWindowExW(
                0,
                btn_cls.as_ptr(),
                to_wide("📋 ইউনিকোড কপি").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                670,
                120,
                134,
                26,
                hwnd,
                IDC_BTN_COPY_U,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(btn_copy_u, WM_SETFONT, FONT_BTN as WPARAM, 1);
            BTN_COPY_U_HWND = btn_copy_u;

            let edit_u = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                edit_cls.as_ptr(),
                std::ptr::null(),
                WS_CHILD
                    | WS_VISIBLE
                    | WS_VSCROLL
                    | WS_BORDER
                    | ES_MULTILINE
                    | ES_AUTOVSCROLL
                    | ES_WANTRETURN,
                20,
                148,
                784,
                154,
                hwnd,
                IDC_EDIT_UNICODE,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(edit_u, WM_SETFONT, FONT_TEXT_BOX as WPARAM, 1);
            EDIT_U_HWND = edit_u;

            // Center Actions Row
            let btn_u2a = CreateWindowExW(
                0,
                btn_cls.as_ptr(),
                to_wide("⬇ ইউনিকোড ➔ বিজয় ANSI").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON,
                20,
                310,
                230,
                34,
                hwnd,
                IDC_BTN_U2A,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(btn_u2a, WM_SETFONT, FONT_BTN as WPARAM, 1);
            BTN_U2A_HWND = btn_u2a;

            let btn_a2u = CreateWindowExW(
                0,
                btn_cls.as_ptr(),
                to_wide("⬆ বিজয় ANSI ➔ ইউনিকোড").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON,
                260,
                310,
                230,
                34,
                hwnd,
                IDC_BTN_A2U,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(btn_a2u, WM_SETFONT, FONT_BTN as WPARAM, 1);
            BTN_A2U_HWND = btn_a2u;

            let btn_clr = CreateWindowExW(
                0,
                btn_cls.as_ptr(),
                to_wide("🗑 উভয় বক্স মুছুন (Clear All)").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                584,
                310,
                220,
                34,
                hwnd,
                IDC_BTN_CLEAR,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(btn_clr, WM_SETFONT, FONT_BTN as WPARAM, 1);
            BTN_CLEAR_HWND = btn_clr;

            // Bottom ANSI Section
            let lbl_a = CreateWindowExW(
                0,
                static_cls.as_ptr(),
                to_wide("বিজয় এএনএসআই টেক্সট (SutonnyMJ ANSI — প্রেস, প্রিন্ট, ফটোশপ, ইলাস্ট্রেটর):")
                    .as_ptr(),
                WS_CHILD | WS_VISIBLE,
                20,
                354,
                600,
                20,
                hwnd,
                IDC_LBL_A,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(lbl_a, WM_SETFONT, FONT_LABEL_BOLD as WPARAM, 1);
            LBL_A_HWND = lbl_a;

            let btn_copy_a = CreateWindowExW(
                0,
                btn_cls.as_ptr(),
                to_wide("📋 ANSI কপি").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                670,
                350,
                134,
                26,
                hwnd,
                IDC_BTN_COPY_A,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(btn_copy_a, WM_SETFONT, FONT_BTN as WPARAM, 1);
            BTN_COPY_A_HWND = btn_copy_a;

            let edit_a = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                edit_cls.as_ptr(),
                std::ptr::null(),
                WS_CHILD
                    | WS_VISIBLE
                    | WS_VSCROLL
                    | WS_BORDER
                    | ES_MULTILINE
                    | ES_AUTOVSCROLL
                    | ES_WANTRETURN,
                20,
                378,
                784,
                154,
                hwnd,
                IDC_EDIT_ANSI,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(edit_a, WM_SETFONT, FONT_TEXT_BOX as WPARAM, 1);
            EDIT_A_HWND = edit_a;

            // -------------------------------------------------------------
            // TAB 2: DOCUMENT & FILE CONVERSION CONTROLS (Initially Hidden)
            // -------------------------------------------------------------
            // 1. Source Document
            let doc_src_lbl = CreateWindowExW(
                0,
                static_cls.as_ptr(),
                to_wide(
                    "ধাপ ১: যে ফাইলটি রূপান্তর করতে চান তা নির্বাচন করুন (Word .docx, .doc, PDF বা .txt):",
                )
                .as_ptr(),
                WS_CHILD,
                20,
                126,
                640,
                20,
                hwnd,
                IDC_DOC_SRC_LBL,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(doc_src_lbl, WM_SETFONT, FONT_LABEL_BOLD as WPARAM, 1);
            DOC_SRC_LBL_HWND = doc_src_lbl;

            let doc_src_edit = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                edit_cls.as_ptr(),
                std::ptr::null(),
                WS_CHILD | WS_BORDER | ES_AUTOHSCROLL,
                20,
                150,
                620,
                32,
                hwnd,
                IDC_DOC_SRC_EDIT,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(doc_src_edit, WM_SETFONT, FONT_REGULAR as WPARAM, 1);
            DOC_SRC_EDIT_HWND = doc_src_edit;

            let doc_src_btn = CreateWindowExW(
                0,
                btn_cls.as_ptr(),
                to_wide("📂 ফাইল ব্রাউজ (Browse)").as_ptr(),
                WS_CHILD | BS_PUSHBUTTON,
                648,
                149,
                156,
                34,
                hwnd,
                IDC_DOC_SRC_BTN,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(doc_src_btn, WM_SETFONT, FONT_BTN as WPARAM, 1);
            DOC_SRC_BTN_HWND = doc_src_btn;

            let doc_badge = CreateWindowExW(
                0,
                static_cls.as_ptr(),
                to_wide("ফরম্যাট: নির্বাচন করুন").as_ptr(),
                WS_CHILD,
                20,
                186,
                300,
                18,
                hwnd,
                IDC_DOC_FORMAT_BADGE,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(doc_badge, WM_SETFONT, FONT_SUBTITLE as WPARAM, 1);
            DOC_FORMAT_BADGE_HWND = doc_badge;

            // 2. Conversion Direction
            let doc_dir_lbl = CreateWindowExW(
                0,
                static_cls.as_ptr(),
                to_wide("ধাপ ২: রূপান্তরের দিক নির্ধারণ করুন:").as_ptr(),
                WS_CHILD,
                20,
                216,
                640,
                20,
                hwnd,
                IDC_DOC_DIR_LBL,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(doc_dir_lbl, WM_SETFONT, FONT_LABEL_BOLD as WPARAM, 1);
            DOC_DIR_LBL_HWND = doc_dir_lbl;

            let doc_radio_a2u = CreateWindowExW(
                0,
                btn_cls.as_ptr(),
                to_wide(
                    "বিজয় এএনএসআই (SutonnyMJ) ➔ ইউনিকোড (Unicode) [প্রস্তাবিত — পুরাতন ডক আধুনিকীকরণ]",
                )
                .as_ptr(),
                WS_CHILD | BS_AUTORADIOBUTTON,
                24,
                240,
                750,
                24,
                hwnd,
                IDC_DOC_RADIO_A2U,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(doc_radio_a2u, WM_SETFONT, FONT_REGULAR as WPARAM, 1);
            SendMessageW(doc_radio_a2u, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
            DOC_RADIO_A2U_HWND = doc_radio_a2u;

            let doc_radio_u2a = CreateWindowExW(
                0,
                btn_cls.as_ptr(),
                to_wide(
                    "ইউনিকোড (Unicode) ➔ বিজয় এএনএসআই (SutonnyMJ) [প্রেস ও প্রিন্টিং ডিজাইনের জন্য]",
                )
                .as_ptr(),
                WS_CHILD | BS_AUTORADIOBUTTON,
                24,
                268,
                750,
                24,
                hwnd,
                IDC_DOC_RADIO_U2A,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(doc_radio_u2a, WM_SETFONT, FONT_REGULAR as WPARAM, 1);
            DOC_RADIO_U2A_HWND = doc_radio_u2a;

            // 3. Destination Output File
            let doc_out_lbl = CreateWindowExW(
                0,
                static_cls.as_ptr(),
                to_wide("ধাপ ৩: রূপান্তরিত ফাইল সংরক্ষণের গন্তব্য (Output File):").as_ptr(),
                WS_CHILD,
                20,
                304,
                640,
                20,
                hwnd,
                IDC_DOC_OUT_LBL,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(doc_out_lbl, WM_SETFONT, FONT_LABEL_BOLD as WPARAM, 1);
            DOC_OUT_LBL_HWND = doc_out_lbl;

            let doc_out_edit = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                edit_cls.as_ptr(),
                std::ptr::null(),
                WS_CHILD | WS_BORDER | ES_AUTOHSCROLL,
                20,
                328,
                620,
                32,
                hwnd,
                IDC_DOC_OUT_EDIT,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(doc_out_edit, WM_SETFONT, FONT_REGULAR as WPARAM, 1);
            DOC_OUT_EDIT_HWND = doc_out_edit;

            let doc_out_btn = CreateWindowExW(
                0,
                btn_cls.as_ptr(),
                to_wide("📁 গন্তব্য ব্রাউজ").as_ptr(),
                WS_CHILD | BS_PUSHBUTTON,
                648,
                327,
                156,
                34,
                hwnd,
                IDC_DOC_OUT_BTN,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(doc_out_btn, WM_SETFONT, FONT_BTN as WPARAM, 1);
            DOC_OUT_BTN_HWND = doc_out_btn;

            // 4. Hero Convert Button
            let doc_convert_btn = CreateWindowExW(
                0,
                btn_cls.as_ptr(),
                to_wide("⚡ নথি রূপান্তর করুন (Convert Document Now)").as_ptr(),
                WS_CHILD | BS_DEFPUSHBUTTON,
                20,
                376,
                784,
                42,
                hwnd,
                IDC_DOC_CONVERT_BTN,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(doc_convert_btn, WM_SETFONT, FONT_TITLE as WPARAM, 1);
            DOC_CONVERT_BTN_HWND = doc_convert_btn;

            // 5. Result Card & Direct Action Buttons
            let doc_result_lbl = CreateWindowExW(
                0, static_cls.as_ptr(),
                to_wide("নথি নির্বাচন করে 'নথি রূপান্তর করুন' বাটনে চাপুন। মূল লেআউট, টেবিল ও ডিজাইন সম্পূর্ণ অপরিবর্তিত থাকবে।").as_ptr(),
                WS_CHILD, 20, 434, 784, 46, hwnd, IDC_DOC_RESULT_LBL, hinst, std::ptr::null_mut(),
            );
            SendMessageW(doc_result_lbl, WM_SETFONT, FONT_REGULAR as WPARAM, 1);
            DOC_RESULT_LBL_HWND = doc_result_lbl;

            let doc_open_folder = CreateWindowExW(
                0,
                btn_cls.as_ptr(),
                to_wide("📂 ফোল্ডার খুলুন (Open Folder)").as_ptr(),
                WS_CHILD | BS_PUSHBUTTON,
                20,
                486,
                220,
                34,
                hwnd,
                IDC_DOC_OPEN_FOLDER,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(doc_open_folder, WM_SETFONT, FONT_BTN as WPARAM, 1);
            EnableWindow(doc_open_folder, 0);
            DOC_OPEN_FOLDER_HWND = doc_open_folder;

            let doc_open_file = CreateWindowExW(
                0,
                btn_cls.as_ptr(),
                to_wide("📄 ফাইলটি খুলুন (Open File)").as_ptr(),
                WS_CHILD | BS_PUSHBUTTON,
                250,
                486,
                220,
                34,
                hwnd,
                IDC_DOC_OPEN_FILE,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(doc_open_file, WM_SETFONT, FONT_BTN as WPARAM, 1);
            EnableWindow(doc_open_file, 0);
            DOC_OPEN_FILE_HWND = doc_open_file;

            // -------------------------------------------------------------
            // BOTTOM STATUS BAR
            // -------------------------------------------------------------
            let status_text = to_wide(
                "প্রস্তুত (Ready) | ১০০% অফলাইন ও নিরাপদ | Ruposhi Open Labs — Lead: Ryan Shelby",
            );
            let lbl_status = CreateWindowExW(
                0,
                static_cls.as_ptr(),
                status_text.as_ptr(),
                WS_CHILD | WS_VISIBLE,
                20,
                642,
                784,
                24,
                hwnd,
                IDC_LABEL_STATUS,
                hinst,
                std::ptr::null_mut(),
            );
            SendMessageW(lbl_status, WM_SETFONT, FONT_SUBTITLE as WPARAM, 1);
            STATUS_HWND = lbl_status;

            switch_tab(0);

            0
        }

        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let color_green = rgb(6, 106, 74);
            let color_red = rgb(224, 47, 45);
            let color_parchment = rgb(251, 243, 220);
            let color_gold = rgb(245, 197, 24);

            let mut client_rc: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut client_rc);
            let w = client_rc.right - client_rc.left;
            let h = client_rc.bottom - client_rc.top;

            let mem_dc = CreateCompatibleDC(hdc);
            let mem_bmp = CreateCompatibleBitmap(hdc, w, h);
            let old_bmp = SelectObject(mem_dc, mem_bmp);

            // 1. Heritage Parchment Background
            let bg_brush = CreateSolidBrush(color_parchment);
            FillRect(mem_dc, &client_rc, bg_brush);
            DeleteObject(bg_brush);

            // 2. Bangladesh Forest Green Header Banner
            let header_brush = CreateSolidBrush(color_green);
            let header_rc = RECT {
                left: 0,
                top: 0,
                right: w,
                bottom: 68,
            };
            FillRect(mem_dc, &header_rc, header_brush);
            DeleteObject(header_brush);

            // 3. Crimson Red Emblem Circle behind "ব"
            let red_brush = CreateSolidBrush(color_red);
            let red_pen = CreatePen(PS_SOLID, 1, color_red);
            let old_brush = SelectObject(mem_dc, red_brush);
            let old_pen = SelectObject(mem_dc, red_pen);
            Ellipse(mem_dc, 20, 10, 68, 58);
            SelectObject(mem_dc, old_brush);
            SelectObject(mem_dc, old_pen);
            DeleteObject(red_brush);
            DeleteObject(red_pen);

            // "ব" glyph inside circle
            SetBkMode(mem_dc, 1);
            SetTextColor(mem_dc, rgb(255, 255, 255));
            SelectObject(mem_dc, FONT_TITLE);
            let logo_txt = to_wide("ব");
            let mut logo_rc = RECT {
                left: 20,
                top: 12,
                right: 68,
                bottom: 56,
            };
            DrawTextW(
                mem_dc,
                logo_txt.as_ptr(),
                -1,
                &mut logo_rc,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );

            // Title & Subtitle text in header
            SetTextColor(mem_dc, rgb(255, 255, 255));
            let title_txt = to_wide("বর্ণমালা রূপান্তরকারী — Bornomala Universal Converter");
            let mut title_rc = RECT {
                left: 80,
                top: 12,
                right: w - 20,
                bottom: 36,
            };
            DrawTextW(
                mem_dc,
                title_txt.as_ptr(),
                -1,
                &mut title_rc,
                DT_LEFT | DT_SINGLELINE,
            );

            SetTextColor(mem_dc, color_gold);
            SelectObject(mem_dc, FONT_SUBTITLE);
            let sub_txt = to_wide("দ্বিমুখী রূপান্তর: ইউনিকোড ⇄ বিজয় ANSI | ডকু রূপান্তর: DOCX • DOC • PDF • TXT (১০০% নিখুঁত লেআউট)");
            let mut sub_rc = RECT {
                left: 80,
                top: 38,
                right: w - 20,
                bottom: 58,
            };
            DrawTextW(
                mem_dc,
                sub_txt.as_ptr(),
                -1,
                &mut sub_rc,
                DT_LEFT | DT_SINGLELINE,
            );

            // 4. Subtle line separator above status bar
            let line_pen = CreatePen(PS_SOLID, 1, rgb(200, 220, 210));
            let old_lp = SelectObject(mem_dc, line_pen);
            MoveToEx(mem_dc, 20, 634, std::ptr::null_mut());
            LineTo(mem_dc, w - 20, 634);
            SelectObject(mem_dc, old_lp);
            DeleteObject(line_pen);

            // Copy to screen DC
            BitBlt(hdc, 0, 0, w, h, mem_dc, 0, 0, SRCCOPY);

            SelectObject(mem_dc, old_bmp);
            DeleteObject(mem_bmp);
            DeleteDC(mem_dc);
            EndPaint(hwnd, &ps);
            0
        }

        WM_CTLCOLORSTATIC => {
            let hdc_ctl = wparam as HDC;
            SetBkMode(hdc_ctl, 1); // TRANSPARENT
            SetTextColor(hdc_ctl, rgb(6, 106, 74));
            BRUSH_BG as LRESULT
        }

        WM_CTLCOLOREDIT => {
            let hdc_ctl = wparam as HDC;
            SetBkColor(hdc_ctl, rgb(255, 255, 255));
            SetTextColor(hdc_ctl, rgb(17, 24, 39));
            BRUSH_WHITE as LRESULT
        }

        WM_CTLCOLORBTN => BRUSH_BG as LRESULT,

        WM_COMMAND => {
            let id = wparam & 0xFFFF;
            match id {
                IDC_TAB_TEXT => {
                    switch_tab(0);
                    InvalidateRect(hwnd, std::ptr::null(), 1);
                }

                IDC_TAB_DOC => {
                    switch_tab(1);
                    InvalidateRect(hwnd, std::ptr::null(), 1);
                }

                IDC_BTN_U2A => {
                    let unicode_text = get_edit_text(EDIT_U_HWND);
                    let conv = BanglaConverter::new();
                    let (ansi_result, _) = conv.unicode_to_ansi(&unicode_text);
                    set_edit_text(EDIT_A_HWND, &ansi_result);

                    let status = format!(
                        "সফলভাবে ইউনিকোড থেকে বিজয় ANSI রূপান্তর সম্পন্ন ({} অক্ষর)।",
                        ansi_result.chars().count()
                    );
                    SetWindowTextW(STATUS_HWND, to_wide(&status).as_ptr());
                }

                IDC_BTN_A2U => {
                    let ansi_text = get_edit_text(EDIT_A_HWND);
                    let conv = BanglaConverter::new();
                    let unicode_result = conv.ansi_to_unicode(&ansi_text);
                    set_edit_text(EDIT_U_HWND, &unicode_result);

                    let status = format!(
                        "সফলভাবে বিজয় ANSI থেকে ইউনিকোড রূপান্তর সম্পন্ন ({} অক্ষর)।",
                        unicode_result.chars().count()
                    );
                    SetWindowTextW(STATUS_HWND, to_wide(&status).as_ptr());
                }

                IDC_BTN_COPY_U => {
                    let text = get_edit_text(EDIT_U_HWND);
                    set_clipboard_text(hwnd, &text);
                    SetWindowTextW(
                        STATUS_HWND,
                        to_wide("ইউনিকোড টেক্সট ক্লিপবোর্ডে কপি করা হয়েছে।").as_ptr(),
                    );
                }

                IDC_BTN_COPY_A => {
                    let text = get_edit_text(EDIT_A_HWND);
                    set_clipboard_text(hwnd, &text);
                    SetWindowTextW(
                        STATUS_HWND,
                        to_wide("ANSI টেক্সট ক্লিপবোর্ডে কপি করা হয়েছে।").as_ptr(),
                    );
                }

                IDC_BTN_CLEAR => {
                    let empty = to_wide("");
                    SetWindowTextW(EDIT_U_HWND, empty.as_ptr());
                    SetWindowTextW(EDIT_A_HWND, empty.as_ptr());
                    SetWindowTextW(
                        STATUS_HWND,
                        to_wide("উভয় টেক্সট বক্স মুছে ফেলা হয়েছে।").as_ptr(),
                    );
                }

                IDC_DOC_SRC_BTN => {
                    if let Some(path) = prompt_open_document(hwnd) {
                        set_edit_text(DOC_SRC_EDIT_HWND, &path.to_string_lossy());

                        let fmt = DocumentFormat::from_path(&path);
                        let badge_text = format!("ফরম্যাট: {}", fmt.display_name());
                        SetWindowTextW(DOC_FORMAT_BADGE_HWND, to_wide(&badge_text).as_ptr());

                        // Auto-suggest target output path
                        let is_a2u =
                            SendMessageW(DOC_RADIO_A2U_HWND, BM_GETCHECK, 0, 0) == BST_CHECKED;
                        let dir = if is_a2u {
                            ConversionDirection::AnsiToUnicode
                        } else {
                            ConversionDirection::UnicodeToAnsi
                        };
                        let out_path = default_output_path(&path, dir);
                        set_edit_text(DOC_OUT_EDIT_HWND, &out_path.to_string_lossy());

                        let msg = format!(
                            "ফাইল নির্বাচিত: {}",
                            path.file_name().unwrap_or_default().to_string_lossy()
                        );
                        SetWindowTextW(STATUS_HWND, to_wide(&msg).as_ptr());
                    }
                }

                IDC_DOC_OUT_BTN => {
                    let curr_src = get_edit_text(DOC_SRC_EDIT_HWND);
                    let filter_ext = Path::new(&curr_src)
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("docx");
                    if let Some(path) = prompt_save_document(hwnd, filter_ext) {
                        set_edit_text(DOC_OUT_EDIT_HWND, &path.to_string_lossy());
                    }
                }

                IDC_DOC_CONVERT_BTN => {
                    let src_str = get_edit_text(DOC_SRC_EDIT_HWND);
                    let src_path = PathBuf::from(src_str.trim());
                    if !src_path.exists() {
                        let msg = "অনুগ্রহ করে প্রথমে রূপান্তরের জন্য একটি বৈধ ফাইল (.docx, .pdf, .txt) নির্বাচন করুন।";
                        SetWindowTextW(DOC_RESULT_LBL_HWND, to_wide(msg).as_ptr());
                        SetWindowTextW(STATUS_HWND, to_wide("ত্রুটি: উৎস ফাইল পাওয়া যায়নি।").as_ptr());
                        return 0;
                    }

                    let out_str = get_edit_text(DOC_OUT_EDIT_HWND);
                    let out_path = if out_str.trim().is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(out_str.trim()))
                    };

                    let is_a2u = SendMessageW(DOC_RADIO_A2U_HWND, BM_GETCHECK, 0, 0) == BST_CHECKED;
                    let direction = if is_a2u {
                        ConversionDirection::AnsiToUnicode
                    } else {
                        ConversionDirection::UnicodeToAnsi
                    };

                    SetWindowTextW(
                        STATUS_HWND,
                        to_wide("নথি রূপান্তর প্রক্রিয়াধীন, অনুগ্রহ করে অপেক্ষা করুন...").as_ptr(),
                    );

                    match convert_file(&src_path, out_path.as_deref(), direction) {
                        Ok(rep) => {
                            LAST_CONVERTED_OUTPUT = Some(rep.output_path.clone());
                            let result_text = format!(
                                "✓ সফলভাবে রূপান্তর সম্পন্ন হয়েছে!\nফাইল: {}\nউপাদান: {} টি | রূপান্তর: {} টি অক্ষর | সময়: {} ms\nলেআউট, টেবিল, ফন্ট সাইজ ও মার্জিন ১০০% অক্ষত রয়েছে।",
                                rep.output_path.file_name().unwrap_or_default().to_string_lossy(),
                                rep.items_processed,
                                rep.characters_converted,
                                rep.duration_ms
                            );
                            SetWindowTextW(DOC_RESULT_LBL_HWND, to_wide(&result_text).as_ptr());
                            SetWindowTextW(
                                STATUS_HWND,
                                to_wide("নথি সফলভাবে সংরক্ষিত হয়েছে!").as_ptr(),
                            );
                            EnableWindow(DOC_OPEN_FOLDER_HWND, 1);
                            EnableWindow(DOC_OPEN_FILE_HWND, 1);
                        }
                        Err(e) => {
                            let err_text = format!("✕ রূপান্তর ত্রুটি: {}", e);
                            SetWindowTextW(DOC_RESULT_LBL_HWND, to_wide(&err_text).as_ptr());
                            SetWindowTextW(STATUS_HWND, to_wide("রূপান্তর ব্যর্থ হয়েছে।").as_ptr());
                        }
                    }
                }

                IDC_DOC_OPEN_FOLDER => {
                    if let Some(ref out_path) = LAST_CONVERTED_OUTPUT {
                        if let Some(parent) = out_path.parent() {
                            let parent_str = parent.to_string_lossy();
                            ShellExecuteW(
                                0,
                                to_wide("open").as_ptr(),
                                to_wide(&parent_str).as_ptr(),
                                std::ptr::null(),
                                std::ptr::null(),
                                SW_SHOWNORMAL,
                            );
                        }
                    }
                }

                IDC_DOC_OPEN_FILE => {
                    if let Some(ref out_path) = LAST_CONVERTED_OUTPUT {
                        let path_str = out_path.to_string_lossy();
                        ShellExecuteW(
                            0,
                            to_wide("open").as_ptr(),
                            to_wide(&path_str).as_ptr(),
                            std::ptr::null(),
                            std::ptr::null(),
                            SW_SHOWNORMAL,
                        );
                    }
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
            if IS_STANDALONE.load(Ordering::SeqCst) {
                PostQuitMessage(0);
            }
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn switch_tab(tab: usize) {
    CURRENT_TAB = tab;

    let show_text = if tab == 0 { SW_SHOW } else { SW_HIDE };
    let show_doc = if tab == 1 { SW_SHOW } else { SW_HIDE };

    // Text controls
    ShowWindow(LBL_U_HWND, show_text);
    ShowWindow(BTN_COPY_U_HWND, show_text);
    ShowWindow(EDIT_U_HWND, show_text);
    ShowWindow(BTN_U2A_HWND, show_text);
    ShowWindow(BTN_A2U_HWND, show_text);
    ShowWindow(BTN_CLEAR_HWND, show_text);
    ShowWindow(LBL_A_HWND, show_text);
    ShowWindow(BTN_COPY_A_HWND, show_text);
    ShowWindow(EDIT_A_HWND, show_text);

    // Doc controls
    ShowWindow(DOC_SRC_LBL_HWND, show_doc);
    ShowWindow(DOC_SRC_EDIT_HWND, show_doc);
    ShowWindow(DOC_SRC_BTN_HWND, show_doc);
    ShowWindow(DOC_FORMAT_BADGE_HWND, show_doc);
    ShowWindow(DOC_DIR_LBL_HWND, show_doc);
    ShowWindow(DOC_RADIO_A2U_HWND, show_doc);
    ShowWindow(DOC_RADIO_U2A_HWND, show_doc);
    ShowWindow(DOC_OUT_LBL_HWND, show_doc);
    ShowWindow(DOC_OUT_EDIT_HWND, show_doc);
    ShowWindow(DOC_OUT_BTN_HWND, show_doc);
    ShowWindow(DOC_CONVERT_BTN_HWND, show_doc);
    ShowWindow(DOC_RESULT_LBL_HWND, show_doc);
    ShowWindow(DOC_OPEN_FOLDER_HWND, show_doc);
    ShowWindow(DOC_OPEN_FILE_HWND, show_doc);
}

unsafe fn prompt_open_document(owner: HWND) -> Option<PathBuf> {
    let mut file_buf = vec![0u16; 1024];

    let filter = to_wide_filter(&[
        (
            "সমর্থিত সকল নথি (*.docx, *.doc, *.pdf, *.txt)",
            "*.docx;*.doc;*.pdf;*.txt",
        ),
        ("Microsoft Word নথি (*.docx)", "*.docx"),
        ("Word ৯৭-২০০৩ নথি (*.doc)", "*.doc"),
        ("PDF দলিল (*.pdf)", "*.pdf"),
        ("টেক্সট ফাইল (*.txt)", "*.txt"),
        ("সকল ফাইল (*.*)", "*.*"),
    ]);

    let title = to_wide("রূপান্তরের জন্য নথি নির্বাচন করুন");

    let mut ofn: OPENFILENAMEW = std::mem::zeroed();
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = owner;
    ofn.lpstrFilter = filter.as_ptr();
    ofn.lpstrFile = file_buf.as_mut_ptr();
    ofn.nMaxFile = file_buf.len() as u32;
    ofn.lpstrTitle = title.as_ptr();
    ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;

    if GetOpenFileNameW(&mut ofn) != 0 {
        let len = file_buf
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(file_buf.len());
        let path_str = String::from_utf16_lossy(&file_buf[..len]);
        Some(PathBuf::from(path_str))
    } else {
        None
    }
}

unsafe fn prompt_save_document(owner: HWND, default_ext: &str) -> Option<PathBuf> {
    let mut file_buf = vec![0u16; 1024];

    let filter = match default_ext {
        "doc" => to_wide_filter(&[
            ("Word ৯৭-২০০৩ নথি (*.doc)", "*.doc"),
            ("Microsoft Word নথি (*.docx)", "*.docx"),
            ("সকল ফাইল (*.*)", "*.*"),
        ]),
        "pdf" => to_wide_filter(&[("PDF দলিল (*.pdf)", "*.pdf"), ("সকল ফাইল (*.*)", "*.*")]),
        "txt" => to_wide_filter(&[("টেক্সট ফাইল (*.txt)", "*.txt"), ("সকল ফাইল (*.*)", "*.*")]),
        _ => to_wide_filter(&[
            ("Microsoft Word নথি (*.docx)", "*.docx"),
            ("Word ৯৭-২০০৩ নথি (*.doc)", "*.doc"),
            ("সকল ফাইল (*.*)", "*.*"),
        ]),
    };

    let title = to_wide("রূপান্তরিত ফাইল সংরক্ষণের অবস্থান নির্ধারণ করুন");
    let def_ext = to_wide(default_ext);

    let mut ofn: OPENFILENAMEW = std::mem::zeroed();
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = owner;
    ofn.lpstrFilter = filter.as_ptr();
    ofn.lpstrFile = file_buf.as_mut_ptr();
    ofn.nMaxFile = file_buf.len() as u32;
    ofn.lpstrTitle = title.as_ptr();
    ofn.lpstrDefExt = def_ext.as_ptr();
    ofn.Flags = OFN_PATHMUSTEXIST | OFN_OVERWRITEPROMPT | OFN_NOCHANGEDIR;

    if GetSaveFileNameW(&mut ofn) != 0 {
        let len = file_buf
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(file_buf.len());
        let path_str = String::from_utf16_lossy(&file_buf[..len]);
        Some(PathBuf::from(path_str))
    } else {
        None
    }
}

fn to_wide_filter(filters: &[(&str, &str)]) -> Vec<u16> {
    let mut out = Vec::new();
    for (desc, pattern) in filters {
        out.extend(desc.encode_utf16());
        out.push(0);
        out.extend(pattern.encode_utf16());
        out.push(0);
    }
    out.push(0);
    out
}

unsafe fn get_edit_text(edit_hwnd: HWND) -> String {
    let len = GetWindowTextLengthW(edit_hwnd);
    if len <= 0 {
        return String::new();
    }
    let mut buf: Vec<u16> = vec![0; (len + 1) as usize];
    GetWindowTextW(edit_hwnd, buf.as_mut_ptr(), len + 1);
    buf.pop();
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
