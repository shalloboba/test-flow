using System.Collections.ObjectModel;
using System.Globalization;
using System.Text.Json.Serialization;

namespace RadialMenu.Settings.Models;

public sealed class Item : Observable
{
    bool _enabled = true;
    string _label = "";
    string _icon = "\uE710";
    string _kind = "keys";
    string _value = "";
    string _hint = "";

    [JsonPropertyName("enabled")]
    public bool Enabled { get => _enabled; set => Set(ref _enabled, value); }

    [JsonPropertyName("label")]
    public string Label { get => _label; set => Set(ref _label, value ?? ""); }

    [JsonPropertyName("icon")]
    public string Icon
    {
        get => _icon;
        set
        {
            if (Set(ref _icon, string.IsNullOrEmpty(value) ? "\uE710" : value))
                Raise(nameof(IconIndex));
        }
    }

    [JsonPropertyName("type")]
    public string Kind
    {
        get => _kind;
        set
        {
            var k = value is "run" or "url" ? value : "keys";
            if (Set(ref _kind, k))
            {
                Raise(nameof(KindIndex));
                Raise(nameof(ValueHeader));
                Raise(nameof(ValuePlaceholder));
            }
        }
    }

    [JsonPropertyName("value")]
    public string Value
    {
        get => _value;
        set { if (Set(ref _value, value ?? "")) Raise(nameof(Subtitle)); }
    }

    [JsonPropertyName("hint")]
    public string Hint
    {
        get => _hint;
        set { if (Set(ref _hint, value ?? "")) Raise(nameof(Subtitle)); }
    }

    // ---- 画面用(JSON には保存しない) ----

    [JsonIgnore]
    public int KindIndex
    {
        get => _kind switch { "run" => 1, "url" => 2, _ => 0 };
        set => Kind = value switch { 1 => "run", 2 => "url", _ => "keys" };
    }

    [JsonIgnore]
    public string ValueHeader => _kind switch
    {
        "run" => "プログラム（パスと引数）",
        "url" => "URL",
        _ => "キー（+ でつなぐ）",
    };

    [JsonIgnore]
    public string ValuePlaceholder => _kind switch
    {
        "run" => "例: notepad.exe",
        "url" => "例: https://example.com",
        _ => "例: Ctrl+Shift+S",
    };

    [JsonIgnore]
    public string Subtitle =>
        _hint.Length > 0
            ? _hint
            : string.Join(" + ", _value.Split('+').Select(s => s.Trim()));

    [JsonIgnore]
    public int IconIndex
    {
        get => IconCatalog.IndexOf(_icon);
        set
        {
            if (value < 0 || value >= IconCatalog.All.Length) return;
            Icon = IconCatalog.All[value].Glyph;
            Raise(nameof(IconHex));
        }
    }

    /// <summary>アイコンの文字コード(16進)。手入力用。入力途中の値は無視する。</summary>
    [JsonIgnore]
    public string IconHex
    {
        get => _icon.Length > 0 ? char.ConvertToUtf32(_icon, 0).ToString("X4") : "";
        set
        {
            var t = (value ?? "").Trim().TrimStart('U', 'u', '+');
            if (t.Length is < 4 or > 5) return;
            if (!int.TryParse(t, NumberStyles.HexNumber, CultureInfo.InvariantCulture, out var cp)) return;
            if (cp is < 0x20 or > 0x10FFFF || (cp >= 0xD800 && cp <= 0xDFFF)) return;
            Icon = char.ConvertFromUtf32(cp);
        }
    }

    // ---- 並べ替え・削除(x:Bind のクリックから呼ぶ) ----

    [JsonIgnore]
    internal ObservableCollection<Item>? Owner { get; set; }

    public void Delete()
    {
        if (Owner is { } o && o.Count > Config.MinItems) o.Remove(this);
    }

    public void MoveUp()
    {
        if (Owner is not { } o) return;
        var i = o.IndexOf(this);
        if (i > 0) o.Move(i, i - 1);
    }

    public void MoveDown()
    {
        if (Owner is not { } o) return;
        var i = o.IndexOf(this);
        if (i >= 0 && i < o.Count - 1) o.Move(i, i + 1);
    }

    public Item Clone() => new()
    {
        Enabled = _enabled, Label = _label, Icon = _icon, Kind = _kind, Value = _value, Hint = _hint,
    };
}
