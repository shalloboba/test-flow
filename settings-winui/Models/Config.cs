using System.Collections.ObjectModel;
using System.Globalization;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace RadialMenu.Settings.Models;

/// <summary>config.json と同じ形(Rust 側の Config と互換)。</summary>
public sealed class Config : Observable
{
    public const int MinItems = 2;
    public const int MaxItems = 12;
    public const int MaxLayers = 9;

    bool _enabled = true;
    int _holdMs = 220;
    string _theme = "auto";
    string _accent = "auto";
    int _scale = 100;
    string _updateUrl = "";
    ObservableCollection<Layer> _layers = new();

    [JsonPropertyName("enabled")]
    public bool Enabled { get => _enabled; set => Set(ref _enabled, value); }

    [JsonPropertyName("hold_ms")]
    public int HoldMs
    {
        get => _holdMs;
        set { if (Set(ref _holdMs, Math.Clamp(value, 80, 1500))) Raise(nameof(HoldMsValue)); }
    }

    [JsonPropertyName("theme")]
    public string Theme
    {
        get => _theme;
        set
        {
            var t = value is "light" or "dark" ? value : "auto";
            if (Set(ref _theme, t)) Raise(nameof(ThemeIndex));
        }
    }

    [JsonPropertyName("accent")]
    public string Accent
    {
        get => _accent;
        set
        {
            var a = IsHex(value) ? value.ToLowerInvariant() : "auto";
            if (Set(ref _accent, a))
            {
                Raise(nameof(UseAutoAccent));
                Raise(nameof(UseCustomAccent));
            }
        }
    }

    [JsonPropertyName("scale")]
    public int Scale
    {
        get => _scale;
        set
        {
            if (Set(ref _scale, Math.Clamp(value, 60, 200)))
            {
                Raise(nameof(ScaleValue));
                Raise(nameof(ScaleText));
            }
        }
    }

    [JsonPropertyName("update_url")]
    public string UpdateUrl { get => _updateUrl; set => Set(ref _updateUrl, value ?? ""); }

    [JsonPropertyName("layers")]
    public ObservableCollection<Layer> Layers
    {
        get => _layers;
        set { _layers = value ?? new ObservableCollection<Layer>(); Raise(); }
    }

    /// <summary>旧形式(段のない items だけの設定)。読み込み時に 1 段目へ移す。</summary>
    [JsonPropertyName("items")]
    public List<Item>? LegacyItems { get; set; }

    /// <summary>このアプリが知らない項目を、保存するときに失わないよう保持する。</summary>
    [JsonExtensionData]
    public Dictionary<string, JsonElement>? Extra { get; set; }

    // ---- 画面用(JSON には保存しない) ----

    [JsonIgnore]
    public double HoldMsValue
    {
        get => _holdMs;
        set { if (!double.IsNaN(value)) HoldMs = (int)Math.Round(value); }
    }

    [JsonIgnore]
    public double ScaleValue
    {
        get => _scale;
        set { if (!double.IsNaN(value)) Scale = (int)Math.Round(value); }
    }

    [JsonIgnore]
    public string ScaleText => $"{_scale}%";

    [JsonIgnore]
    public int ThemeIndex
    {
        get => _theme switch { "light" => 1, "dark" => 2, _ => 0 };
        set => Theme = value switch { 1 => "light", 2 => "dark", _ => "auto" };
    }

    [JsonIgnore]
    public bool UseAutoAccent
    {
        get => _accent == "auto";
        set { if (value) Accent = "auto"; else if (_accent == "auto") Accent = "#0078d4"; }
    }

    [JsonIgnore]
    public bool UseCustomAccent => _accent != "auto";

    /// <summary>アクセント色の手入力欄。#RRGGBB になるまでは反映しない。</summary>
    [JsonIgnore]
    public string AccentText
    {
        get => _accent == "auto" ? "" : _accent;
        set { if (IsHex(value)) Accent = value; }
    }

    public static bool IsHex(string? s)
    {
        if (s is not { Length: 7 } || s[0] != '#') return false;
        return int.TryParse(s.AsSpan(1), NumberStyles.HexNumber, CultureInfo.InvariantCulture, out _);
    }

    /// <summary>ホスト(Rust 側)の sanitize と同じ規則で整える。</summary>
    public void Sanitize()
    {
        if (_layers.Count == 0 && LegacyItems is { Count: > 0 })
        {
            var l = new Layer { Name = "" };
            foreach (var i in LegacyItems) l.Items.Add(i);
            _layers.Add(l);
        }
        LegacyItems = null;

        while (_layers.Count > MaxLayers) _layers.RemoveAt(_layers.Count - 1);
        foreach (var l in _layers)
            while (l.Items.Count > MaxItems) l.Items.RemoveAt(l.Items.Count - 1);
        for (var i = _layers.Count - 1; i >= 0; i--)
            if (_layers[i].Items.Count < MinItems) _layers.RemoveAt(i);
        if (_layers.Count == 0)
            foreach (var l in CreateDefault().Layers) _layers.Add(l);
        for (var i = 0; i < _layers.Count; i++)
            if (string.IsNullOrWhiteSpace(_layers[i].Name)) _layers[i].Name = $"{i + 1}段目";
    }

    public void CopyFrom(Config other)
    {
        Enabled = other.Enabled;
        HoldMs = other.HoldMs;
        Theme = other.Theme;
        Accent = other.Accent;
        Scale = other.Scale;
        UpdateUrl = other.UpdateUrl;
        Layers.Clear();
        foreach (var l in other.Layers) Layers.Add(l.Clone());
    }

    static Item It(string label, string glyph, string kind, string value) =>
        new() { Label = label, Icon = glyph, Kind = kind, Value = value };

    public static Config CreateDefault()
    {
        var c = new Config();
        var main = new Layer { Name = "1段目" };
        foreach (var i in new[]
        {
            It("エクスプローラー", "\uE838", "keys", "Win+E"),
            It("ブラウザー", "\uE774", "url", "https://www.bing.com"),
            It("ターミナル", "\uE756", "run", "wt.exe"),
            It("スクリーンショット", "\uE722", "keys", "Win+Shift+S"),
            It("タスクビュー", "\uE7C4", "keys", "Win+Tab"),
            It("設定", "\uE713", "keys", "Win+I"),
            It("ロック", "\uE72E", "keys", "Win+L"),
            It("デスクトップ", "\uE7F4", "keys", "Win+D"),
        }) main.Items.Add(i);
        var media = new Layer { Name = "メディア" };
        foreach (var i in new[]
        {
            It("再生 / 一時停止", "\uE768", "keys", "PlayPause"),
            It("次の曲", "\uE893", "keys", "NextTrack"),
            It("前の曲", "\uE892", "keys", "PrevTrack"),
            It("音量を上げる", "\uE767", "keys", "VolumeUp"),
            It("音量を下げる", "\uE767", "keys", "VolumeDown"),
            It("ミュート", "\uE74F", "keys", "Mute"),
        }) media.Items.Add(i);
        c.Layers.Add(main);
        c.Layers.Add(media);
        return c;
    }
}
