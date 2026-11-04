//! System Tray Integration for Bornomala.
//! Adds a tray notification icon, mode status tooltip, and right-click menu
//! for seamless background operation.

use super::api::*;
use super::toolbar::{FloatingToolbar, CURRENT_SCHEME};
use bornomala_engine::InputScheme;
use std::sync::atomic::Ordering;

pub const ID_TRAY_ICON: u32 = 1001;

// Menu Command IDs
const ID_MENU_HEADER: usize = 2000;
const ID_MENU_UNICODE: usize = 2001;
const ID_MENU_ANSI: usize = 2002;
const ID_MENU_PHONETIC: usize = 2003;
const ID_MENU_ENGLISH: usize = 2004;
const ID_MENU_CONVERTER: usize = 2005;
const ID_MENU_TOOLBAR: usize = 2006;
const ID_MENU_GITHUB: usize = 2007;
const ID_MENU_ABOUT: usize = 2008;
const ID_MENU_UPDATE: usize = 2010;
const ID_MENU_SETTINGS: usize = 2011;
const ID_MENU_EXIT: usize = 2009;

#[allow(dead_code)]
pub struct SystemTray {
    pub hwnd: HWND,
    nid: NOTIFYICONDATAW,
}

impl SystemTray {
    pub fn new(hwnd: HWND, hinstance: HINSTANCE) -> Self {
        unsafe {
            let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = ID_TRAY_ICON;
            nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
            nid.uCallbackMessage = WM_BORNOMALA_TRAY;

            // Load native embedded icon resource (104: English by default)
            let mut hicon = LoadIconW(hinstance, 104 as *const u16);
            if hicon == 0 {
                hicon = super::assets::get_tray_hicon(3);
            }
            if hicon == 0 {
                hicon = LoadIconW(0, 32512 as *const u16);
            }
            nid.hIcon = hicon;

            let tip = to_wide("Bornomala (বর্ণমালা) — English (Pass-through)");
            for (i, &ch) in tip.iter().take(127).enumerate() {
                nid.szTip[i] = ch;
            }

            Shell_NotifyIconW(NIM_ADD, &nid);

            Self { hwnd, nid }
        }
    }

    pub fn update_tooltip(&mut self, scheme: InputScheme) {
        let (scheme_str, res_id, mode_idx) = match scheme {
            InputScheme::ClassicUnicode => ("ইউনিকোড (Unicode)", 101, 0),
            InputScheme::ClassicAnsi => ("বিজয় (SutonnyMJ ANSI)", 102, 1),
            InputScheme::Phonetic => ("ফোনেটিক (Avro Phonetic)", 103, 2),
            InputScheme::PassThrough => ("English (Pass-through)", 104, 3),
        };
        let tip_text = format!("Bornomala (বর্ণমালা) — {}", scheme_str);
        let wide = to_wide(&tip_text);

        for c in self.nid.szTip.iter_mut() {
            *c = 0;
        }
        for (i, &ch) in wide.iter().take(127).enumerate() {
            self.nid.szTip[i] = ch;
        }

        unsafe {
            let hinst = GetModuleHandleW(std::ptr::null());
            let mut hicon = LoadIconW(hinst, res_id as *const u16);
            if hicon == 0 {
                hicon = super::assets::get_tray_hicon(mode_idx);
            }
            if hicon != 0 {
                self.nid.hIcon = hicon;
            }
            Shell_NotifyIconW(NIM_MODIFY, &self.nid);
        }
    }

    pub fn notify_minimized(&mut self) {
        let title = to_wide("বর্ণমালা (Bornomala)");
        let info = to_wide("টুলবার সিস্টেম ট্রে-তে মিনিমাইজ করা হয়েছে। পুনরায় দেখতে এখানে বা ট্রে আইকনে ক্লিক করুন।");

        for c in self.nid.szInfoTitle.iter_mut() {
            *c = 0;
        }
        for (i, &ch) in title.iter().take(63).enumerate() {
            self.nid.szInfoTitle[i] = ch;
        }

        for c in self.nid.szInfo.iter_mut() {
            *c = 0;
        }
        for (i, &ch) in info.iter().take(255).enumerate() {
            self.nid.szInfo[i] = ch;
        }

        self.nid.uFlags |= NIF_INFO;
        self.nid.dwInfoFlags = NIIF_INFO;

        unsafe {
            Shell_NotifyIconW(NIM_MODIFY, &self.nid);
            // Clear info flag so future updates don't keep popping balloon
            self.nid.uFlags &= !NIF_INFO;
        }
    }

    pub fn handle_tray_message(hwnd: HWND, lparam: LPARAM) {
        let event = lparam as u32;
        unsafe {
            match event {
                WM_LBUTTONUP | WM_LBUTTONDOWN => {
                    FloatingToolbar::toggle_visibility();
                }

                WM_RBUTTONUP => {
                    let mut pt: POINT = std::mem::zeroed();
                    GetCursorPos(&mut pt);

                    let hmenu = CreatePopupMenu();
                    let current = CURRENT_SCHEME.load(Ordering::SeqCst);

                    let add_item = |menu: HMENU, id: usize, text: &str, checked: bool, disabled: bool| {
                        let mut flags = MF_STRING;
                        if checked {
                            flags |= MF_CHECKED;
                        }
                        if disabled {
                            flags |= MF_GRAYED;
                        }
                        let wide = to_wide(text);
                        AppendMenuW(menu, flags, id, wide.as_ptr());
                    };

                    let add_sep = |menu: HMENU| {
                        AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
                    };

                    add_item(hmenu, ID_MENU_HEADER, "Bornomala (বর্ণমালা) v0.1.0", false, true);
                    add_sep(hmenu);

                    add_item(hmenu, ID_MENU_UNICODE, "  ইউনিকোড মোড (Ctrl+Alt+V)", current == 0, false);
                    add_item(hmenu, ID_MENU_ANSI, "  বিজয় এএনএসআই মোড (Ctrl+Alt+B)", current == 1, false);
                    add_item(hmenu, ID_MENU_PHONETIC, "  ফোনেটিক মোড (F12)", current == 2, false);
                    add_item(hmenu, ID_MENU_ENGLISH, "  ইংরেজি মোড (Ctrl+Alt+E)", current == 3, false);
                    add_sep(hmenu);

                    add_item(hmenu, ID_MENU_TOOLBAR, "📌 ফ্লোটিং বার দেখান/লুকান (Toggle Bar)", false, false);
                    add_item(hmenu, ID_MENU_CONVERTER, "🔄 বাংলা রূপান্তরকারী (Converter)...", false, false);
                    add_item(hmenu, ID_MENU_SETTINGS, "⚙ কনফিগারেশন ও সেটিংস (Settings)...", false, false);
                    add_sep(hmenu);

                    add_item(hmenu, ID_MENU_GITHUB, "🌐 সোর্স কোড ও ডকুমেন্টেশন (GitHub)", false, false);
                    add_item(hmenu, ID_MENU_ABOUT, "ℹ বর্ণমালা পরিচিতি (About)", false, false);
                    add_item(hmenu, ID_MENU_UPDATE, "🔄 আপডেট পরীক্ষা করুন (Check for Updates)...", false, false);
                    add_sep(hmenu);

                    add_item(hmenu, ID_MENU_EXIT, "❌ সম্পূর্ণ প্রস্থান করুন (Exit)", false, false);

                    SetForegroundWindow(hwnd);
                    TrackPopupMenu(hmenu, TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, std::ptr::null());
                    DestroyMenu(hmenu);
                }

                _ => {}
            }
        }
    }

    pub fn handle_menu_command(id: usize) {
        match id {
            ID_MENU_UNICODE => {
                FloatingToolbar::set_scheme(InputScheme::ClassicUnicode);
            }
            ID_MENU_ANSI => {
                FloatingToolbar::set_scheme(InputScheme::ClassicAnsi);
            }
            ID_MENU_PHONETIC => {
                FloatingToolbar::set_scheme(InputScheme::Phonetic);
            }
            ID_MENU_ENGLISH => {
                FloatingToolbar::set_scheme(InputScheme::PassThrough);
            }
            ID_MENU_TOOLBAR => {
                FloatingToolbar::toggle_visibility();
            }
            ID_MENU_CONVERTER => {
                super::converter::ConverterWindow::open();
            }
            ID_MENU_SETTINGS => {
                super::settings::SettingsWindow::open();
            }
            ID_MENU_GITHUB => {
                unsafe {
                    let open_op = to_wide("open");
                    let url = to_wide("https://github.com/RuposhiOpenLabs/bornomala");
                    ShellExecuteW(0, open_op.as_ptr(), url.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL);
                }
            }
            ID_MENU_ABOUT => {
                super::about::AboutWindow::open();
            }
            ID_MENU_UPDATE => {
                super::updater::trigger_manual_update_check("0.1.0");
            }
            ID_MENU_EXIT => {
                unsafe {
                    PostQuitMessage(0);
                }
            }
            _ => {}
        }
    }
}

impl Drop for SystemTray {
    fn drop(&mut self) {
        unsafe {
            Shell_NotifyIconW(NIM_DELETE, &self.nid);
        }
    }
}
