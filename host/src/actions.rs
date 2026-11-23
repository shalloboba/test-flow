use radial_core::Item;
use std::mem::{size_of, zeroed};
use std::ptr::null;
use windows_sys::Win32::System::Shutdown::LockWorkStation;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

pub const MAGIC: usize = 0x52444D4E;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn run(item: &Item) {
    match item.kind.as_str() {
        "keys" => keys(&item.value),
        "run" => launch(&item.value),
        "url" => open(&item.value),
        _ => {}
    }
}

fn open(target: &str) {
    let t = target.trim();
    if t.is_empty() {
        return;
    }
    let verb = wide("open");
    let file = wide(t);
    unsafe {
        ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), file.as_ptr(), null(), null(), SW_SHOWNORMAL);
    }
}

fn launch(cmd: &str) {
    let c = cmd.trim();
    if c.is_empty() {
        return;
    }
    let (file, args) = if std::path::Path::new(c).exists() {
        (c.to_string(), String::new())
    } else if let Some(rest) = c.strip_prefix('"') {
        match rest.find('"') {
            Some(end) => (rest[..end].to_string(), rest[end + 1..].trim().to_string()),
            None => (rest.to_string(), String::new()),
        }
    } else {
        match c.split_once(char::is_whitespace) {
            Some((f, a)) => (f.to_string(), a.trim().to_string()),
            None => (c.to_string(), String::new()),
        }
    };
    let verb = wide("open");
    let f = wide(&file);
    let a = wide(&args);
    unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            f.as_ptr(),
            if args.is_empty() { null() } else { a.as_ptr() },
            null(),
            SW_SHOWNORMAL,
        );
    }
}

fn vk_of(token: &str) -> Option<(u16, bool)> {
    let t = token.trim().to_lowercase();
    let named = match t.as_str() {
        "ctrl" | "control" => (VK_CONTROL, false),
        "alt" => (VK_MENU, false),
        "shift" => (VK_SHIFT, false),
        "win" | "windows" | "super" | "meta" => (VK_LWIN, false),
        "enter" | "return" => (VK_RETURN, false),
        "esc" | "escape" => (VK_ESCAPE, false),
        "tab" => (VK_TAB, false),
        "space" => (VK_SPACE, false),
        "backspace" => (VK_BACK, false),
        "delete" | "del" => (VK_DELETE, true),
        "insert" | "ins" => (VK_INSERT, true),
        "home" => (VK_HOME, true),
        "end" => (VK_END, true),
        "pageup" | "pgup" => (VK_PRIOR, true),
        "pagedown" | "pgdn" => (VK_NEXT, true),
        "up" => (VK_UP, true),
        "down" => (VK_DOWN, true),
        "left" => (VK_LEFT, true),
        "right" => (VK_RIGHT, true),
        "printscreen" | "prtsc" | "prtscn" => (VK_SNAPSHOT, false),
        "playpause" => (VK_MEDIA_PLAY_PAUSE, false),
        "nexttrack" | "next" => (VK_MEDIA_NEXT_TRACK, false),
        "prevtrack" | "prev" => (VK_MEDIA_PREV_TRACK, false),
        "volumeup" => (VK_VOLUME_UP, false),
        "volumedown" => (VK_VOLUME_DOWN, false),
        "mute" => (VK_VOLUME_MUTE, false),
        "plus" => (VK_OEM_PLUS, false),
        "minus" => (VK_OEM_MINUS, false),
        "comma" => (VK_OEM_COMMA, false),
        "period" => (VK_OEM_PERIOD, false),
        _ => (0, false),
    };
    if named.0 != 0 {
        return Some(named);
    }
    if let Some(n) = t.strip_prefix('f').and_then(|x| x.parse::<u16>().ok()) {
        if (1..=24).contains(&n) {
            return Some((VK_F1 + n - 1, false));
        }
    }
    let mut it = t.chars();
    let c = it.next()?;
    if it.next().is_none() && c.is_ascii_alphanumeric() {
        return Some((c.to_ascii_uppercase() as u16, false));
    }
    None
}

fn key_input(vk: u16, up: bool, ext: bool) -> INPUT {
    let mut flags = 0;
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    if ext {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    let mut i: INPUT = unsafe { zeroed() };
    i.r#type = INPUT_KEYBOARD;
    i.Anonymous.ki = KEYBDINPUT {
        wVk: vk,
        wScan: 0,
        dwFlags: flags,
        time: 0,
        dwExtraInfo: MAGIC,
    };
    i
}

fn send(inputs: &[INPUT]) {
    if inputs.is_empty() {
        return;
    }
    unsafe {
        SendInput(inputs.len() as u32, inputs.as_ptr(), size_of::<INPUT>() as i32);
    }
}

pub fn keys(spec: &str) {
    let norm: String = spec.split('+').map(|s| s.trim().to_lowercase()).collect::<Vec<_>>().join("+");
    if norm == "win+l" {
        unsafe {
            LockWorkStation();
        }
        return;
    }
    let parts: Vec<(u16, bool)> = spec.split('+').filter_map(vk_of).collect();
    if parts.is_empty() {
        return;
    }
    let mut seq = Vec::new();
    for &(vk, ext) in &parts {
        seq.push(key_input(vk, false, ext));
    }
    for &(vk, ext) in parts.iter().rev() {
        seq.push(key_input(vk, true, ext));
    }
    send(&seq);
}

/// 短い押しだったときに、本来のクリックを再生する。bit: 1=中, 2=右, 4=X1, 8=X2。
pub fn replay_click(bit: u32) {
    let (down_flag, up_flag, data) = match bit {
        2 => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, 0),
        4 => (MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, 1),
        8 => (MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, 2),
        _ => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, 0),
    };
    let mut down: INPUT = unsafe { zeroed() };
    down.r#type = INPUT_MOUSE;
    down.Anonymous.mi = MOUSEINPUT {
        dx: 0,
        dy: 0,
        mouseData: data,
        dwFlags: down_flag,
        time: 0,
        dwExtraInfo: MAGIC,
    };
    let mut up = down;
    up.Anonymous.mi.dwFlags = up_flag;
    send(&[down, up]);
}
