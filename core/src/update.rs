use crate::net;
use serde::Deserialize;

pub const DEFAULT_URL: &str = match option_env!("RADIAL_UPDATE_URL") {
    Some(v) => v,
    None => "",
};

#[derive(Clone, Debug, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub url: String,
    #[serde(default)]
    pub sha256: String,
}

pub fn fetch_manifest(base: &str) -> Result<Manifest, String> {
    let base = base.trim().trim_end_matches('/');
    if base.is_empty() {
        return Err("更新元の URL が設定されていません".into());
    }
    let bytes = net::get_bytes(&format!("{}/latest.json", base))?;
    let text = String::from_utf8_lossy(&bytes);
    let mut m: Manifest = serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| format!("latest.json を読めません: {}", e))?;
    if !m.url.starts_with("http://") && !m.url.starts_with("https://") {
        m.url = format!("{}/{}", base, m.url.trim_start_matches('/'));
    }
    Ok(m)
}

fn parts(v: &str) -> Vec<u64> {
    v.trim_start_matches(['v', 'V'])
        .split('.')
        .map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0))
        .collect()
}

pub fn is_newer(latest: &str, current: &str) -> bool {
    let a = parts(latest);
    let b = parts(current);
    for i in 0..a.len().max(b.len()) {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        if x != y {
            return x > y;
        }
    }
    false
}
