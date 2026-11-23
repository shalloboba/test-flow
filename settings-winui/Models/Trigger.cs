using System.Collections.ObjectModel;
using System.Text.Json.Serialization;

namespace RadialMenu.Settings.Models;

/// <summary>メニューを開く操作 1 つ分。Rust 側の Trigger と互換。</summary>
public sealed class Trigger : Observable
{
    string _type = "mouse";
    string _button = "middle";
    ObservableCollection<string> _keys = new();

    public Trigger()
    {
        _keys.CollectionChanged += (_, _) => Raise(nameof(Display));
    }

    [JsonPropertyName("type")]
    public string Type
    {
        get => _type;
        set { if (Set(ref _type, value == "keys" ? "keys" : "mouse")) Raise(nameof(Display)); }
    }

    /// <summary>mouse のときのボタン: "middle" / "right" / "x1" / "x2"</summary>
    [JsonPropertyName("button")]
    public string Button
    {
        get => _button;
        set
        {
            var b = value is "right" or "x1" or "x2" ? value : "middle";
            if (Set(ref _button, b)) Raise(nameof(Display));
        }
    }

    /// <summary>keys のときのキー名の列 (例: "LControl", "Space")</summary>
    [JsonPropertyName("keys")]
    public ObservableCollection<string> Keys
    {
        get => _keys;
        set
        {
            _keys = value ?? new ObservableCollection<string>();
            _keys.CollectionChanged += (_, _) => Raise(nameof(Display));
            Raise();
            Raise(nameof(Display));
        }
    }

    // ---- 画面用(JSON には保存しない) ----

    [JsonIgnore]
    public string ButtonLabel => _button switch
    {
        "right" => "マウス 右ボタン",
        "x1" => "マウス サイドボタン1",
        "x2" => "マウス サイドボタン2",
        _ => "マウス 中ボタン",
    };

    [JsonIgnore]
    public string Display => _type == "keys"
        ? string.Join(" + ", _keys) + " の長押し"
        : ButtonLabel + " の長押し";

    public Trigger Clone()
    {
        var t = new Trigger { Type = _type, Button = _button };
        foreach (var k in _keys) t.Keys.Add(k);
        return t;
    }
}

/// <summary>起動トリガーの集まり。Rust 側の Activation と互換。</summary>
public sealed class Activation : Observable
{
    ObservableCollection<Trigger> _triggers = new();

    [JsonPropertyName("triggers")]
    public ObservableCollection<Trigger> Triggers
    {
        get => _triggers;
        set { _triggers = value ?? new ObservableCollection<Trigger>(); Raise(); }
    }

    public static Activation CreateDefault()
    {
        var a = new Activation();
        a.Triggers.Add(new Trigger());
        return a;
    }
}
