//! Floating Candidate & Live Suggestion Window for Bornomala Phonetic Mode.
//!
//! Anchored just below the caret in the active target window, showing Roman input query
//! and ranked Bangla suggestions with index badges and active selection highlight.
//! Styled with the authentic Bangladesh flag green and red design language (#006a4e, #f42a41, #fbf3dc).
//! Uses WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW so it never steals focus.

use super::api::*;
use std::sync::Mutex;

const WINDOW_CLASS_NAME: &str = "BornomalaCandidateWindow";
const CANDIDATE_ITEM_HEIGHT: i32 = 28;
const HEADER_HEIGHT: i32 = 24;
const WINDOW_WIDTH: i32 = 240;

static mut CANDIDATE_HWND: HWND = 0;
static CANDIDATE_STATE: Mutex<CandidateState> = Mutex::new(CandidateState::new());

struct CandidateState {
    roman_query: String,
    candidates: Vec<String>,
    selected_idx: usize,
    font_bengali: HFONT,
    font_roman: HFONT,
    brush_bg: HBRUSH,
    brush_header: HBRUSH,
    brush_selected: HBRUSH,
    brush_border: HBRUSH,
}

impl CandidateState {
    const fn new() -> Self {
        Self {
            roman_query: String::new(),
            candidates: Vec::new(),
            selected_idx: 0,
            font_bengali: 0,
            font_roman: 0,
            brush_bg: 0,
            brush_header: 0,
            brush_selected: 0,
            brush_border: 0,
        }
    }
}

pub struct CandidateWindow;

impl CandidateWindow {
    pub fn init(hinstance: HINSTANCE) {
        unsafe {
            let class_name_w = to_wide(WINDOW_CLASS_NAME);

            let wnd_class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(candidate_window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: 0,
                hCursor: LoadCursorW(0, 32512 as *const u16),
                hbrBackground: 0,
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name_w.as_ptr(),
                hIconSm: 0,
            };

            RegisterClassExW(&wnd_class);

            CANDIDATE_HWND = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                class_name_w.as_ptr(),
                class_name_w.as_ptr(),
                WS_POPUP,
                0,
                0,
                WINDOW_WIDTH,
                100,
                0,
                0,
                hinstance,
                std::ptr::null_mut(),
            );

            // Initialize GDI resources
            if let Ok(mut state) = CANDIDATE_STATE.lock() {
                let font_name_bn = to_wide("Kalpurush");
                let font_name_fallback = to_wide("Segoe UI");

                let mut hfont = CreateFontW(
                    -16,
                    0,
                    0,
                    0,
                    600,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    4,
                    0,
                    font_name_bn.as_ptr(),
                );
                if hfont == 0 {
                    hfont = CreateFontW(
                        -15,
                        0,
                        0,
                        0,
                        600,
                        0,
                        0,
                        0,
                        1,
                        0,
                        0,
                        4,
                        0,
                        font_name_fallback.as_ptr(),
                    );
                }
                state.font_bengali = hfont;

                let font_name_roman = to_wide("Segoe UI");
                state.font_roman = CreateFontW(
                    -13,
                    0,
                    0,
                    0,
                    600,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    4,
                    0,
                    font_name_roman.as_ptr(),
                );

                // Palette: Cream #fbf3dc, Deep Green #006a4e, Vivid Red #f42a41
                state.brush_bg = CreateSolidBrush(rgb(251, 243, 220));
                state.brush_header = CreateSolidBrush(rgb(0, 106, 78));
                state.brush_selected = CreateSolidBrush(rgb(244, 42, 65));
                state.brush_border = CreateSolidBrush(rgb(0, 106, 78));
            }
        }
    }

    /// Update candidate list and show anchored below active caret.
    pub fn update_and_show(roman: &str, candidates: &[String], selected_idx: usize) {
        unsafe {
            if CANDIDATE_HWND == 0 || candidates.is_empty() {
                Self::hide();
                return;
            }

            let display_count = candidates.len().min(9);
            let window_height = HEADER_HEIGHT + (display_count as i32 * CANDIDATE_ITEM_HEIGHT) + 8;

            {
                if let Ok(mut state) = CANDIDATE_STATE.lock() {
                    state.roman_query = roman.to_string();
                    state.candidates = candidates.to_vec();
                    state.selected_idx = selected_idx;
                }
            }

            // Determine caret anchor point
            let (anchor_x, anchor_y) = Self::get_caret_screen_pos(window_height);

            SetWindowPos(
                CANDIDATE_HWND,
                HWND_TOPMOST,
                anchor_x,
                anchor_y,
                WINDOW_WIDTH,
                window_height,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );

            InvalidateRect(CANDIDATE_HWND, std::ptr::null(), TRUE);
        }
    }

    pub fn hide() {
        unsafe {
            if CANDIDATE_HWND != 0 {
                ShowWindow(CANDIDATE_HWND, SW_HIDE);
            }
        }
    }

    #[allow(dead_code)]
    pub fn is_visible() -> bool {
        unsafe {
            if CANDIDATE_HWND != 0 {
                IsWindowVisible(CANDIDATE_HWND) != 0
            } else {
                false
            }
        }
    }

    /// Calculate screen anchor coordinates for caret, clamped to active monitor.
    unsafe fn get_caret_screen_pos(window_height: i32) -> (i32, i32) {
        let mut caret_pt = POINT { x: 0, y: 0 };
        let mut found_caret = false;

        let hwnd_fg = GetForegroundWindow();
        if hwnd_fg != 0 {
            let thread_id = GetWindowThreadProcessId(hwnd_fg, std::ptr::null_mut());
            let current_thread = GetCurrentThreadId();

            if thread_id != current_thread {
                AttachThreadInput(current_thread, thread_id, TRUE);
            }

            let mut gui_info: GUITHREADINFO = std::mem::zeroed();
            gui_info.cbSize = std::mem::size_of::<GUITHREADINFO>() as u32;

            if GetGUIThreadInfo(thread_id, &mut gui_info) != 0 && gui_info.hwndCaret != 0 {
                caret_pt.x = gui_info.rcCaret.left;
                caret_pt.y = gui_info.rcCaret.bottom + 4;
                ClientToScreen(gui_info.hwndCaret, &mut caret_pt);
                found_caret = true;
            }

            if thread_id != current_thread {
                AttachThreadInput(current_thread, thread_id, FALSE);
            }
        }

        // Fallback to cursor position if caret rect is unavailable
        if !found_caret {
            GetCursorPos(&mut caret_pt);
            caret_pt.y += 18;
        }

        // Clamp to monitor work area
        let hmon = MonitorFromPoint(caret_pt, MONITOR_DEFAULTTONEAREST);
        let mut mi: MONITORINFO = std::mem::zeroed();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

        let (mut final_x, mut final_y) = (caret_pt.x, caret_pt.y);

        if GetMonitorInfoW(hmon, &mut mi) != 0 {
            let work_left = mi.rcWork.left;
            let work_top = mi.rcWork.top;
            let work_right = mi.rcWork.right;
            let work_bottom = mi.rcWork.bottom;

            // Flip above caret if overflowing bottom
            if final_y + window_height > work_bottom {
                final_y = (caret_pt.y - window_height - 24).max(work_top);
            }

            // Clamp horizontal
            if final_x + WINDOW_WIDTH > work_right {
                final_x = work_right - WINDOW_WIDTH - 8;
            }
            if final_x < work_left {
                final_x = work_left + 8;
            }
        }

        (final_x, final_y)
    }
}

unsafe extern "system" fn candidate_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut rect: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut rect);

            if let Ok(state) = CANDIDATE_STATE.lock() {
                // Double buffering
                let mem_dc = CreateCompatibleDC(hdc);
                let mem_bm = CreateCompatibleBitmap(hdc, rect.right, rect.bottom);
                let old_bm = SelectObject(mem_dc, mem_bm);

                // 1. Background fill (Cream #fbf3dc)
                FillRect(mem_dc, &rect, state.brush_bg);

                // 2. Outer Border (Deep Green #006a4e)
                FrameRect(mem_dc, &rect, state.brush_border);

                // 3. Header: Roman Query Bar (#006a4e)
                let header_rc = RECT {
                    left: 2,
                    top: 2,
                    right: rect.right - 2,
                    bottom: HEADER_HEIGHT,
                };
                FillRect(mem_dc, &header_rc, state.brush_header);

                SetBkMode(mem_dc, TRANSPARENT);
                SetTextColor(mem_dc, rgb(255, 255, 255));
                let old_font = SelectObject(mem_dc, state.font_roman);

                let header_label = format!("  বর্ণমালা  •  [{}]", state.roman_query);
                let wide_header = to_wide(&header_label);
                let mut text_rc = header_rc;
                text_rc.left += 6;
                DrawTextW(
                    mem_dc,
                    wide_header.as_ptr(),
                    -1,
                    &mut text_rc,
                    DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                );

                // 4. Candidate Items List
                SelectObject(mem_dc, state.font_bengali);
                let display_count = state.candidates.len().min(9);

                for i in 0..display_count {
                    let top = HEADER_HEIGHT + 4 + (i as i32 * CANDIDATE_ITEM_HEIGHT);
                    let item_rc = RECT {
                        left: 4,
                        top,
                        right: rect.right - 4,
                        bottom: top + CANDIDATE_ITEM_HEIGHT - 2,
                    };

                    let is_selected = i == state.selected_idx;

                    if is_selected {
                        // Highlight Pill: Flag Red (#f42a41) with rounded look
                        FillRect(mem_dc, &item_rc, state.brush_selected);
                        SetTextColor(mem_dc, rgb(255, 255, 255));
                    } else {
                        SetTextColor(mem_dc, rgb(0, 77, 56));
                    }

                    // Badge number [1], [2]...
                    let candidate_text = format!(" {}. {}", i + 1, state.candidates[i]);
                    let wide_cand = to_wide(&candidate_text);
                    let mut cand_rc = item_rc;
                    cand_rc.left += 6;

                    DrawTextW(
                        mem_dc,
                        wide_cand.as_ptr(),
                        -1,
                        &mut cand_rc,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                    );
                }

                // Restore GDI objects
                SelectObject(mem_dc, old_font);

                // Blit buffer to screen
                BitBlt(hdc, 0, 0, rect.right, rect.bottom, mem_dc, 0, 0, SRCCOPY);

                SelectObject(mem_dc, old_bm);
                DeleteObject(mem_bm);
                DeleteDC(mem_dc);
            }

            EndPaint(hwnd, &ps);
            0
        }
        WM_LBUTTONDOWN => {
            let y = (lparam >> 16) as i32;
            if y > HEADER_HEIGHT {
                let clicked_idx = ((y - HEADER_HEIGHT - 4) / CANDIDATE_ITEM_HEIGHT) as usize;
                super::hook::KeyboardHook::on_candidate_clicked(clicked_idx);
            }
            0
        }
        WM_DESTROY => 0,
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
