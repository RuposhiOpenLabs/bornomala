//! Windows GUI Implementation Module for Bornomala.

pub mod api;
pub mod assets;
pub mod converter;
pub mod dock;
pub mod hook;
pub mod splash;
pub mod toolbar;
pub mod tray;
pub mod updater;

use api::*;
use converter::ConverterWindow;
use dock::TrayDock;
use hook::KeyboardHook;
use splash::SplashWindow;
use toolbar::FloatingToolbar;
use tray::SystemTray;

static mut MAIN_HWND: HWND = 0;
static mut SYSTEM_TRAY: Option<SystemTray> = None;

pub fn minimize_toolbar_to_tray() {
    unsafe {
        FloatingToolbar::minimize();
        TrayDock::show();
        if let Some(ref mut tray) = SYSTEM_TRAY {
            tray.notify_minimized();
        }
    }
}

pub fn run_app(converter_only: bool, _startup_mode: bool) {
    unsafe {
        let hinst = GetModuleHandleW(std::ptr::null());

        // --------------------------------------------------------------------
        // 1. Standalone Converter Mode (--converter / -c)
        // --------------------------------------------------------------------
        if converter_only {
            let conv_class = to_wide("BornomalaConverterClass");
            let existing_conv = FindWindowW(conv_class.as_ptr(), std::ptr::null());
            if existing_conv != 0 {
                ShowWindow(existing_conv, SW_RESTORE);
                SetForegroundWindow(existing_conv);
                return;
            }

            ConverterWindow::open_standalone();

            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            return;
        }

        // --------------------------------------------------------------------
        // 2. Main Bornomala Application (Typing Suite, Floating Bar, Tray)
        // --------------------------------------------------------------------
        let main_class = to_wide("BornomalaCoreMsgClass");
        let main_title = to_wide("Bornomala Core Event Window");

        // Check if main instance is already running
        let existing_main = FindWindowW(main_class.as_ptr(), main_title.as_ptr());
        if existing_main != 0 {
            // Already running! Signal existing instance to restore toolbar & bring to top
            PostMessageW(existing_main, WM_BORNOMALA_RESTORE_TOOLBAR, 0, 0);
            return;
        }

        // Enforce single instance via Windows Named Mutex
        let mutex_name = to_wide("Bornomala_SingleInstance_Mutex");
        let h_mutex = CreateMutexW(std::ptr::null_mut(), TRUE, mutex_name.as_ptr());
        if h_mutex != 0 && GetLastError() == 183 {
            return;
        }

        // Create main event message-handler window
        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.lpfnWndProc = Some(main_event_wnd_proc);
        wc.hInstance = hinst;
        wc.lpszClassName = main_class.as_ptr();
        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            0,
            main_class.as_ptr(),
            main_title.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            hinst,
            std::ptr::null_mut(),
        );

        MAIN_HWND = hwnd;

        // 1. Initialize Authentic SVG Bitmap Assets & Tray Icons
        assets::init_assets(0);

        // 2. Initialize System Tray Notification Icon & Dock Widget (default English)
        TrayDock::create(hinst);
        let mut tray = SystemTray::new(hwnd, hinst);
        tray.update_tooltip(bornomala_engine::InputScheme::PassThrough);
        SYSTEM_TRAY = Some(tray);

        // 3. Initialize Low-Level Keyboard Hook & Global Hotkeys (F12, Ctrl+Alt+V, etc.)
        KeyboardHook::init(hwnd, hinst);

        // 4. Create the Top Floating Toolbar (initially hidden)
        let _tb = FloatingToolbar::create(hinst);

        // 5. Show the authentic Bengali 500x500 Startup Splash Screen in center of desktop
        let _splash = SplashWindow::show(hinst, || {
            // Splash auto-hide/click callback: Reveal the elegant parchment top toolbar!
            FloatingToolbar::show();
            SetForegroundWindow(FloatingToolbar::get_hwnd());
        });

        // 6. Start asynchronous background GitHub update check
        updater::init_update_checker("0.1.0");

        // Standard Win32 Message Loop
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Cleanup
        assets::cleanup_assets();
        KeyboardHook::cleanup(hwnd);
        SYSTEM_TRAY = None;
        if h_mutex != 0 {
            CloseHandle(h_mutex);
        }
    }
}

unsafe extern "system" fn main_event_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_HOTKEY => {
            let hotkey_id = wparam as i32;
            KeyboardHook::handle_hotkey(hotkey_id);
            if let Some(ref mut tray) = SYSTEM_TRAY {
                tray.update_tooltip(FloatingToolbar::get_scheme());
            }
            0
        }

        WM_COMMAND => {
            let cmd_id = wparam & 0xFFFF;
            tray::SystemTray::handle_menu_command(cmd_id);
            if let Some(ref mut tray) = SYSTEM_TRAY {
                tray.update_tooltip(FloatingToolbar::get_scheme());
            }
            0
        }

        WM_BORNOMALA_TRAY => {
            tray::SystemTray::handle_tray_message(hwnd, lparam);
            0
        }

        WM_BORNOMALA_RESTORE_TOOLBAR => {
            // Restore and focus the top floating toolbar
            FloatingToolbar::show();
            SetForegroundWindow(FloatingToolbar::get_hwnd());
            0
        }

        WM_BORNOMALA_OPEN_CONVERTER => {
            ConverterWindow::open();
            0
        }

        WM_BORNOMALA_TRIGGER_UPDATE_CHECK => {
            updater::trigger_manual_update_check("0.1.0");
            0
        }

        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
