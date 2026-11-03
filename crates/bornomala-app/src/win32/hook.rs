//! Global Keyboard Interception & Hotkeys for Bornomala.
//! Registers F12, Ctrl+Alt+V, Ctrl+Alt+B, Ctrl+Alt+E and installs a low-level
//! keyboard hook (WH_KEYBOARD_LL) to drive the Bornomala typing engine into any application.

use super::api::*;
use super::toolbar::FloatingToolbar;
use bornomala_engine::{BornomalaEngine, InputScheme, KeyEvent, TextEdit};
use std::sync::Mutex;

const ID_HOTKEY_F12: i32 = 1;
const ID_HOTKEY_UNICODE: i32 = 2;
const ID_HOTKEY_ANSI: i32 = 3;
const ID_HOTKEY_ENGLISH: i32 = 4;
const ID_HOTKEY_CTRL_SPACE: i32 = 5;

static mut HOOK_HANDLE: HHOOK = 0;
static mut GLOBAL_ENGINE: Option<Mutex<BornomalaEngine>> = None;

#[link(name = "user32")]
extern "system" {
    fn GetKeyState(nVirtKey: i32) -> i16;
}

pub struct KeyboardHook;

impl KeyboardHook {
    pub fn init(hwnd: HWND, hinstance: HINSTANCE) {
        unsafe {
            GLOBAL_ENGINE = Some(Mutex::new(BornomalaEngine::new(InputScheme::ClassicUnicode)));

            // Register Global Mode Switch Hotkeys
            // F12 & Ctrl + Space: Phonetic Toggle
            RegisterHotKey(hwnd, ID_HOTKEY_F12, 0, VK_F12);
            RegisterHotKey(hwnd, ID_HOTKEY_CTRL_SPACE, MOD_CONTROL, VK_SPACE);

            // Ctrl + Alt + V: Classic Unicode
            RegisterHotKey(hwnd, ID_HOTKEY_UNICODE, MOD_CONTROL | MOD_ALT, 'V' as u32);

            // Ctrl + Alt + B: Classic SutonnyMJ ANSI
            RegisterHotKey(hwnd, ID_HOTKEY_ANSI, MOD_CONTROL | MOD_ALT, 'B' as u32);

            // Ctrl + Alt + E: English
            RegisterHotKey(hwnd, ID_HOTKEY_ENGLISH, MOD_CONTROL | MOD_ALT, 'E' as u32);

            // Install low-level keyboard hook
            HOOK_HANDLE = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                low_level_keyboard_proc,
                hinstance,
                0,
            );
        }
    }

    pub fn handle_hotkey(id: i32) {
        match id {
            ID_HOTKEY_F12 | ID_HOTKEY_CTRL_SPACE => {
                let cur = FloatingToolbar::get_scheme();
                if cur == InputScheme::Phonetic {
                    FloatingToolbar::set_scheme(InputScheme::PassThrough);
                } else {
                    FloatingToolbar::set_scheme(InputScheme::Phonetic);
                }
            }
            ID_HOTKEY_UNICODE => {
                FloatingToolbar::set_scheme(InputScheme::ClassicUnicode);
            }
            ID_HOTKEY_ANSI => {
                FloatingToolbar::set_scheme(InputScheme::ClassicAnsi);
            }
            ID_HOTKEY_ENGLISH => {
                FloatingToolbar::set_scheme(InputScheme::PassThrough);
            }
            _ => {}
        }
    }

    pub fn cleanup(hwnd: HWND) {
        unsafe {
            if HOOK_HANDLE != 0 {
                UnhookWindowsHookEx(HOOK_HANDLE);
                HOOK_HANDLE = 0;
            }
            UnregisterHotKey(hwnd, ID_HOTKEY_F12);
            UnregisterHotKey(hwnd, ID_HOTKEY_CTRL_SPACE);
            UnregisterHotKey(hwnd, ID_HOTKEY_UNICODE);
            UnregisterHotKey(hwnd, ID_HOTKEY_ANSI);
            UnregisterHotKey(hwnd, ID_HOTKEY_ENGLISH);
        }
    }
}

unsafe extern "system" fn low_level_keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(HOOK_HANDLE, code, wparam, lparam);
    }

    let scheme = FloatingToolbar::get_scheme();
    if scheme == InputScheme::PassThrough {
        return CallNextHookEx(HOOK_HANDLE, code, wparam, lparam);
    }

    let kbd = &*(lparam as *const KBDLLHOOKSTRUCT);
    let msg = wparam as u32;

    // Check if injected by ourselves (LLKHF_INJECTED = 0x00000010)
    if (kbd.flags & 0x00000010) != 0 {
        return CallNextHookEx(HOOK_HANDLE, code, wparam, lparam);
    }

    // Pass through control combos like Ctrl+C, Ctrl+V, Alt+Tab
    let ctrl_down = (GetKeyState(VK_CONTROL as i32) as i16) < 0;
    let alt_down = (GetKeyState(VK_MENU as i32) as i16) < 0;
    if ctrl_down || alt_down {
        return CallNextHookEx(HOOK_HANDLE, code, wparam, lparam);
    }

    if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
        let shift_down = (GetKeyState(VK_SHIFT as i32) as i16) < 0;
        let vk = kbd.vkCode;

        // Skip modifiers
        if vk == VK_SHIFT || vk == VK_CONTROL || vk == VK_MENU {
            return CallNextHookEx(HOOK_HANDLE, code, wparam, lparam);
        }

        // Map VK to char
        let ch = match vk {
            0x41..=0x5A => { // A-Z
                let base = if shift_down { 'A' } else { 'a' };
                let offset = (vk - 0x41) as u8;
                Some((base as u8 + offset) as char)
            }
            0x30..=0x39 => { // 0-9
                let c = if shift_down {
                    match vk {
                        0x31 => '!', 0x32 => '@', 0x33 => '#', 0x34 => '$', 0x35 => '%',
                        0x36 => '^', 0x37 => '&', 0x38 => '*', 0x39 => '(', 0x30 => ')',
                        _ => '0',
                    }
                } else {
                    let offset = (vk - 0x30) as u8;
                    (b'0' + offset) as char
                };
                Some(c)
            }
            VK_SPACE => Some(' '),
            VK_BACK => Some('\x08'),
            VK_RETURN => Some('\n'),
            0xBA => Some(if shift_down { ':' } else { ';' }),
            0xBB => Some(if shift_down { '+' } else { '=' }),
            0xBC => Some(if shift_down { '<' } else { ',' }),
            0xBD => Some(if shift_down { '_' } else { '-' }),
            0xBE => Some(if shift_down { '>' } else { '.' }),
            0xBF => Some(if shift_down { '?' } else { '/' }),
            0xC0 => Some(if shift_down { '~' } else { '`' }),
            0xDB => Some(if shift_down { '{' } else { '[' }),
            0xDC => Some(if shift_down { '|' } else { '\\' }),
            0xDD => Some(if shift_down { '}' } else { ']' }),
            0xDE => Some(if shift_down { '"' } else { '\'' }),
            _ => None,
        };

        if let Some(ref engine_mutex) = GLOBAL_ENGINE {
            if let Ok(mut engine) = engine_mutex.lock() {
                if engine.active_scheme() != scheme {
                    engine.set_scheme(scheme);
                }

                let event = KeyEvent::new_down(vk, ch, shift_down, ctrl_down, alt_down);
                let action = engine.process_key(&event);

                let mut handled_edits = false;
                for edit in &action.edits {
                    match edit {
                        TextEdit::Insert(text) => {
                            inject_unicode_text(text);
                            handled_edits = true;
                        }
                        TextEdit::ReplaceLast { count, text } => {
                            inject_backspaces(*count);
                            inject_unicode_text(text);
                            handled_edits = true;
                        }
                        TextEdit::DeleteBack(count) => {
                            inject_backspaces(*count);
                            handled_edits = true;
                        }
                        TextEdit::PassThrough => {}
                    }
                }

                if handled_edits || action.consumed {
                    return 1; // Intercept and block original ASCII key
                }
            }
        }
    }

    CallNextHookEx(HOOK_HANDLE, code, wparam, lparam)
}

unsafe fn inject_backspaces(count: usize) {
    if count == 0 {
        return;
    }
    let mut inputs: Vec<INPUT> = Vec::with_capacity(count * 2);
    for _ in 0..count {
        let mut in_down: INPUT = std::mem::zeroed();
        in_down.input_type = INPUT_KEYBOARD;
        in_down.ki.wVk = VK_BACK as u16;
        inputs.push(in_down);

        let mut in_up: INPUT = std::mem::zeroed();
        in_up.input_type = INPUT_KEYBOARD;
        in_up.ki.wVk = VK_BACK as u16;
        in_up.ki.dwFlags = KEYEVENTF_KEYUP;
        inputs.push(in_up);
    }
    SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32);
}

unsafe fn inject_unicode_text(text: &str) {
    let wide: Vec<u16> = text.encode_utf16().collect();
    if wide.is_empty() {
        return;
    }

    let mut inputs: Vec<INPUT> = Vec::with_capacity(wide.len() * 2);
    for &code_unit in &wide {
        // Key down
        let mut in_down: INPUT = std::mem::zeroed();
        in_down.input_type = INPUT_KEYBOARD;
        in_down.ki.wScan = code_unit;
        in_down.ki.dwFlags = KEYEVENTF_UNICODE;
        inputs.push(in_down);

        // Key up
        let mut in_up: INPUT = std::mem::zeroed();
        in_up.input_type = INPUT_KEYBOARD;
        in_up.ki.wScan = code_unit;
        in_up.ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
        inputs.push(in_up);
    }

    SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32);
}
