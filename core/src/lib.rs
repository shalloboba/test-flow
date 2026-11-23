use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod net;
pub mod system;
pub mod update;

pub const HOST_CLASS: &str = "RadialMenuHostWindow";
pub const MSG_RELOAD: u32 = 0x8000 + 2;
pub const MAX_ITEMS: usize = 12;
pub const MIN_ITEMS: usize = 2;
pub const MAX_LAYERS: usize = 9;

/// "1"〜"0"、"A"〜"Z"、"F1"〜"F12" を仮想キーコードにする
pub fn key_vk(name: &str) -> Option<u32> {
    let t = name.trim();
    let mut chars = t.chars();
    let first = chars.next()?;
    if chars.next().is_none() {
        return if first.is_ascii_alphanumeric() {
            Some(first.to_ascii_uppercase() as u32)
        } else {
            None
        };
    }
    let n = t.strip_prefix(['f', 'F'])?.parse::<u32>().ok()?;
    if (1..=12).contains(&n) {
        Some(0x70 + n - 1)
    } else {
        None
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Item {
    pub enabled: bool,
    pub label: String,
    pub icon: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub value: String,
    pub hint: String,
}

impl Default for Item {
    fn default() -> Self {
        Item {
            enabled: true,
            label: String::new(),
            icon: "\u{e710}".into(),
            kind: "keys".into(),
            value: String::new(),
            hint: String::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Layer {
    pub name: String,
    /// この段へ切り替えるキー。"" = 自動(段の順番の数字キー)、"none" = 割り当てなし、
    /// それ以外は "1"〜"0" / "A"〜"Z" / "F1"〜"F12"
    pub key: String,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub enabled: bool,
    pub hold_ms: u32,
    pub theme: String,
    pub accent: String,
    pub scale: u32,
    pub update_url: String,
    pub layers: Vec<Layer>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<Item>,
}

fn item(label: &str, icon: &str, kind: &str, value: &str) -> Item {
    Item {
        enabled: true,
        label: label.into(),
        icon: icon.into(),
        kind: kind.into(),
        value: value.into(),
        hint: String::new(),
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            enabled: true,
            hold_ms: 220,
            theme: "auto".into(),
            accent: "auto".into(),
            scale: 100,
            update_url: update::DEFAULT_URL.into(),
            layers: vec![
                Layer {
                    name: "1段目".into(),
                    key: String::new(),
                    items: vec![
                        item("エクスプローラー", "\u{e838}", "keys", "Win+E"),
                        item("ブラウザー", "\u{e774}", "url", "https://www.bing.com"),
                        item("ターミナル", "\u{e756}", "run", "wt.exe"),
                        item("スクリーンショット", "\u{e722}", "keys", "Win+Shift+S"),
                        item("タスクビュー", "\u{e7c4}", "keys", "Win+Tab"),
                        item("設定", "\u{e713}", "keys", "Win+I"),
                        item("ロック", "\u{e72e}", "keys", "Win+L"),
                        item("デスクトップ", "\u{e7f4}", "keys", "Win+D"),
                    ],
                },
                Layer {
                    name: "メディア".into(),
                    key: String::new(),
                    items: vec![
                        item("再生 / 一時停止", "\u{e768}", "keys", "PlayPause"),
                        item("次の曲", "\u{e893}", "keys", "NextTrack"),
                        item("前の曲", "\u{e892}", "keys", "PrevTrack"),
                        item("音量を上げる", "\u{e767}", "keys", "VolumeUp"),
                        item("音量を下げる", "\u{e767}", "keys", "VolumeDown"),
                        item("ミュート", "\u{e74f}", "keys", "Mute"),
                    ],
                },
            ],
            items: Vec::new(),
        }
    }
}

impl Config {
    pub fn sanitize(&mut self) {
        self.hold_ms = self.hold_ms.clamp(80, 1500);
        self.scale = self.scale.clamp(60, 200);
        if !matches!(self.theme.as_str(), "auto" | "dark" | "light") {
            self.theme = "auto".into();
        }
        if self.accent != "auto" && system::parse_hex(&self.accent).is_none() {
            self.accent = "auto".into();
        }
        if self.layers.is_empty() && !self.items.is_empty() {
            let moved = std::mem::take(&mut self.items);
            self.layers.push(Layer {
                name: String::new(),
                key: String::new(),
                items: moved,
            });
        }
        self.items.clear();
        self.layers.truncate(MAX_LAYERS);
        for layer in &mut self.layers {
            layer.items.truncate(MAX_ITEMS);
            for it in &mut layer.items {
                if !matches!(it.kind.as_str(), "keys" | "run" | "url") {
                    it.kind = "keys".into();
                }
            }
        }
        self.layers.retain(|l| l.items.len() >= MIN_ITEMS);
        if self.layers.is_empty() {
            self.layers = Config::default().layers;
        }
        for (i, layer) in self.layers.iter_mut().enumerate() {
            if layer.name.trim().is_empty() {
                layer.name = format!("{}段目", i + 1);
            }
        }
    }

    pub fn for_host(mut self) -> Config {
        for layer in &mut self.layers {
            let active: Vec<Item> = layer.items.iter().filter(|i| i.enabled).cloned().collect();
            if active.len() >= MIN_ITEMS {
                layer.items = active;
            }
        }
        self
    }

    /// 各段に対応する仮想キーコード(0 = 割り当てなし)。段の数だけ返す。
    pub fn layer_vks(&self) -> Vec<u32> {
        self.layers
            .iter()
            .enumerate()
            .map(|(i, l)| match l.key.trim() {
                "" | "auto" => {
                    if i < 9 {
                        0x31 + i as u32
                    } else {
                        0
                    }
                }
                "none" => 0,
                k => key_vk(k).unwrap_or(0),
            })
            .collect()
    }

    pub fn path() -> PathBuf {
        let mut portable = std::env::current_exe().unwrap_or_default();
        portable.set_file_name("config.json");
        if portable.exists() {
            return portable;
        }
        match std::env::var_os("APPDATA") {
            Some(base) => {
                let dir = PathBuf::from(base).join("RadialMenu");
                let _ = std::fs::create_dir_all(&dir);
                dir.join("config.json")
            }
            None => portable,
        }
    }

    pub fn load() -> Config {
        let path = Self::path();
        let mut cfg = match std::fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str::<Config>(text.trim_start_matches('\u{feff}')).unwrap_or_default(),
            Err(_) => {
                let c = Config::default();
                let _ = c.save();
                c
            }
        };
        cfg.sanitize();
        cfg
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path();
        let tmp = path.with_extension("json.tmp");
        let text = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(&tmp, text)?;
        std::fs::rename(&tmp, &path)
    }
}

impl Item {
    pub fn subtitle(&self) -> String {
        if !self.hint.is_empty() {
            return self.hint.clone();
        }
        self.value.split('+').map(|s| s.trim()).collect::<Vec<_>>().join(" + ")
    }
}
