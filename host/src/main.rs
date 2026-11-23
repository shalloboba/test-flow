#![windows_subsystem = "windows"]

mod actions;
mod fonts;
mod render;

use radial_core::{system, Config, Item, HOST_CLASS, MSG_RELOAD};
use std::cell::RefCell;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicIsize, AtomicU32, Ordering};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::HiDpi::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

const WM_DOWN: u32 = WM_APP + 10;
const WM_UP: u32 = WM_APP + 11;
const WM_MOVE_HOOK: u32 = WM_APP + 12;
const WM_LAYER: u32 = WM_APP + 13;
const WM_TRAY: u32 = WM_APP + 20;
const TIMER_HOLD: usize = 1;
const TIMER_ANIM: usize = 2;
const TIMER_TOP: usize = 3;
const ID_SETTINGS: usize = 1;
const ID_RELOAD: usize = 2;
const ID_QUIT: usize = 3;

static HOST: AtomicIsize = AtomicIsize::new(0);
static ENABLED: AtomicBool = AtomicBool::new(true);
static PRESSED: AtomicBool = AtomicBool::new(false);
static MENU_OPEN: AtomicBool = AtomicBool::new(false);
static MOVE_PENDING: AtomicBool = AtomicBool::new(false);
static PRESS_X: AtomicI32 = AtomicI32::new(0);
static PRESS_Y: AtomicI32 = AtomicI32::new(0);
static SWALLOWED_KEY: AtomicU32 = AtomicU32::new(0);
static LAYER_VK: [AtomicU32; 9] = [
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
];

fn apply_layer_keys(cfg: &Config) {
    let v = cfg.layer_vks();
    for (i, slot) in LAYER_VK.iter().enumerate() {
        slot.store(v.get(i).copied().unwrap_or(0), Ordering::Relaxed);
    }
}

enum Phase {
    Idle,
    Pending,
}

struct Open {
    geo: render::Geometry,
    theme: render::Theme,
    left: i32,
    top: i32,
    hover: Option<usize>,
    alpha: u8,
    layer: usize,
}

struct App {
    cfg: Config,
    fonts: fonts::Fonts,
    phase: Phase,
    open: Option<Open>,
    icon: HICON,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn host() -> HWND {
    HOST.load(Ordering::Relaxed) as HWND
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && ENABLED.load(Ordering::Relaxed) {
        let info = &*(lparam as *const MSLLHOOKSTRUCT);
        if info.dwExtraInfo != actions::MAGIC {
            match wparam as u32 {
                WM_MBUTTONDOWN => {
                    PRESSED.store(true, Ordering::Relaxed);
                    PRESS_X.store(info.pt.x, Ordering::Relaxed);
                    PRESS_Y.store(info.pt.y, Ordering::Relaxed);
                    PostMessageW(host(), WM_DOWN, 0, 0);
                    return 1;
                }
                WM_MBUTTONUP => {
                    if PRESSED.swap(false, Ordering::Relaxed) {
                        PostMessageW(host(), WM_UP, 0, 0);
                        return 1;
                    }
                }
                WM_MOUSEWHEEL => {
                    if MENU_OPEN.load(Ordering::Relaxed) {
                        let delta = (info.mouseData >> 16) as i16;
                        PostMessageW(host(), WM_LAYER, if delta < 0 { 100 } else { 101 }, 0);
                        return 1;
                    }
                }
                WM_MOUSEMOVE => {
                    if MENU_OPEN.load(Ordering::Relaxed) && !MOVE_PENDING.swap(true, Ordering::Relaxed) {
                        PostMessageW(host(), WM_MOVE_HOOK, 0, 0);
                    }
                }
                _ => {}
            }
        }
    }
    CallNextHookEx(null_mut(), code, wparam, lparam)
}

unsafe extern "system" fn key_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let k = &*(lparam as *const KBDLLHOOKSTRUCT);
        if k.dwExtraInfo != actions::MAGIC {
            let vk = k.vkCode;
            let msg = wparam as u32;
            let is_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;
            let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
            if is_up && SWALLOWED_KEY.load(Ordering::Relaxed) == vk {
                SWALLOWED_KEY.store(0, Ordering::Relaxed);
                return 1;
            }
            if is_down && MENU_OPEN.load(Ordering::Relaxed) {
                let shift = GetAsyncKeyState(0x10) < 0;
                let mut target = if vk == 0x09 { Some(if shift { 101 } else { 100 }) } else { None };
                if target.is_none() {
                    for (i, slot) in LAYER_VK.iter().enumerate() {
                        let v = slot.load(Ordering::Relaxed);
                        if v == 0 {
                            continue;
                        }
                        let numpad = if (0x30..=0x39).contains(&v) { v + 0x30 } else { 0 };
                        if vk == v || vk == numpad {
                            target = Some(i + 1);
                            break;
                        }
                    }
                }
                if let Some(t) = target {
                    if SWALLOWED_KEY.swap(vk, Ordering::Relaxed) != vk {
                        PostMessageW(host(), WM_LAYER, t, 0);
                    }
                    return 1;
                }
            }
        }
    }
    CallNextHookEx(null_mut(), code, wparam, lparam)
}

fn present(hwnd: HWND, pm: &tiny_skia::Pixmap, left: i32, top: i32, alpha: u8) {
    unsafe {
        let screen = GetDC(null_mut());
        let mem = CreateCompatibleDC(screen);
        let w = pm.width() as i32;
        let h = pm.height() as i32;
        let mut bi: BITMAPINFO = zeroed();
        bi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        bi.bmiHeader.biWidth = w;
        bi.bmiHeader.biHeight = -h;
        bi.bmiHeader.biPlanes = 1;
        bi.bmiHeader.biBitCount = 32;
        bi.bmiHeader.biCompression = BI_RGB;
        let mut bits: *mut core::ffi::c_void = null_mut();
        let dib = CreateDIBSection(mem, &bi, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
        if !dib.is_null() && !bits.is_null() {
            let dst = std::slice::from_raw_parts_mut(bits as *mut u8, (w * h * 4) as usize);
            for (d, s) in dst.chunks_exact_mut(4).zip(pm.data().chunks_exact(4)) {
                d[0] = s[2];
                d[1] = s[1];
                d[2] = s[0];
                d[3] = s[3];
            }
            let old = SelectObject(mem, dib);
            let pt_dst = POINT { x: left, y: top };
            let size = SIZE { cx: w, cy: h };
            let pt_src = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: alpha,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            UpdateLayeredWindow(hwnd, screen, &pt_dst, &size, mem, &pt_src, 0, &blend, ULW_ALPHA);
            SelectObject(mem, old);
        }
        if !dib.is_null() {
            DeleteObject(dib);
        }
        DeleteDC(mem);
        ReleaseDC(null_mut(), screen);
    }
}

fn layer_items(app: &App, layer: usize) -> &[Item] {
    let i = layer.min(app.cfg.layers.len() - 1);
    &app.cfg.layers[i].items
}

fn redraw(app: &App, hwnd: HWND) {
    if let Some(o) = &app.open {
        let li = o.layer.min(app.cfg.layers.len() - 1);
        let info = render::LayerInfo {
            name: &app.cfg.layers[li].name,
            index: li,
            count: app.cfg.layers.len(),
        };
        let pm = render::render(layer_items(app, o.layer), &info, &o.theme, &o.geo, &app.fonts, o.hover);
        present(hwnd, &pm, o.left, o.top, o.alpha);
    }
}

fn cursor() -> (i32, i32) {
    let mut p = POINT { x: 0, y: 0 };
    unsafe {
        GetCursorPos(&mut p);
    }
    (p.x, p.y)
}

fn hover_at(app: &App) -> Option<usize> {
    let o = app.open.as_ref()?;
    let (x, y) = cursor();
    let cx = o.left as f32 + o.geo.c;
    let cy = o.top as f32 + o.geo.c;
    render::hit(&o.geo, x as f32 - cx, y as f32 - cy, layer_items(app, o.layer).len())
}

fn open_menu(app: &mut App, hwnd: HWND, px: i32, py: i32) {
    unsafe {
        let mon = MonitorFromPoint(POINT { x: px, y: py }, MONITOR_DEFAULTTONEAREST);
        let mut dx = 96u32;
        let mut dy = 96u32;
        GetDpiForMonitor(mon, MDT_EFFECTIVE_DPI, &mut dx, &mut dy);
        let scale = dx as f32 / 96.0 * app.cfg.scale as f32 / 100.0;
        let geo = render::Geometry::new(scale);
        let mut mi: MONITORINFO = zeroed();
        mi.cbSize = size_of::<MONITORINFO>() as u32;
        GetMonitorInfoW(mon, &mut mi);
        let size = geo.size;
        let left = (px - size / 2).clamp(mi.rcMonitor.left, (mi.rcMonitor.right - size).max(mi.rcMonitor.left));
        let top = (py - size / 2).clamp(mi.rcMonitor.top, (mi.rcMonitor.bottom - size).max(mi.rcMonitor.top));
        let cx = left + size / 2;
        let cy = top + size / 2;
        if cx != px || cy != py {
            SetCursorPos(cx, cy);
        }
        app.open = Some(Open {
            geo,
            theme: render::Theme::resolve(&app.cfg),
            left,
            top,
            hover: None,
            alpha: 0,
            layer: 0,
        });
        redraw(app, hwnd);
        SetWindowPos(hwnd, HWND_TOPMOST, left, top, size, size, SWP_NOACTIVATE | SWP_SHOWWINDOW);
        SetTimer(hwnd, TIMER_TOP, 100, None);
        MENU_OPEN.store(true, Ordering::Relaxed);
        SetTimer(hwnd, TIMER_ANIM, 15, None);
    }
}

fn close_menu(app: &mut App, hwnd: HWND) {
    MENU_OPEN.store(false, Ordering::Relaxed);
    app.open = None;
    unsafe {
        KillTimer(hwnd, TIMER_ANIM);
        KillTimer(hwnd, TIMER_TOP);
        ShowWindow(hwnd, SW_HIDE);
    }
}

fn launch_settings() {
    let Ok(me) = std::env::current_exe() else { return };
    let Some(dir) = me.parent() else { return };
    let candidates = [
        dir.join("RadialMenu.Settings.exe"),
        dir.join("settings").join("RadialMenu.Settings.exe"),
    ];
    for exe in &candidates {
        if exe.exists() {
            let _ = std::process::Command::new(exe).current_dir(exe.parent().unwrap_or(dir)).spawn();
            return;
        }
    }
    show_error("設定アプリ (RadialMenu.Settings.exe) が見つかりません。\nRadialMenu.exe と同じフォルダーに置いてください。");
}

fn reload(app: &mut App, hwnd: HWND) {
    app.cfg = Config::load().for_host();
    apply_layer_keys(&app.cfg);
    ENABLED.store(app.cfg.enabled, Ordering::Relaxed);
    if app.open.is_some() {
        close_menu(app, hwnd);
    }
    set_tray_icon(app, hwnd, false);
}

fn make_icon(accent: [u8; 3]) -> HICON {
    unsafe {
        let pm = render::render_icon(accent);
        let mut bi: BITMAPINFO = zeroed();
        bi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        bi.bmiHeader.biWidth = 32;
        bi.bmiHeader.biHeight = -32;
        bi.bmiHeader.biPlanes = 1;
        bi.bmiHeader.biBitCount = 32;
        bi.bmiHeader.biCompression = BI_RGB;
        let screen = GetDC(null_mut());
        let mut bits: *mut core::ffi::c_void = null_mut();
        let color = CreateDIBSection(screen, &bi, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
        if !bits.is_null() {
            let dst = std::slice::from_raw_parts_mut(bits as *mut u8, 32 * 32 * 4);
            for (d, s) in dst.chunks_exact_mut(4).zip(pm.data().chunks_exact(4)) {
                d[0] = s[2];
                d[1] = s[1];
                d[2] = s[0];
                d[3] = s[3];
            }
        }
        let mask = CreateBitmap(32, 32, 1, 1, null());
        let info = ICONINFO {
            fIcon: 1,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: mask,
            hbmColor: color,
        };
        let icon = CreateIconIndirect(&info);
        DeleteObject(color);
        DeleteObject(mask);
        ReleaseDC(null_mut(), screen);
        icon
    }
}

fn set_tray_icon(app: &mut App, hwnd: HWND, add: bool) {
    unsafe {
        let accent = if app.cfg.accent == "auto" {
            system::system_accent()
        } else {
            system::parse_hex(&app.cfg.accent).unwrap_or_else(system::system_accent)
        };
        let old = app.icon;
        app.icon = make_icon(accent);
        let mut nid: NOTIFYICONDATAW = zeroed();
        nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.uCallbackMessage = WM_TRAY;
        nid.hIcon = app.icon;
        for (i, c) in "Radial Menu".encode_utf16().enumerate() {
            nid.szTip[i] = c;
        }
        Shell_NotifyIconW(if add { NIM_ADD } else { NIM_MODIFY }, &nid);
        if !old.is_null() {
            DestroyIcon(old);
        }
    }
}

fn tray_menu(hwnd: HWND) {
    unsafe {
        let menu = CreatePopupMenu();
        AppendMenuW(menu, MF_STRING, ID_SETTINGS, wide("設定").as_ptr());
        AppendMenuW(menu, MF_STRING, ID_RELOAD, wide("設定を再読み込み").as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        AppendMenuW(menu, MF_STRING, ID_QUIT, wide("終了").as_ptr());
        let mut p = POINT { x: 0, y: 0 };
        GetCursorPos(&mut p);
        SetForegroundWindow(hwnd);
        TrackPopupMenu(menu, TPM_RIGHTBUTTON | TPM_BOTTOMALIGN, p.x, p.y, 0, hwnd, null());
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(menu);
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DOWN => {
            APP.with(|a| {
                if let Some(app) = a.borrow_mut().as_mut() {
                    app.phase = Phase::Pending;
                    SetTimer(hwnd, TIMER_HOLD, app.cfg.hold_ms, None);
                }
            });
            0
        }
        WM_UP => {
            let mut run_item = None;
            let mut click = false;
            APP.with(|a| {
                if let Some(app) = a.borrow_mut().as_mut() {
                    KillTimer(hwnd, TIMER_HOLD);
                    if matches!(app.phase, Phase::Pending) {
                        click = true;
                    } else if app.open.is_some() {
                        let h = hover_at(app);
                        let layer = app.open.as_ref().map(|o| o.layer).unwrap_or(0);
                        run_item = h.and_then(|i| layer_items(app, layer).get(i).cloned());
                        close_menu(app, hwnd);
                    }
                    app.phase = Phase::Idle;
                }
            });
            if click {
                actions::middle_click();
            }
            if let Some(it) = run_item {
                actions::run(&it);
            }
            0
        }
        WM_TIMER => {
            match wparam {
                TIMER_HOLD => {
                    KillTimer(hwnd, TIMER_HOLD);
                    APP.with(|a| {
                        if let Some(app) = a.borrow_mut().as_mut() {
                            if matches!(app.phase, Phase::Pending) && PRESSED.load(Ordering::Relaxed) {
                                app.phase = Phase::Idle;
                                open_menu(app, hwnd, PRESS_X.load(Ordering::Relaxed), PRESS_Y.load(Ordering::Relaxed));
                            }
                        }
                    });
                }
                TIMER_ANIM => {
                    APP.with(|a| {
                        if let Some(app) = a.borrow_mut().as_mut() {
                            let done = match app.open.as_mut() {
                                Some(o) => {
                                    o.alpha = o.alpha.saturating_add(45);
                                    o.alpha == 255
                                }
                                None => true,
                            };
                            redraw(app, hwnd);
                            if done {
                                KillTimer(hwnd, TIMER_ANIM);
                            }
                        }
                    });
                }
                TIMER_TOP => {
                    SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
                }
                _ => {}
            }
            0
        }
        WM_LAYER => {
            APP.with(|a| {
                if let Some(app) = a.borrow_mut().as_mut() {
                    let count = app.cfg.layers.len();
                    let mut changed = false;
                    if let Some(o) = app.open.as_mut() {
                        let cur = o.layer;
                        let next = match wparam {
                            1..=9 => wparam - 1,
                            100 => (cur + 1) % count,
                            101 => (cur + count - 1) % count,
                            _ => cur,
                        };
                        if next < count && next != cur {
                            o.layer = next;
                            changed = true;
                        }
                    }
                    if changed {
                        let h = hover_at(app);
                        if let Some(o) = app.open.as_mut() {
                            o.hover = h;
                        }
                        redraw(app, hwnd);
                    }
                }
            });
            0
        }
        WM_MOVE_HOOK => {
            MOVE_PENDING.store(false, Ordering::Relaxed);
            APP.with(|a| {
                if let Some(app) = a.borrow_mut().as_mut() {
                    let h = hover_at(app);
                    let changed = app.open.as_ref().map(|o| o.hover != h).unwrap_or(false);
                    if changed {
                        if let Some(o) = app.open.as_mut() {
                            o.hover = h;
                        }
                        redraw(app, hwnd);
                    }
                }
            });
            0
        }
        WM_TRAY => {
            match (lparam & 0xffff) as u32 {
                WM_RBUTTONUP => tray_menu(hwnd),
                WM_LBUTTONDBLCLK => launch_settings(),
                _ => {}
            }
            0
        }
        WM_COMMAND => {
            match wparam & 0xffff {
                ID_SETTINGS => launch_settings(),
                ID_RELOAD => APP.with(|a| {
                    if let Some(app) = a.borrow_mut().as_mut() {
                        reload(app, hwnd);
                    }
                }),
                ID_QUIT => {
                    DestroyWindow(hwnd);
                }
                _ => {}
            }
            0
        }
        m if m == MSG_RELOAD => {
            APP.with(|a| {
                if let Some(app) = a.borrow_mut().as_mut() {
                    reload(app, hwnd);
                }
            });
            0
        }
        WM_DESTROY => {
            let mut nid: NOTIFYICONDATAW = zeroed();
            nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = 1;
            Shell_NotifyIconW(NIM_DELETE, &nid);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn show_error(text: &str) {
    if let Ok(base) = std::env::var("LOCALAPPDATA") {
        let dir = std::path::Path::new(&base).join("RadialMenu");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("error.log"), text);
    }
    unsafe {
        MessageBoxW(null_mut(), wide(text).as_ptr(), wide("Radial Menu").as_ptr(), MB_OK | MB_ICONERROR);
    }
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        show_error(&format!("Radial Menu でエラーが発生しました。\n\n{}", info));
    }));
}

fn main() {
    install_panic_hook();
    if std::env::args().any(|a| a == "--settings") {
        launch_settings();
        return;
    }
    unsafe {
        let name = wide("Local\\RadialMenuHostMutex");
        let mutex = CreateMutexW(null(), 0, name.as_ptr());
        if mutex.is_null() {
            show_error("起動の準備に失敗しました（ミューテックスを作成できません）。");
            return;
        }
        if GetLastError() == ERROR_ALREADY_EXISTS {
            launch_settings();
            return;
        }
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        let hinst = GetModuleHandleW(null());
        let class = wide(HOST_CLASS);
        let mut wc: WNDCLASSW = zeroed();
        wc.lpfnWndProc = Some(wnd_proc);
        wc.hInstance = hinst;
        wc.lpszClassName = class.as_ptr();
        wc.hCursor = LoadCursorW(null_mut(), IDC_ARROW);
        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT,
            class.as_ptr(),
            wide("Radial Menu").as_ptr(),
            WS_POPUP,
            0,
            0,
            1,
            1,
            null_mut(),
            null_mut(),
            hinst,
            null(),
        );
        if hwnd.is_null() {
            show_error(&format!("メニュー用ウィンドウを作成できませんでした（エラー {}）。", GetLastError()));
            return;
        }
        HOST.store(hwnd as isize, Ordering::Relaxed);

        let cfg = Config::load().for_host();
        ENABLED.store(cfg.enabled, Ordering::Relaxed);
        apply_layer_keys(&cfg);
        let mut app = App {
            cfg,
            fonts: fonts::Fonts::load(),
            phase: Phase::Idle,
            open: None,
            icon: null_mut(),
        };
        set_tray_icon(&mut app, hwnd, true);
        APP.with(|a| *a.borrow_mut() = Some(app));

        let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), hinst, 0);
        let key_hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(key_proc), hinst, 0);
        if hook.is_null() {
            show_error(&format!("マウスフックを設定できませんでした（エラー {}）。", GetLastError()));
        }

        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        if !hook.is_null() {
            UnhookWindowsHookEx(hook);
        }
        if !key_hook.is_null() {
            UnhookWindowsHookEx(key_hook);
        }
    }
}
