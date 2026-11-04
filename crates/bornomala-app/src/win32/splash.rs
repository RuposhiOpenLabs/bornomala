//! Startup Splash Screen for Bornomala.
//! Displays the authentic 500x500 Bengali startup artwork in the center of the desktop
//! with smooth auto-hide after 2.5 seconds (or click-to-dismiss), matching classic Bijoy Bayanno / Ekushe UX.

use super::api::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

pub const SPLASH_WIDTH: i32 = 320;
pub const SPLASH_HEIGHT: i32 = 320;
pub const SPLASH_DURATION_MS: u32 = 2500;

static SPLASH_BMP_BYTES: &[u8] = include_bytes!("../../../../assets/branding/startup_screen_bn.bmp");
static IS_SPLASH_DISMISSED: AtomicBool = AtomicBool::new(false);
static ON_SPLASH_DISMISS: Mutex<Option<Box<dyn Fn() + Send + 'static>>> = Mutex::new(None);

#[allow(dead_code)]
pub struct SplashWindow {
    pub hwnd: HWND,
}

impl SplashWindow {
    pub fn show<F>(hinstance: HINSTANCE, on_dismiss: F) -> Option<Self>
    where
        F: Fn() + Send + 'static,
    {
        unsafe {
            if let Ok(mut guard) = ON_SPLASH_DISMISS.lock() {
                *guard = Some(Box::new(on_dismiss));
            }
            IS_SPLASH_DISMISSED.store(false, Ordering::SeqCst);

            let class_name = to_wide("BornomalaSplashClass");
            let mut wc: WNDCLASSEXW = std::mem::zeroed();
            wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
            wc.lpfnWndProc = Some(splash_wnd_proc);
            wc.hInstance = hinstance;
            wc.hCursor = LoadCursorW(0, 32512 as *const u16); // IDC_ARROW
            wc.hbrBackground = CreateSolidBrush(rgb(10, 31, 24)); // Emerald dark
            wc.lpszClassName = class_name.as_ptr();

            RegisterClassExW(&wc);

            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            let x = (screen_w - SPLASH_WIDTH) / 2;
            let y = (screen_h - SPLASH_HEIGHT) / 2;

            let window_title = to_wide("Bornomala (বর্ণমালা) — স্বাগতম");
            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                class_name.as_ptr(),
                window_title.as_ptr(),
                WS_POPUP | WS_VISIBLE,
                x,
                y,
                SPLASH_WIDTH,
                SPLASH_HEIGHT,
                0,
                0,
                hinstance,
                std::ptr::null_mut(),
            );

            if hwnd == 0 {
                return None;
            }

            // Set auto-hide timer for 2.5 seconds
            SetTimer(hwnd, 1, SPLASH_DURATION_MS, None);
            UpdateWindow(hwnd);

            Some(Self { hwnd })
        }
    }
}

unsafe extern "system" fn splash_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            // Paint embedded 500x500 Bengali startup artwork
            if SPLASH_BMP_BYTES.len() >= 54 && &SPLASH_BMP_BYTES[0..2] == b"BM" {
                let bmi_ptr = SPLASH_BMP_BYTES.as_ptr().add(14) as *const BITMAPINFO;
                let bits_ptr = SPLASH_BMP_BYTES.as_ptr().add(54) as *const std::ffi::c_void;

                SetDIBitsToDevice(
                    hdc,
                    0,
                    0,
                    SPLASH_WIDTH as u32,
                    SPLASH_HEIGHT as u32,
                    0,
                    0,
                    0,
                    SPLASH_HEIGHT as u32,
                    bits_ptr,
                    bmi_ptr,
                    0, // DIB_RGB_COLORS
                );
            } else {
                // Fallback elegant background
                let bg_brush = CreateSolidBrush(rgb(10, 31, 24));
                let mut rc: RECT = std::mem::zeroed();
                GetClientRect(hwnd, &mut rc);
                FillRect(hdc, &rc, bg_brush);
                DeleteObject(bg_brush);
            }

            // Draw a subtle, sleek emerald loading bar at the very bottom
            let progress_brush = CreateSolidBrush(rgb(16, 185, 129)); // #10b981
            let progress_rc = RECT {
                left: 0,
                top: SPLASH_HEIGHT - 6,
                right: SPLASH_WIDTH,
                bottom: SPLASH_HEIGHT,
            };
            FillRect(hdc, &progress_rc, progress_brush);
            DeleteObject(progress_brush);

            EndPaint(hwnd, &ps);
            0
        }

        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_KEYDOWN => {
            // Dismiss immediately on user click or key press
            dismiss_splash(hwnd);
            0
        }

        WM_TIMER => {
            if wparam == 1 {
                dismiss_splash(hwnd);
            }
            0
        }

        WM_DESTROY => {
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn dismiss_splash(hwnd: HWND) {
    if !IS_SPLASH_DISMISSED.swap(true, Ordering::SeqCst) {
        KillTimer(hwnd, 1);
        DestroyWindow(hwnd);
        let cb_opt = if let Ok(mut guard) = ON_SPLASH_DISMISS.lock() {
            guard.take()
        } else {
            None
        };
        if let Some(cb) = cb_opt {
            cb();
        }
    }
}
