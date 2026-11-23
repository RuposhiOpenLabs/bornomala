//! Bornomala Secure Update System
//!
//! Provides:
//! 1. GitHub Releases checker for new stable versions.
//! 2. User update notification popup with [Update Now], [Later], and [Skip this version].
//! 3. Live downloading window with real-time transfer speed (e.g. MB/s) and progress bar.
//! 4. Cryptographic integrity & Ed25519 signature verification against official public key.
//! 5. UAC Administrator elevation for seamless, one-click Update Wizard (no manual re-setup).
//! 6. Automated silent upgrade and application restart.

use std::ffi::c_void;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use bornomala_update::{IntegrityVerifier, UpdateCandidate, UpdateChecker, UpdatePreferences};

use super::api::*;

static SHOWN_THIS_SESSION: AtomicBool = AtomicBool::new(false);
static PENDING_CANDIDATE: Mutex<Option<UpdateCandidate>> = Mutex::new(None);
static CHECK_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub enum PromptAction {
    UpdateNow(UpdateCandidate),
    Later,
    Skip,
    Dismissed,
}

pub fn init_update_checker(current_version: &str) {
    let ver = current_version.to_string();
    std::thread::spawn(move || {
        // Wait 4 seconds after app launch before checking GitHub releases in the background
        std::thread::sleep(std::time::Duration::from_secs(4));
        check_for_updates(&ver, false);
    });
}

pub fn trigger_manual_update_check(current_version: &str) {
    let ver = current_version.to_string();
    std::thread::spawn(move || {
        check_for_updates(&ver, true);
    });
}

fn check_for_updates(current_version: &str, is_manual: bool) {
    if CHECK_IN_PROGRESS
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }
    struct CheckGuard;
    impl Drop for CheckGuard {
        fn drop(&mut self) {
            CHECK_IN_PROGRESS.store(false, Ordering::SeqCst);
        }
    }
    let _guard = CheckGuard;

    let url = UpdateChecker::GITHUB_LATEST_RELEASE_URL;
    match fetch_url_string(url) {
        Ok(json_response) => {
            match UpdateChecker::parse_latest_release(&json_response, current_version) {
                Ok(Some(candidate)) => {
                    let prefs = UpdatePreferences::load();
                    if !is_manual
                        && !prefs.should_notify(
                            &candidate.version,
                            SHOWN_THIS_SESSION.load(Ordering::SeqCst),
                        )
                    {
                        return;
                    }

                    // Save candidate
                    if let Ok(mut lock) = PENDING_CANDIDATE.lock() {
                        *lock = Some(candidate.clone());
                    }

                    let action = unsafe {
                        UpdatePromptWindow::show(candidate.clone(), current_version)
                    };

                    match action {
                        PromptAction::UpdateNow(cand) => unsafe {
                            DownloaderWindow::run(cand);
                        },
                        PromptAction::Later => {
                            SHOWN_THIS_SESSION.store(true, Ordering::SeqCst);
                            let mut prefs = UpdatePreferences::load();
                            prefs.set_postponed(&candidate.version);
                        }
                        PromptAction::Skip => {
                            let mut prefs = UpdatePreferences::load();
                            prefs.set_skip_version(&candidate.version);
                        }
                        PromptAction::Dismissed => {}
                    }
                }
                Ok(None) => {
                    if is_manual {
                        unsafe {
                            let msg = format!(
                                "বর্ণমালা সর্বশেষ সংস্করণে রয়েছে (v{})। নতুন কোনো আপডেট নেই।",
                                current_version
                            );
                            let title = "বর্ণমালা আপডেট";
                            MessageBoxW(
                                0,
                                to_wide(&msg).as_ptr(),
                                to_wide(title).as_ptr(),
                                0x00000040,
                            );
                        }
                    }
                }
                Err(e) => {
                    if is_manual {
                        unsafe {
                            let msg = format!("আপডেট পরীক্ষা করতে ব্যর্থ হয়েছে:\n{}", e);
                            let title = "বর্ণমালা আপডেট ত্রুটি";
                            MessageBoxW(
                                0,
                                to_wide(&msg).as_ptr(),
                                to_wide(title).as_ptr(),
                                0x00000010,
                            );
                        }
                    }
                }
            }
        }
        Err(e) => {
            if is_manual {
                unsafe {
                    let msg = format!("সার্ভারের সাথে সংযোগ স্থাপন করা যায়নি:\n{}", e);
                    let title = "বর্ণমালা আপডেট ত্রুটি";
                    MessageBoxW(
                        0,
                        to_wide(&msg).as_ptr(),
                        to_wide(title).as_ptr(),
                        0x00000010,
                    );
                }
            }
        }
    }
}

// ----------------------------------------------------------------------------
// WinINet HTTP Fetching & Streaming Downloader
// ----------------------------------------------------------------------------

pub fn fetch_url_string(url: &str) -> Result<String, String> {
    unsafe {
        let ua = format!("Bornomala-UpdateChecker/{}", env!("CARGO_PKG_VERSION"));
        let user_agent = to_wide(&ua);
        let h_internet = InternetOpenW(
            user_agent.as_ptr(),
            INTERNET_OPEN_TYPE_PRECONFIG,
            std::ptr::null(),
            std::ptr::null(),
            0,
        );
        if h_internet == 0 {
            return Err("Failed to initialize Windows Internet connection (WinINet)".into());
        }

        let wide_url = to_wide(url);
        let flags = INTERNET_FLAG_RELOAD | INTERNET_FLAG_SECURE | INTERNET_FLAG_NO_CACHE_WRITE;
        let h_url = InternetOpenUrlW(h_internet, wide_url.as_ptr(), std::ptr::null(), 0, flags, 0);
        if h_url == 0 {
            InternetCloseHandle(h_internet);
            return Err("Failed to open remote URL".into());
        }

        let mut buffer = [0u8; 8192];
        let mut response_bytes = Vec::new();
        loop {
            let mut bytes_read: u32 = 0;
            let success = InternetReadFile(
                h_url,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as u32,
                &mut bytes_read,
            );
            if success == 0 || bytes_read == 0 {
                break;
            }
            response_bytes.extend_from_slice(&buffer[..bytes_read as usize]);
        }

        InternetCloseHandle(h_url);
        InternetCloseHandle(h_internet);

        String::from_utf8(response_bytes)
            .map_err(|e| format!("Invalid UTF-8 response from update server: {}", e))
    }
}

pub fn download_file_with_progress<F>(
    url: &str,
    dest_path: &Path,
    cancel_flag: Arc<AtomicBool>,
    mut on_progress: F,
) -> Result<(), String>
where
    F: FnMut(u64, u64, f64), // (downloaded_bytes, total_bytes, speed_bytes_per_sec)
{
    unsafe {
        let ua = format!("Bornomala-Downloader/{}", env!("CARGO_PKG_VERSION"));
        let user_agent = to_wide(&ua);
        let h_internet = InternetOpenW(
            user_agent.as_ptr(),
            INTERNET_OPEN_TYPE_PRECONFIG,
            std::ptr::null(),
            std::ptr::null(),
            0,
        );
        if h_internet == 0 {
            return Err("Failed to initialize Windows Internet session".into());
        }

        let wide_url = to_wide(url);
        let flags = INTERNET_FLAG_RELOAD | INTERNET_FLAG_SECURE | INTERNET_FLAG_NO_CACHE_WRITE;
        let h_url = InternetOpenUrlW(h_internet, wide_url.as_ptr(), std::ptr::null(), 0, flags, 0);
        if h_url == 0 {
            InternetCloseHandle(h_internet);
            return Err("Failed to connect to download source".into());
        }

        // Query total content length
        let mut content_length: u32 = 0;
        let mut buf_len: u32 = std::mem::size_of::<u32>() as u32;
        let mut index: u32 = 0;
        let total_size = if HttpQueryInfoW(
            h_url,
            HTTP_QUERY_CONTENT_LENGTH | HTTP_QUERY_FLAG_NUMBER,
            &mut content_length as *mut u32 as *mut c_void,
            &mut buf_len,
            &mut index,
        ) != 0
        {
            content_length as u64
        } else {
            0
        };

        if let Some(parent) = dest_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut file = fs::File::create(dest_path)
            .map_err(|e| format!("Cannot create local download file: {}", e))?;

        let mut buffer = [0u8; 16384];
        let mut downloaded: u64 = 0;
        let mut last_tick = Instant::now();
        let mut bytes_since_tick: u64 = 0;
        let mut current_speed: f64 = 0.0;

        loop {
            if cancel_flag.load(Ordering::SeqCst) {
                InternetCloseHandle(h_url);
                InternetCloseHandle(h_internet);
                drop(file);
                let _ = fs::remove_file(dest_path);
                return Err("Download cancelled by user".into());
            }

            let mut bytes_read: u32 = 0;
            let success = InternetReadFile(
                h_url,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as u32,
                &mut bytes_read,
            );
            if success == 0 || bytes_read == 0 {
                break;
            }

            use std::io::Write;
            file.write_all(&buffer[..bytes_read as usize])
                .map_err(|e| format!("Disk write error: {}", e))?;

            downloaded += bytes_read as u64;
            bytes_since_tick += bytes_read as u64;

            let elapsed = last_tick.elapsed().as_secs_f64();
            if elapsed >= 0.25 {
                current_speed = (bytes_since_tick as f64) / elapsed;
                bytes_since_tick = 0;
                last_tick = Instant::now();
                on_progress(downloaded, total_size, current_speed);
            }
        }

        InternetCloseHandle(h_url);
        InternetCloseHandle(h_internet);

        on_progress(downloaded, total_size, current_speed);
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// 1. Update Available Notification Popup (UpdatePromptWindow)
// ----------------------------------------------------------------------------

const BTN_UPDATE_NOW: usize = 1001;
const BTN_LATER: usize = 1002;
const BTN_SKIP: usize = 1003;

pub struct UpdatePromptWindow;

static mut PROMPT_HWND: HWND = 0;
static PROMPT_INFO: Mutex<Option<(UpdateCandidate, String)>> = Mutex::new(None);
static PROMPT_ACTION: Mutex<Option<PromptAction>> = Mutex::new(None);

impl UpdatePromptWindow {
    pub unsafe fn show(candidate: UpdateCandidate, current_version: &str) -> PromptAction {
        if let Ok(mut lock) = PROMPT_INFO.lock() {
            *lock = Some((candidate.clone(), current_version.to_string()));
        }
        if let Ok(mut lock) = PROMPT_ACTION.lock() {
            *lock = None;
        }

        let hinst = GetModuleHandleW(std::ptr::null());
        let class_name = to_wide("BornomalaUpdatePromptClass");

        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.lpfnWndProc = Some(prompt_wnd_proc);
        wc.hInstance = hinst;
        wc.lpszClassName = class_name.as_ptr();
        wc.hbrBackground = CreateSolidBrush(rgb(251, 243, 220)); // Parchment #fbf3dc
        wc.hCursor = LoadCursorW(0, 32512 as *const u16); // IDC_ARROW
        RegisterClassExW(&wc);

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let width = 480;
        let height = 260;
        let x = (screen_w - width) / 2;
        let y = (screen_h - height) / 2;

        let title = to_wide("বর্ণমালা — নতুন সংস্করণ উপলব্ধ (Update Available)");
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP | WS_VISIBLE | WS_BORDER,
            x,
            y,
            width,
            height,
            0,
            0,
            hinst,
            std::ptr::null_mut(),
        );

        PROMPT_HWND = hwnd;

        // Create Buttons
        let btn_font = CreateFontW(
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
            to_wide("Nirmala UI").as_ptr(),
        );

        // 1. Update Now button
        let btn_now = CreateWindowExW(
            0,
            to_wide("BUTTON").as_ptr(),
            to_wide("এখনই আপডেট করুন (Update Now)").as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON,
            24,
            200,
            200,
            38,
            hwnd,
            BTN_UPDATE_NOW,
            hinst,
            std::ptr::null_mut(),
        );
        SendMessageW(btn_now, WM_SETFONT, btn_font, 1);

        // 2. Later button
        let btn_lat = CreateWindowExW(
            0,
            to_wide("BUTTON").as_ptr(),
            to_wide("পরে মনে করিয়ে দিন (Later)").as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
            234,
            200,
            140,
            38,
            hwnd,
            BTN_LATER,
            hinst,
            std::ptr::null_mut(),
        );
        SendMessageW(btn_lat, WM_SETFONT, btn_font, 1);

        // 3. Skip version button
        let btn_skp = CreateWindowExW(
            0,
            to_wide("BUTTON").as_ptr(),
            to_wide("বাদ দিন (Skip)").as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
            382,
            200,
            74,
            38,
            hwnd,
            BTN_SKIP,
            hinst,
            std::ptr::null_mut(),
        );
        SendMessageW(btn_skp, WM_SETFONT, btn_font, 1);

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
        SetForegroundWindow(hwnd);

        // Run modal message pump on this thread
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        PROMPT_ACTION
            .lock()
            .ok()
            .and_then(|mut a| a.take())
            .unwrap_or(PromptAction::Dismissed)
    }
}

unsafe extern "system" fn prompt_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_LBUTTONDOWN => {
            ReleaseCapture();
            SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as WPARAM, 0);
            0
        }

        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            // 1. Header Banner (Bornomala Green #066a4a)
            let header_brush = CreateSolidBrush(rgb(6, 106, 74));
            let header_rect = RECT {
                left: 0,
                top: 0,
                right: 480,
                bottom: 58,
            };
            FillRect(hdc, &header_rect, header_brush);
            DeleteObject(header_brush);

            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, rgb(254, 247, 222)); // Cream text

            let header_font = CreateFontW(
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
                to_wide("Nirmala UI").as_ptr(),
            );
            let old_font = SelectObject(hdc, header_font);
            let mut h_text_rect = RECT {
                left: 20,
                top: 16,
                right: 460,
                bottom: 50,
            };
            let title_text = to_wide("বর্ণমালার নতুন সংস্করণ প্রকাশিত হয়েছে!");
            DrawTextW(
                hdc,
                title_text.as_ptr(),
                -1,
                &mut h_text_rect,
                DT_LEFT | DT_SINGLELINE,
            );

            // 2. Body Text
            let info_clone = PROMPT_INFO.lock().ok().and_then(|c| c.clone());
            if let Some((ref cand, ref cur_ver)) = info_clone {
                SetTextColor(hdc, rgb(6, 106, 74));
                let body_font = CreateFontW(
                    -14,
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
                    to_wide("Nirmala UI").as_ptr(),
                );
                SelectObject(hdc, body_font);

                let ver_str = format!("নতুন সংস্করণ: v{} (আপনার সংস্করণ: v{})", cand.version, cur_ver);
                let mut v_rect = RECT {
                    left: 24,
                    top: 72,
                    right: 460,
                    bottom: 96,
                };
                DrawTextW(
                    hdc,
                    to_wide(&ver_str).as_ptr(),
                    -1,
                    &mut v_rect,
                    DT_LEFT | DT_SINGLELINE,
                );

                let desc_font = CreateFontW(
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
                    to_wide("Nirmala UI").as_ptr(),
                );
                SelectObject(hdc, desc_font);
                SetTextColor(hdc, rgb(30, 40, 35));

                let note_str = "এই সংস্করণে নতুন ফিচার, গতি বৃদ্ধি ও নিরাপত্তা হালনাগাদ যুক্ত হয়েছে।\nআপডেটটি সম্পূর্ণ স্বয়ংক্রিয় এবং কোনো ফাইল পুনরায় কনফিগার করতে হবে না।";
                let mut d_rect = RECT {
                    left: 24,
                    top: 104,
                    right: 456,
                    bottom: 180,
                };
                DrawTextW(
                    hdc,
                    to_wide(note_str).as_ptr(),
                    -1,
                    &mut d_rect,
                    DT_LEFT | DT_WORDBREAK,
                );

                DeleteObject(body_font);
                DeleteObject(desc_font);
            }

            SelectObject(hdc, old_font);
            DeleteObject(header_font);
            EndPaint(hwnd, &ps);
            0
        }

        WM_COMMAND => {
            let id = wparam & 0xFFFF;
            match id {
                BTN_UPDATE_NOW => {
                    let cand = PROMPT_INFO.lock().ok().and_then(|c| c.as_ref().map(|(cand, _)| cand.clone()));
                    if let Some(cand) = cand {
                        if let Ok(mut lock) = PROMPT_ACTION.lock() {
                            *lock = Some(PromptAction::UpdateNow(cand));
                        }
                    }
                    DestroyWindow(hwnd);
                }
                BTN_LATER => {
                    if let Ok(mut lock) = PROMPT_ACTION.lock() {
                        *lock = Some(PromptAction::Later);
                    }
                    DestroyWindow(hwnd);
                }
                BTN_SKIP => {
                    if let Ok(mut lock) = PROMPT_ACTION.lock() {
                        *lock = Some(PromptAction::Skip);
                    }
                    DestroyWindow(hwnd);
                }
                _ => {}
            }
            0
        }

        WM_CLOSE => {
            if let Ok(mut lock) = PROMPT_ACTION.lock() {
                if lock.is_none() {
                    *lock = Some(PromptAction::Later);
                }
            }
            DestroyWindow(hwnd);
            0
        }

        WM_DESTROY => {
            PROMPT_HWND = 0;
            PostQuitMessage(0);
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// ----------------------------------------------------------------------------
// 2. Live Downloader Window (DownloaderWindow)
// ----------------------------------------------------------------------------

const BTN_CANCEL_DOWNLOAD: usize = 2001;
const WM_DL_PROGRESS: u32 = WM_USER + 310;
const WM_DL_FINISHED: u32 = WM_USER + 311;

pub struct DownloaderWindow;

static mut DL_HWND: HWND = 0;
static DL_PROGRESS_PCT: AtomicU64 = AtomicU64::new(0);
static DL_DOWNLOADED_KB: AtomicU64 = AtomicU64::new(0); // in KB
static DL_TOTAL_KB: AtomicU64 = AtomicU64::new(0); // in KB
static DL_SPEED_KB: AtomicU64 = AtomicU64::new(0); // in KB/s
static DL_CANCEL_FLAG: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);
static DL_STATUS_MSG: Mutex<String> = Mutex::new(String::new());
static DL_ERROR_MSG: Mutex<String> = Mutex::new(String::new());
static DL_TARGET_INFO: Mutex<Option<(PathBuf, String)>> = Mutex::new(None);

fn set_dl_status(msg: &str) {
    if let Ok(mut lock) = DL_STATUS_MSG.lock() {
        *lock = msg.to_string();
    }
}

fn get_dl_status() -> String {
    DL_STATUS_MSG.lock().map(|s| s.clone()).unwrap_or_default()
}

fn set_dl_error(msg: &str) {
    if let Ok(mut lock) = DL_ERROR_MSG.lock() {
        *lock = msg.to_string();
    }
}

fn get_dl_error() -> String {
    DL_ERROR_MSG.lock().map(|s| s.clone()).unwrap_or_default()
}

impl DownloaderWindow {
    pub unsafe fn run(candidate: UpdateCandidate) {
        let app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
        let update_dir = PathBuf::from(app_data).join("Bornomala").join("updates");
        let target_file = update_dir.join(&candidate.asset_name);

        if let Ok(mut lock) = DL_TARGET_INFO.lock() {
            *lock = Some((target_file.clone(), candidate.version.clone()));
        }

        DL_PROGRESS_PCT.store(0, Ordering::SeqCst);
        DL_DOWNLOADED_KB.store(0, Ordering::SeqCst);
        DL_TOTAL_KB.store(0, Ordering::SeqCst);
        DL_SPEED_KB.store(0, Ordering::SeqCst);
        set_dl_status("গিটহাব থেকে আপডেট ফাইল ডাউনলোড হচ্ছে...");
        set_dl_error("");

        let hinst = GetModuleHandleW(std::ptr::null());
        let class_name = to_wide("BornomalaDownloaderClass");

        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.lpfnWndProc = Some(dl_wnd_proc);
        wc.hInstance = hinst;
        wc.lpszClassName = class_name.as_ptr();
        wc.hbrBackground = CreateSolidBrush(rgb(251, 243, 220));
        wc.hCursor = LoadCursorW(0, 32512 as *const u16);
        RegisterClassExW(&wc);

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let width = 480;
        let height = 220;
        let x = (screen_w - width) / 2;
        let y = (screen_h - height) / 2;

        let title = to_wide("বর্ণমালা ডাউনলোডার (Downloading Update)");
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP | WS_VISIBLE | WS_BORDER,
            x,
            y,
            width,
            height,
            0,
            0,
            hinst,
            std::ptr::null_mut(),
        );

        DL_HWND = hwnd;

        let cancel_btn = CreateWindowExW(
            0,
            to_wide("BUTTON").as_ptr(),
            to_wide("বাতিল (Cancel)").as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
            340,
            165,
            110,
            34,
            hwnd,
            BTN_CANCEL_DOWNLOAD,
            hinst,
            std::ptr::null_mut(),
        );
        let btn_font = CreateFontW(
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
            to_wide("Nirmala UI").as_ptr(),
        );
        SendMessageW(cancel_btn, WM_SETFONT, btn_font, 1);

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
        SetForegroundWindow(hwnd);

        // Spawn background download task
        let cancel_flag = Arc::new(AtomicBool::new(false));
        if let Ok(mut lock) = DL_CANCEL_FLAG.lock() {
            *lock = Some(cancel_flag.clone());
        }

        let cand_clone = candidate.clone();
        let target_file_clone = target_file.clone();
        let dl_hwnd = hwnd;

        std::thread::spawn(move || {
            let res = download_file_with_progress(
                &cand_clone.download_url,
                &target_file_clone,
                cancel_flag.clone(),
                |downloaded, total, speed| {
                    let pct = if total > 0 {
                        (downloaded * 100) / total
                    } else {
                        0
                    };
                    DL_PROGRESS_PCT.store(pct, Ordering::Relaxed);
                    DL_DOWNLOADED_KB.store(downloaded / 1024, Ordering::Relaxed);
                    DL_TOTAL_KB.store(total / 1024, Ordering::Relaxed);
                    DL_SPEED_KB.store((speed / 1024.0) as u64, Ordering::Relaxed);

                    unsafe {
                        PostMessageW(dl_hwnd, WM_DL_PROGRESS, 0, 0);
                    }
                },
            );

            if cancel_flag.load(Ordering::SeqCst) {
                unsafe {
                    PostMessageW(dl_hwnd, WM_DL_FINISHED, 0, 0);
                }
                return;
            }

            match res {
                Ok(()) => {
                    set_dl_status("ক্রিপ্টোগ্রাফিক ডিজিটাল স্বাক্ষর ও হ্যাশ যাচাই করা হচ্ছে...");
                    unsafe {
                        PostMessageW(dl_hwnd, WM_DL_PROGRESS, 0, 0);
                    }

                    // Cryptographic Ed25519 & SHA-256 Verification
                    let verifier_res = IntegrityVerifier::official();
                    let verified = match verifier_res {
                        Ok(verifier) => verifier.verify_downloaded_payload(
                            &target_file_clone,
                            cand_clone.expected_sha256.as_deref(),
                            cand_clone.ed25519_signature.as_deref(),
                        ),
                        Err(e) => Err(e),
                    };

                    match verified {
                        Ok(_) => unsafe {
                            PostMessageW(dl_hwnd, WM_DL_FINISHED, 1, 0);
                        },
                        Err(err) => {
                            let _ = fs::remove_file(&target_file_clone);
                            set_dl_error(&format!(
                                "নিরাপত্তা সতর্কতা: ডাউনলোড করা ফাইলের ক্রিপ্টোগ্রাফিক যাচাইকরণ ব্যর্থ হয়েছে!\n({})\n\nসম্ভাব্য ম্যালওয়্যার প্রতিরোধে ফাইলটি মুছে ফেলা হয়েছে।",
                                err
                            ));
                            unsafe {
                                PostMessageW(dl_hwnd, WM_DL_FINISHED, 2, 0);
                            }
                        }
                    }
                }
                Err(err) => {
                    if err.contains("cancelled") {
                        unsafe {
                            PostMessageW(dl_hwnd, WM_DL_FINISHED, 0, 0);
                        }
                    } else {
                        set_dl_error(&format!("ডাউনলোড সম্পন্ন করা সম্ভব হয়নি:\n{}", err));
                        unsafe {
                            PostMessageW(dl_hwnd, WM_DL_FINISHED, 3, 0);
                        }
                    }
                }
            }
        });

        // Run modal message pump on this thread
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe extern "system" fn dl_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_LBUTTONDOWN => {
            ReleaseCapture();
            SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as WPARAM, 0);
            0
        }

        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            // Double buffering
            let mut client_rc: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut client_rc);
            let w = client_rc.right - client_rc.left;
            let h = client_rc.bottom - client_rc.top;

            let mem_dc = CreateCompatibleDC(hdc);
            let mem_bmp = CreateCompatibleBitmap(hdc, w, h);
            let old_bmp = SelectObject(mem_dc, mem_bmp);

            // Background
            let bg_brush = CreateSolidBrush(rgb(251, 243, 220));
            FillRect(mem_dc, &client_rc, bg_brush);
            DeleteObject(bg_brush);

            SetBkMode(mem_dc, TRANSPARENT);

            // Title
            let title_font = CreateFontW(
                -16,
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
                to_wide("Nirmala UI").as_ptr(),
            );
            let old_font = SelectObject(mem_dc, title_font);
            SetTextColor(mem_dc, rgb(6, 106, 74));
            let mut t_rect = RECT {
                left: 24,
                top: 18,
                right: 456,
                bottom: 44,
            };
            DrawTextW(
                mem_dc,
                to_wide("বর্ণমালা আপডেট ডাউনলোড হচ্ছে...").as_ptr(),
                -1,
                &mut t_rect,
                DT_LEFT | DT_SINGLELINE,
            );

            // Progress Bar Track
            let track_brush = CreateSolidBrush(rgb(230, 220, 195));
            let border_pen = CreatePen(PS_SOLID, 1, rgb(6, 106, 74));
            let old_pen = SelectObject(mem_dc, border_pen);
            let old_track_brush = SelectObject(mem_dc, track_brush);
            RoundRect(mem_dc, 24, 60, 456, 84, 10, 10);

            // Progress Bar Fill
            let pct = DL_PROGRESS_PCT.load(Ordering::Relaxed).min(100);
            let fill_width = ((456 - 24) * (pct as i32)) / 100;
            if fill_width > 0 {
                let fill_brush = CreateSolidBrush(rgb(6, 106, 74));
                SelectObject(mem_dc, fill_brush);
                RoundRect(mem_dc, 24, 60, 24 + fill_width, 84, 10, 10);
                DeleteObject(fill_brush);
            }

            SelectObject(mem_dc, old_track_brush);
            SelectObject(mem_dc, old_pen);
            DeleteObject(track_brush);
            DeleteObject(border_pen);

            // Stats: Left = Downloaded / Total (Pct), Right = Live Speed
            let stats_font = CreateFontW(
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
                to_wide("Nirmala UI").as_ptr(),
            );
            SelectObject(mem_dc, stats_font);
            SetTextColor(mem_dc, rgb(30, 50, 40));

            let down_kb = DL_DOWNLOADED_KB.load(Ordering::Relaxed);
            let tot_kb = DL_TOTAL_KB.load(Ordering::Relaxed);
            let speed_kb = DL_SPEED_KB.load(Ordering::Relaxed);

            let stats_left = format!(
                "{:.1} MB / {:.1} MB ({}%)",
                (down_kb as f64) / 1024.0,
                (tot_kb as f64) / 1024.0,
                pct
            );
            let stats_right = if speed_kb >= 1024 {
                format!("{:.2} MB/s", (speed_kb as f64) / 1024.0)
            } else {
                format!("{} KB/s", speed_kb)
            };

            let mut sl_rect = RECT {
                left: 26,
                top: 94,
                right: 280,
                bottom: 120,
            };
            DrawTextW(
                mem_dc,
                to_wide(&stats_left).as_ptr(),
                -1,
                &mut sl_rect,
                DT_LEFT | DT_SINGLELINE,
            );

            let mut sr_rect = RECT {
                left: 280,
                top: 94,
                right: 454,
                bottom: 120,
            };
            DrawTextW(
                mem_dc,
                to_wide(&stats_right).as_ptr(),
                -1,
                &mut sr_rect,
                DT_RIGHT | DT_SINGLELINE,
            );

            // Status message
            let status_text = get_dl_status();
            let mut st_rect = RECT {
                left: 26,
                top: 126,
                right: 454,
                bottom: 154,
            };
            DrawTextW(
                mem_dc,
                to_wide(&status_text).as_ptr(),
                -1,
                &mut st_rect,
                DT_LEFT | DT_SINGLELINE,
            );

            // Copy to screen DC
            BitBlt(hdc, 0, 0, w, h, mem_dc, 0, 0, 0x00CC0020);

            SelectObject(mem_dc, old_font);
            SelectObject(mem_dc, old_bmp);
            DeleteObject(title_font);
            DeleteObject(stats_font);
            DeleteObject(mem_bmp);
            DeleteDC(mem_dc);

            EndPaint(hwnd, &ps);
            0
        }

        WM_DL_PROGRESS => {
            InvalidateRect(hwnd, std::ptr::null(), FALSE);
            0
        }

        WM_DL_FINISHED => {
            let code = wparam;
            match code {
                1 => {
                    // Succeeded!
                    ShowWindow(hwnd, SW_HIDE);
                    DestroyWindow(hwnd);

                    let target_info = DL_TARGET_INFO.lock().ok().and_then(|mut t| t.take());
                    if let Some((target_file, version)) = target_info {
                        if let Ok(current_exe) = std::env::current_exe() {
                            let runas = to_wide("runas");
                            let exe_wide = to_wide(&current_exe.to_string_lossy());
                            let params = format!(
                                "--update-wizard \"{}\" \"{}\"",
                                target_file.to_string_lossy(),
                                version
                            );
                            let params_wide = to_wide(&params);

                            ShellExecuteW(
                                0,
                                runas.as_ptr(),
                                exe_wide.as_ptr(),
                                params_wide.as_ptr(),
                                std::ptr::null(),
                                SW_SHOWNORMAL,
                            );
                        }
                    }
                }
                2 => {
                    // Verification failed
                    let err = get_dl_error();
                    MessageBoxW(
                        hwnd,
                        to_wide(&err).as_ptr(),
                        to_wide("নিরাপত্তা ত্রুটি").as_ptr(),
                        0x00000010,
                    );
                    DestroyWindow(hwnd);
                }
                3 => {
                    // Download error
                    let err = get_dl_error();
                    MessageBoxW(
                        hwnd,
                        to_wide(&err).as_ptr(),
                        to_wide("ডাউনলোড ত্রুটি").as_ptr(),
                        0x00000010,
                    );
                    DestroyWindow(hwnd);
                }
                _ => {
                    // Cancelled
                    DestroyWindow(hwnd);
                }
            }
            0
        }

        WM_COMMAND => {
            let id = wparam & 0xFFFF;
            if id == BTN_CANCEL_DOWNLOAD {
                if let Ok(lock) = DL_CANCEL_FLAG.lock() {
                    if let Some(ref flag) = *lock {
                        flag.store(true, Ordering::SeqCst);
                    }
                }
                DestroyWindow(hwnd);
            }
            0
        }

        WM_CLOSE => {
            if let Ok(lock) = DL_CANCEL_FLAG.lock() {
                if let Some(ref flag) = *lock {
                    flag.store(true, Ordering::SeqCst);
                }
            }
            DestroyWindow(hwnd);
            0
        }

        WM_DESTROY => {
            DL_HWND = 0;
            PostQuitMessage(0);
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// ----------------------------------------------------------------------------
// 3. Elevated Update Wizard Window (UpdateWizardWindow)
// ----------------------------------------------------------------------------

const BTN_WIZARD_UPDATE: usize = 3001;
const BTN_WIZARD_CANCEL: usize = 3002;

static mut WIZARD_HWND: HWND = 0;
static WIZARD_PAYLOAD: Mutex<String> = Mutex::new(String::new());
static WIZARD_VERSION: Mutex<String> = Mutex::new(String::new());
static WIZARD_UPDATING: AtomicBool = AtomicBool::new(false);

pub unsafe fn run_update_wizard(payload_path: &str, target_version: &str) {
    if let Ok(mut lock) = WIZARD_PAYLOAD.lock() {
        *lock = payload_path.to_string();
    }
    if let Ok(mut lock) = WIZARD_VERSION.lock() {
        *lock = target_version.to_string();
    }
    WIZARD_UPDATING.store(false, Ordering::SeqCst);

    let hinst = GetModuleHandleW(std::ptr::null());
    let class_name = to_wide("BornomalaUpdateWizardClass");

    let mut wc: WNDCLASSEXW = std::mem::zeroed();
    wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
    wc.lpfnWndProc = Some(wizard_wnd_proc);
    wc.hInstance = hinst;
    wc.lpszClassName = class_name.as_ptr();
    wc.hbrBackground = CreateSolidBrush(rgb(251, 243, 220));
    wc.hCursor = LoadCursorW(0, 32512 as *const u16);
    RegisterClassExW(&wc);

    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let screen_h = GetSystemMetrics(SM_CYSCREEN);
    let width = 500;
    let height = 280;
    let x = (screen_w - width) / 2;
    let y = (screen_h - height) / 2;

    let title = to_wide("বর্ণমালা আপডেট উইজার্ড (Bornomala Update Wizard)");
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

    WIZARD_HWND = hwnd;

    let btn_font = CreateFontW(
        -14,
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
        to_wide("Nirmala UI").as_ptr(),
    );

    // Update button
    let btn_up = CreateWindowExW(
        0,
        to_wide("BUTTON").as_ptr(),
        to_wide("আপডেট করুন (Update)").as_ptr(),
        WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON,
        180,
        190,
        180,
        42,
        hwnd,
        BTN_WIZARD_UPDATE,
        hinst,
        std::ptr::null_mut(),
    );
    SendMessageW(btn_up, WM_SETFONT, btn_font, 1);

    // Cancel button
    let btn_can = CreateWindowExW(
        0,
        to_wide("BUTTON").as_ptr(),
        to_wide("বাতিল (Cancel)").as_ptr(),
        WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
        372,
        190,
        94,
        42,
        hwnd,
        BTN_WIZARD_CANCEL,
        hinst,
        std::ptr::null_mut(),
    );
    SendMessageW(btn_can, WM_SETFONT, btn_font, 1);

    ShowWindow(hwnd, SW_SHOW);
    SetForegroundWindow(hwnd);

    let mut msg: MSG = std::mem::zeroed();
    while GetMessageW(&mut msg, 0, 0, 0) > 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}

unsafe extern "system" fn wizard_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            // Header Banner
            let header_brush = CreateSolidBrush(rgb(6, 106, 74));
            let header_rect = RECT {
                left: 0,
                top: 0,
                right: 500,
                bottom: 64,
            };
            FillRect(hdc, &header_rect, header_brush);
            DeleteObject(header_brush);

            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, rgb(254, 247, 222));

            let h_font = CreateFontW(
                -18,
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
                to_wide("Nirmala UI").as_ptr(),
            );
            let old_font = SelectObject(hdc, h_font);
            let mut ht_rect = RECT {
                left: 24,
                top: 18,
                right: 480,
                bottom: 50,
            };
            DrawTextW(
                hdc,
                to_wide("বর্ণমালা আপডেট করতে প্রস্তুত (Ready to Update)").as_ptr(),
                -1,
                &mut ht_rect,
                DT_LEFT | DT_SINGLELINE,
            );

            // Body Area
            SetTextColor(hdc, rgb(6, 106, 74));
            let b_font = CreateFontW(
                -14,
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
                to_wide("Nirmala UI").as_ptr(),
            );
            SelectObject(hdc, b_font);

            let cur_target_ver = WIZARD_VERSION.lock().map(|v| v.clone()).unwrap_or_default();
            let trans_text = format!(
                "হালনাগাদ সংস্করণ: v{} (এক-ক্লিকে স্বয়ংক্রিয় আপডেট)",
                cur_target_ver
            );
            let mut v_rect = RECT {
                left: 24,
                top: 80,
                right: 480,
                bottom: 106,
            };
            DrawTextW(
                hdc,
                to_wide(&trans_text).as_ptr(),
                -1,
                &mut v_rect,
                DT_LEFT | DT_SINGLELINE,
            );

            let note_font = CreateFontW(
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
                to_wide("Nirmala UI").as_ptr(),
            );
            SelectObject(hdc, note_font);
            SetTextColor(hdc, rgb(40, 50, 45));

            let is_updating = WIZARD_UPDATING.load(Ordering::SeqCst);
            let instructions = if is_updating {
                "বর্ণমালা আপডেট প্রয়োগ করা হচ্ছে... দয়া করে অপেক্ষা করুন।"
            } else {
                "আপডেট প্যাকেজটি সফলভাবে ডাউনলোড ও ডিজিটাল স্বাক্ষর যাচাই করা হয়েছে।\n'আপডেট করুন' বাটনে ক্লিক করলে বর্ণমালা স্বয়ংক্রিয়ভাবে আপডেট হয়ে পুনরায় চালু হবে।"
            };

            let mut i_rect = RECT {
                left: 24,
                top: 114,
                right: 476,
                bottom: 180,
            };
            DrawTextW(
                hdc,
                to_wide(instructions).as_ptr(),
                -1,
                &mut i_rect,
                DT_LEFT | DT_WORDBREAK,
            );

            SelectObject(hdc, old_font);
            DeleteObject(h_font);
            DeleteObject(b_font);
            DeleteObject(note_font);

            EndPaint(hwnd, &ps);
            0
        }

        WM_COMMAND => {
            let id = wparam & 0xFFFF;
            if id == BTN_WIZARD_UPDATE && !WIZARD_UPDATING.load(Ordering::SeqCst) {
                WIZARD_UPDATING.store(true, Ordering::SeqCst);
                InvalidateRect(hwnd, std::ptr::null(), TRUE);

                // Disable buttons
                let btn_up = GetDlgItem(hwnd, BTN_WIZARD_UPDATE as i32);
                EnableWindow(btn_up, FALSE);
                let btn_can = GetDlgItem(hwnd, BTN_WIZARD_CANCEL as i32);
                EnableWindow(btn_can, FALSE);

                let payload = WIZARD_PAYLOAD.lock().map(|p| p.clone()).unwrap_or_default();

                std::thread::spawn(move || {
                    // 1. Close existing running instance of bornomala.exe safely via Win32 API
                    unsafe {
                        let main_cls = to_wide("BornomalaAppClass");
                        let main_h = FindWindowW(main_cls.as_ptr(), std::ptr::null());
                        if main_h != 0 {
                            SendMessageW(main_h, WM_CLOSE, 0, 0);
                            let mut pid: u32 = 0;
                            GetWindowThreadProcessId(main_h, &mut pid);
                            if pid != 0 && pid != std::process::id() {
                                let hproc = OpenProcess(0x0001, 0, pid);
                                if hproc != 0 {
                                    TerminateProcess(hproc, 0);
                                    CloseHandle(hproc);
                                }
                            }
                        }

                        let tb_cls = to_wide("BornomalaToolbarClass");
                        let tb_h = FindWindowW(tb_cls.as_ptr(), std::ptr::null());
                        if tb_h != 0 {
                            SendMessageW(tb_h, WM_CLOSE, 0, 0);
                            let mut pid: u32 = 0;
                            GetWindowThreadProcessId(tb_h, &mut pid);
                            if pid != 0 && pid != std::process::id() {
                                let hproc = OpenProcess(0x0001, 0, pid);
                                if hproc != 0 {
                                    TerminateProcess(hproc, 0);
                                    CloseHandle(hproc);
                                }
                            }
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_millis(600));

                    // 2. Execute verified setup with /S (Silent update - no manual setup required)
                    let _status = std::process::Command::new(&payload).arg("/S").status();

                    // 3. Restart the newly updated Bornomala app
                    let install_dir = r"C:\Program Files\Bornomala\bornomala.exe";
                    let target_exe = if Path::new(install_dir).exists() {
                        install_dir.to_string()
                    } else if let Ok(current) = std::env::current_exe() {
                        current.to_string_lossy().to_string()
                    } else {
                        install_dir.to_string()
                    };

                    let _ = std::process::Command::new(&target_exe).spawn();

                    // 4. Exit wizard
                    std::process::exit(0);
                });
            } else if id == BTN_WIZARD_CANCEL && !WIZARD_UPDATING.load(Ordering::SeqCst) {
                DestroyWindow(hwnd);
                PostQuitMessage(0);
            }
            0
        }

        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
