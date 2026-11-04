//! Mini Taskbar Dock Widget for Bornomala.
//! Renders the exact prototype [ [ব] | EN ] tray button docked DIRECTLY IN the Windows taskbar.
//! Positioned inside the taskbar, adjacent to the System Tray notification area.
//! Provides instant 1-click restore to the full floating toolbar.

use super::api::*;
use super::assets::{self, IconId};
use super::toolbar::{FloatingToolbar, CURRENT_SCHEME};
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};

pub const DOCK_WIDTH: i32 = 82;
pub const DOCK_HEIGHT: i32 = 28;

static mut DOCK_HWND: HWND = 0;
static mut FONT_DOCK: HFONT = 0;
static IS_HOVERED: AtomicBool = AtomicBool::new(false);
static IS_DRAGGING: AtomicBool = AtomicBool::new(false);
static mut DRAG_START_X: i32 = 0;
static mut DRAG_START_Y: i32 = 0;
static mut DRAG_START_WIN_X: i32 = 0;

pub struct TrayDock;

impl TrayDock {
    pub fn create(hinstance: HINSTANCE) {
        unsafe {
            let class_name = to_wide("BornomalaTrayDockClass");
            let mut wc: WNDCLASSEXW = std::mem::zeroed();
            wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
            wc.lpfnWndProc = Some(dock_wnd_proc);
            wc.hInstance = hinstance;
            wc.hCursor = LoadCursorW(0, 32512 as *const u16);
            wc.hbrBackground = CreateSolidBrush(rgb(251, 243, 220));
            wc.lpszClassName = class_name.as_ptr();

            RegisterClassExW(&wc);

            let font_name = to_wide("Nirmala UI");
            FONT_DOCK = CreateFontW(
                -12, 0, 0, 0, FW_BOLD, 0, 0, 0,
                DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
                font_name.as_ptr(),
            );

            // Compute exact position INSIDE the Windows taskbar
            let (x, y) = Self::get_taskbar_position(DOCK_WIDTH, DOCK_HEIGHT);

            let title = to_wide("Bornomala Taskbar Dock");
            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_POPUP, // Hidden initially
                x,
                y,
                DOCK_WIDTH,
                DOCK_HEIGHT,
                0,
                0,
                hinstance,
                std::ptr::null_mut(),
            );

            if hwnd != 0 {
                let rgn = CreateRoundRectRgn(0, 0, DOCK_WIDTH + 1, DOCK_HEIGHT + 1, 8, 8);
                SetWindowRgn(hwnd, rgn, TRUE);
                DOCK_HWND = hwnd;

                // Monitor taskbar layout changes periodically (every 1.5s)
                SetTimer(hwnd, 701, 1500, None);
            }
        }
    }

    pub fn show() {
        unsafe {
            if DOCK_HWND != 0 {
                // Re-evaluate taskbar bounds to dock precisely inside the taskbar
                let (x, y) = Self::get_taskbar_position(DOCK_WIDTH, DOCK_HEIGHT);
                SetWindowPos(
                    DOCK_HWND,
                    HWND_TOPMOST,
                    x, y, DOCK_WIDTH, DOCK_HEIGHT,
                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                );
                InvalidateRect(DOCK_HWND, std::ptr::null(), TRUE);
                UpdateWindow(DOCK_HWND);
            }
        }
    }

    pub fn hide() {
        unsafe {
            if DOCK_HWND != 0 {
                ShowWindow(DOCK_HWND, SW_HIDE);
            }
        }
    }

    pub fn update_mode() {
        unsafe {
            if DOCK_HWND != 0 {
                InvalidateRect(DOCK_HWND, std::ptr::null(), TRUE);
            }
        }
    }

    /// Computes the exact screen coordinates to place the widget DIRECTLY IN the Windows taskbar.
    /// Vertically centered within taskbar height, placed adjacent to the notification area.
    pub fn get_taskbar_position(width: i32, height: i32) -> (i32, i32) {
        unsafe {
            let h_taskbar = FindWindowW(to_wide("Shell_TrayWnd").as_ptr(), std::ptr::null());
            if h_taskbar != 0 {
                let mut tb_rc: RECT = std::mem::zeroed();
                GetWindowRect(h_taskbar, &mut tb_rc);

                let tb_w = tb_rc.right - tb_rc.left;
                let tb_h = tb_rc.bottom - tb_rc.top;

                // Find notification area (TrayNotifyWnd) inside taskbar
                let h_tray = FindWindowExW(h_taskbar, 0, to_wide("TrayNotifyWnd").as_ptr(), std::ptr::null());
                let mut tray_rc: RECT = std::mem::zeroed();
                let has_tray = h_tray != 0 && GetWindowRect(h_tray, &mut tray_rc) != 0;

                if tb_h <= tb_w {
                    // Horizontal taskbar (typically at screen bottom or top)
                    // Vertically center inside the taskbar itself
                    let y = tb_rc.top + (tb_h - height) / 2;

                    // Place horizontally adjacent to system tray notification icons
                    let x = if has_tray && tray_rc.left > tb_rc.left + 50 {
                        tray_rc.left - width - 6
                    } else {
                        tb_rc.right - width - 180
                    };
                    return (x, y);
                } else {
                    // Vertical taskbar (left or right side of screen)
                    let x = tb_rc.left + (tb_w - width) / 2;
                    let y = if has_tray && tray_rc.top > tb_rc.top + 50 {
                        tray_rc.top - height - 6
                    } else {
                        tb_rc.bottom - height - 180
                    };
                    return (x, y);
                }
            }

            // Fallback using SPI_GETWORKAREA
            let mut work_area: RECT = std::mem::zeroed();
            SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut work_area as *mut _ as *mut c_void, 0);
            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);

            // In standard bottom taskbar setup, work_area.bottom is where taskbar starts
            let tb_top = work_area.bottom;
            let tb_h = (screen_h - work_area.bottom).max(40);
            let y = tb_top + (tb_h - height) / 2;
            let x = screen_w - width - 200;
            (x, y)
        }
    }
}

unsafe extern "system" fn dock_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let color_green = rgb(6, 106, 74);
            let color_paper = rgb(251, 243, 220);
            let color_hover = rgb(255, 255, 255); // Prototype hover: background #fff

            let is_hovered = IS_HOVERED.load(Ordering::Relaxed);
            let fill_color = if is_hovered { color_hover } else { color_paper };

            // 1. Background Fill with 2px Green Border inside Taskbar
            let bg_brush = CreateSolidBrush(fill_color);
            let border_pen = CreatePen(PS_SOLID, 2, color_green);
            let old_brush = SelectObject(hdc, bg_brush);
            let old_pen = SelectObject(hdc, border_pen);
            RoundRect(hdc, 1, 1, DOCK_WIDTH - 1, DOCK_HEIGHT - 1, 8, 8);

            // 2. Draw Authentic Logo (28x24 scaled down to 22x19)
            assets::draw_icon(hdc, IconId::Logo, 5, 2);

            // 3. Divider Line
            let div_pen = CreatePen(PS_SOLID, 1, color_green);
            SelectObject(hdc, div_pen);
            MoveToEx(hdc, 38, 5, std::ptr::null_mut());
            LineTo(hdc, 38, DOCK_HEIGHT - 5);
            DeleteObject(div_pen);

            // 4. Mode Text Badge (EN / UNI / ANSI / PHO)
            let mode_str = match CURRENT_SCHEME.load(Ordering::SeqCst) {
                1 => "ANSI",
                2 => "PHO",
                3 => "EN",
                _ => "UNI",
            };

            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, color_green);
            SelectObject(hdc, FONT_DOCK);
            let mut text_rc = RECT { left: 42, top: 0, right: DOCK_WIDTH - 4, bottom: DOCK_HEIGHT };
            let wide_mode = to_wide(mode_str);
            DrawTextW(hdc, wide_mode.as_ptr(), -1, &mut text_rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

            // Cleanup
            SelectObject(hdc, old_pen);
            SelectObject(hdc, old_brush);
            DeleteObject(border_pen);
            DeleteObject(bg_brush);

            EndPaint(hwnd, &ps);
            0
        }

        WM_MOUSEMOVE => {
            if !IS_HOVERED.swap(true, Ordering::SeqCst) {
                InvalidateRect(hwnd, std::ptr::null(), TRUE);
            }

            if IS_DRAGGING.load(Ordering::SeqCst) {
                let mut pt: POINT = std::mem::zeroed();
                GetCursorPos(&mut pt);
                let dx = pt.x - DRAG_START_X;
                if dx.abs() > 4 {
                    let mut cur_rc: RECT = std::mem::zeroed();
                    GetWindowRect(hwnd, &mut cur_rc);
                    let new_x = DRAG_START_WIN_X + dx;
                    SetWindowPos(
                        hwnd,
                        HWND_TOPMOST,
                        new_x,
                        cur_rc.top,
                        DOCK_WIDTH,
                        DOCK_HEIGHT,
                        SWP_NOACTIVATE | SWP_NOSIZE,
                    );
                }
            }

            let mut tme: TRACKMOUSEEVENT = std::mem::zeroed();
            tme.cbSize = std::mem::size_of::<TRACKMOUSEEVENT>() as u32;
            tme.dwFlags = TME_LEAVE;
            tme.hwndTrack = hwnd;
            TrackMouseEvent(&mut tme);
            0
        }

        WM_MOUSELEAVE => {
            IS_HOVERED.store(false, Ordering::SeqCst);
            InvalidateRect(hwnd, std::ptr::null(), TRUE);
            0
        }

        WM_LBUTTONDOWN => {
            SetCapture(hwnd);
            let mut pt: POINT = std::mem::zeroed();
            GetCursorPos(&mut pt);
            DRAG_START_X = pt.x;
            DRAG_START_Y = pt.y;
            let mut cur_rc: RECT = std::mem::zeroed();
            GetWindowRect(hwnd, &mut cur_rc);
            DRAG_START_WIN_X = cur_rc.left;
            IS_DRAGGING.store(true, Ordering::SeqCst);
            0
        }

        WM_LBUTTONUP => {
            ReleaseCapture();
            let was_dragging = IS_DRAGGING.swap(false, Ordering::SeqCst);
            let mut pt: POINT = std::mem::zeroed();
            GetCursorPos(&mut pt);
            let dx = (pt.x - DRAG_START_X).abs();
            let dy = (pt.y - DRAG_START_Y).abs();

            if was_dragging && dx < 5 && dy < 5 {
                // User clicked without dragging: restore the full floating toolbar!
                TrayDock::hide();
                FloatingToolbar::show();
            }
            0
        }

        WM_RBUTTONUP => {
            // Context menu from dock button
            let mut pt: POINT = std::mem::zeroed();
            GetCursorPos(&mut pt);

            let hmenu = CreatePopupMenu();
            let add_item = |menu: HMENU, id: usize, text: &str| {
                let wide = to_wide(text);
                AppendMenuW(menu, MF_STRING, id, wide.as_ptr());
            };

            add_item(hmenu, 101, "📌 ফ্লোটিং বার দেখান (Restore Toolbar)");
            add_item(hmenu, 102, "🔄 বাংলা রূপান্তরকারী (Converter)...");
            AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
            add_item(hmenu, 103, "✕ প্রস্থান (Exit Bornomala)");

            SetForegroundWindow(hwnd);
            let cmd = TrackPopupMenu(hmenu, TPM_RIGHTBUTTON | 0x0100 /* TPM_RETURNCMD */, pt.x, pt.y, 0, hwnd, std::ptr::null());
            DestroyMenu(hmenu);

            match cmd as usize {
                101 => {
                    TrayDock::hide();
                    FloatingToolbar::show();
                }
                102 => {
                    super::converter::ConverterWindow::open();
                }
                103 => {
                    PostQuitMessage(0);
                }
                _ => {}
            }
            0
        }

        WM_TIMER => {
            // If visible and not actively dragged, verify dock is cleanly inside the taskbar
            if !IS_DRAGGING.load(Ordering::SeqCst) && IsWindowVisible(hwnd) != 0 {
                let (expected_x, expected_y) = TrayDock::get_taskbar_position(DOCK_WIDTH, DOCK_HEIGHT);
                let mut cur_rc: RECT = std::mem::zeroed();
                GetWindowRect(hwnd, &mut cur_rc);

                // Only adjust if taskbar moved or notification area shifted (> 10px difference)
                if (cur_rc.top - expected_y).abs() > 2 || (cur_rc.left - expected_x).abs() > 40 {
                    SetWindowPos(
                        hwnd,
                        HWND_TOPMOST,
                        expected_x,
                        expected_y,
                        DOCK_WIDTH,
                        DOCK_HEIGHT,
                        SWP_NOACTIVATE | SWP_NOSIZE,
                    );
                }
            }
            0
        }

        WM_DESTROY => {
            KillTimer(hwnd, 701);
            if FONT_DOCK != 0 {
                DeleteObject(FONT_DOCK);
                FONT_DOCK = 0;
            }
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
