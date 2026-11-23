using System.Collections.ObjectModel;
using System.Text.Json.Serialization;

namespace RadialMenu.Settings.Models;

public sealed class Layer : Observable
{
    string _name = "";
    string _key = "";
    ObservableCollection<Item> _items = new();

    public Layer() => HookItems(_items);

    [JsonPropertyName("name")]
    public string Name { get => _name; set => Set(ref _name, value ?? ""); }

    /// <summary>この段へ切り替えるキー。"" = 自動、"none" = なし、それ以外はキー名。</summary>
    [JsonPropertyName("key")]
    public string Key
    {
        get => _key;
        set
        {
            if (Set(ref _key, value ?? ""))
            {
                Raise(nameof(KeyIndex));
                Raise(nameof(Summary));
            }
        }
    }

    [JsonPropertyName("items")]
    public ObservableCollection<Item> Items
    {
        get => _items;
        set
        {
            _items = value ?? new ObservableCollection<Item>();
            HookItems(_items);
            Raise();
            Raise(nameof(Summary));
        }
    }

    [JsonIgnore]
    public int KeyIndex
    {
        get => KeyChoices.IndexOf(_key);
        set
        {
            if (value >= 0 && value < KeyChoices.Values.Length) Key = KeyChoices.Values[value];
        }
    }

    [JsonIgnore]
    public string Summary => $"キー: {KeyChoices.Describe(_key)} ・ {_items.Count} 項目";

    void HookItems(ObservableCollection<Item> c)
    {
        foreach (var i in c) i.Owner = c;
        c.CollectionChanged += (_, e) =>
        {
            if (e.NewItems != null)
                foreach (Item i in e.NewItems) i.Owner = c;
            Raise(nameof(Summary));
        };
    }

    public void AddItem()
    {
        if (_items.Count >= Config.MaxItems) return;
        _items.Add(new Item { Label = $"項目{_items.Count + 1}", Icon = "\uE710", Kind = "keys", Value = "" });
    }

    public static Layer CreateBlank(int number)
    {
        var l = new Layer { Name = $"{number}段目" };
        l.Items.Add(new Item { Label = "項目1", Icon = "\uE710", Kind = "keys", Value = "" });
        l.Items.Add(new Item { Label = "項目2", Icon = "\uE710", Kind = "keys", Value = "" });
        return l;
    }

    public Layer Clone()
    {
        var l = new Layer { Name = _name, Key = _key };
        foreach (var i in _items) l.Items.Add(i.Clone());
        return l;
    }
}
