use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn read_dword(sub: &str, name: &str) -> Option<u32> {
    let sub = wide(sub);
    let name = wide(name);
    let mut data: u32 = 0;
    let mut size: u32 = 4;
    let rc = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            sub.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            &mut data as *mut u32 as *mut _,
            &mut size,
        )
    };
    if rc == 0 {
        Some(data)
    } else {
        None
    }
}

pub fn system_is_light() -> bool {
    read_dword(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
        "AppsUseLightTheme",
    )
    .map(|v| v != 0)
    .unwrap_or(false)
}

pub fn system_accent() -> [u8; 3] {
    match read_dword("Software\\Microsoft\\Windows\\DWM", "AccentColor") {
        Some(v) => [(v & 0xff) as u8, ((v >> 8) & 0xff) as u8, ((v >> 16) & 0xff) as u8],
        None => [0x00, 0x78, 0xd4],
    }
}

pub fn parse_hex(s: &str) -> Option<[u8; 3]> {
    let h = s.strip_prefix('#')?;
    if h.len() != 6 || !h.is_ascii() {
        return None;
    }
    let v = u32::from_str_radix(h, 16).ok()?;
    Some([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

pub fn to_hex(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_NAME: &str = "RadialMenu";

pub fn startup_enabled() -> bool {
    use windows_sys::Win32::System::Registry::RRF_RT_REG_SZ;
    let sub = wide(RUN_KEY);
    let name = wide(RUN_NAME);
    let mut size: u32 = 0;
    let rc = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            sub.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut size,
        )
    };
    rc == 0
}

pub fn set_startup(on: bool, exe: &str) {
    use windows_sys::Win32::System::Registry::{RegDeleteKeyValueW, RegSetKeyValueW, REG_SZ};
    let sub = wide(RUN_KEY);
    let name = wide(RUN_NAME);
    unsafe {
        if on {
            let value = wide(&format!("\"{}\"", exe));
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                sub.as_ptr(),
                name.as_ptr(),
                REG_SZ,
                value.as_ptr() as *const _,
                (value.len() * 2) as u32,
            );
        } else {
            RegDeleteKeyValueW(HKEY_CURRENT_USER, sub.as_ptr(), name.as_ptr());
        }
    }
}

pub fn sibling_exe(name: &str) -> std::path::PathBuf {
    let mut p = std::env::current_exe().unwrap_or_default();
    p.set_file_name(name);
    p
}
