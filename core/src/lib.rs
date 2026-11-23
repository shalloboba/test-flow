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

/// 起動トリガーのキー名を仮想キーコードにする。
/// 修飾キーの汎用名 (Control / Shift / Alt / Win) は左右どちらでもよい印として
/// VK_ANY_* (0x100 番台) を返す。物理キーの VK とは重ならない。
pub const VK_ANY_CONTROL: u32 = 0x100;
pub const VK_ANY_SHIFT: u32 = 0x101;
pub const VK_ANY_ALT: u32 = 0x102;
pub const VK_ANY_WIN: u32 = 0x103;

pub fn trigger_vk(name: &str) -> Option<u32> {
    let t = name.trim().to_lowercase();
    let named = match t.as_str() {
        "lcontrol" | "lctrl" => 0xA2,
        "rcontrol" | "rctrl" => 0xA3,
        "control" | "ctrl" => VK_ANY_CONTROL,
        "lshift" => 0xA0,
        "rshift" => 0xA1,
        "shift" => VK_ANY_SHIFT,
        "lalt" | "lmenu" => 0xA4,
        "ralt" | "rmenu" => 0xA5,
        "alt" | "menu" => VK_ANY_ALT,
        "lwin" | "lwindows" => 0x5B,
        "rwin" | "rwindows" => 0x5C,
        "win" | "windows" | "super" | "meta" => VK_ANY_WIN,
        "space" => 0x20,
        "tab" => 0x09,
        "enter" | "return" => 0x0D,
        "esc" | "escape" => 0x1B,
        "backspace" | "back" => 0x08,
        "delete" | "del" => 0x2E,
        "insert" | "ins" => 0x2D,
        "home" => 0x24,
        "end" => 0x23,
        "pageup" | "pgup" => 0x21,
        "pagedown" | "pgdn" => 0x22,
        "up" => 0x26,
        "down" => 0x28,
        "left" => 0x25,
        "right" => 0x27,
        "capslock" | "capitallock" => 0x14,
        "printscreen" | "prtsc" | "snapshot" => 0x2C,
        "playpause" => 0xB3,
        "nexttrack" => 0xB0,
        "prevtrack" => 0xB1,
        "volumeup" => 0xAF,
        "volumedown" => 0xAE,
        "mute" => 0xAD,
        _ => 0,
    };
    if named != 0 {
        return Some(named);
    }
    if let Some(n) = t.strip_prefix("numpad").and_then(|x| x.parse::<u32>().ok()) {
        if n <= 9 {
            return Some(0x60 + n);
        }
    }
    if let Some(n) = t.strip_prefix('f').and_then(|x| x.parse::<u32>().ok()) {
        if (1..=24).contains(&n) {
            return Some(0x70 + n - 1);
        }
    }
    let mut it = t.chars();
    let c = it.next()?;
    if it.next().is_none() && c.is_ascii_alphanumeric() {
        return Some(c.to_ascii_uppercase() as u32);
    }
    None
}

/// トリガー側のキーコード 1 つが、押下中の集合のどれかで満たされるか。
fn trigger_key_hit(want: u32, pressed: &[u32], used: &mut [bool]) -> bool {
    let group: &[u32] = match want {
        VK_ANY_CONTROL => &[0xA2, 0xA3],
        VK_ANY_SHIFT => &[0xA0, 0xA1],
        VK_ANY_ALT => &[0xA4, 0xA5],
        VK_ANY_WIN => &[0x5B, 0x5C],
        _ => &[],
    };
    for (i, &p) in pressed.iter().enumerate() {
        if used[i] {
            continue;
        }
        if p == want || group.contains(&p) {
            used[i] = true;
            return true;
        }
    }
    false
}

/// 押下中のキーの集合が、トリガーのキー列とちょうど一致するか (余分なキーがあってもいけない)。
pub fn keys_match(trigger: &[u32], pressed: &[u32]) -> bool {
    if trigger.is_empty() || trigger.len() != pressed.len() {
        return false;
    }
    let mut used = vec![false; pressed.len()];
    trigger.iter().all(|&w| trigger_key_hit(w, pressed, &mut used))
}

/// トリガーのキーがすべて押下中の集合に含まれているか (余分なキーは不問)。
/// メニュー表示中に「トリガーを離した」ことを判定するために使う。
pub fn keys_contain(trigger: &[u32], pressed: &[u32]) -> bool {
    if trigger.is_empty() || trigger.len() > pressed.len() {
        return false;
    }
    let mut used = vec![false; pressed.len()];
    trigger.iter().all(|&w| trigger_key_hit(w, pressed, &mut used))
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

/// メニューを開く操作 1 つ分。"mouse" はボタンの長押し、"keys" はキーの同時押し長押し。
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Trigger {
    #[serde(rename = "type")]
    pub kind: String,
    /// mouse のときのボタン: "middle" / "right" / "x1" / "x2"
    pub button: String,
    /// keys のときのキー名の列 (trigger_vk で解釈できる名前)
    pub keys: Vec<String>,
}

impl Default for Trigger {
    fn default() -> Self {
        Trigger {
            kind: "mouse".into(),
            button: "middle".into(),
            keys: Vec::new(),
        }
    }
}

impl Trigger {
    /// mouse トリガーのボタンのビット (1=middle, 2=right, 4=x1, 8=x2)。keys なら 0。
    pub fn mouse_bit(&self) -> u32 {
        if self.kind != "mouse" {
            return 0;
        }
        match self.button.as_str() {
            "right" => 2,
            "x1" => 4,
            "x2" => 8,
            _ => 1,
        }
    }

    /// keys トリガーを VK コード列にする。keys でなければ空。
    pub fn key_vks(&self) -> Vec<u32> {
        if self.kind != "keys" {
            return Vec::new();
        }
        self.keys.iter().filter_map(|k| trigger_vk(k)).collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Activation {
    pub triggers: Vec<Trigger>,
}

impl Default for Activation {
    fn default() -> Self {
        Activation {
            triggers: vec![Trigger::default()],
        }
    }
}

impl Activation {
    /// 有効な mouse トリガー全部のボタンのビットマスク。
    pub fn mouse_mask(&self) -> u32 {
        self.triggers.iter().map(|t| t.mouse_bit()).fold(0, |a, b| a | b)
    }

    /// 有効な keys トリガー全部を VK コード列にしたもの。
    pub fn key_triggers(&self) -> Vec<Vec<u32>> {
        self.triggers
            .iter()
            .map(|t| t.key_vks())
            .filter(|v| !v.is_empty())
            .collect()
    }
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
    pub activation: Activation,
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
            activation: Activation::default(),
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
        // 起動トリガー: 不正なものを落とし、最低 1 つ (中ボタン長押し) を保証する
        {
            let mut triggers: Vec<Trigger> = Vec::new();
            for mut t in std::mem::take(&mut self.activation.triggers) {
                match t.kind.as_str() {
                    "mouse" => {
                        if !matches!(t.button.as_str(), "middle" | "right" | "x1" | "x2") {
                            t.button = "middle".into();
                        }
                        t.keys.clear();
                        triggers.push(t);
                    }
                    "keys" => {
                        let mut seen = std::collections::HashSet::new();
                        t.keys.retain(|k| {
                            trigger_vk(k).is_some() && seen.insert(k.trim().to_lowercase())
                        });
                        if !t.keys.is_empty() {
                            triggers.push(t);
                        }
                    }
                    _ => {}
                }
            }
            triggers.truncate(8);
            if triggers.is_empty() {
                triggers.push(Trigger::default());
            }
            self.activation.triggers = triggers;
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
