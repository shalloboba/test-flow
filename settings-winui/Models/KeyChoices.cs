namespace RadialMenu.Settings.Models;

/// <summary>段を切り替えるキーの選択肢。Values は config.json の "key" に保存される値。</summary>
public static class KeyChoices
{
    public static readonly string[] Values;
    public static readonly string[] Labels;

    static KeyChoices()
    {
        var v = new List<string> { "", "none" };
        var l = new List<string> { "自動（段の順番の数字キー）", "割り当てなし" };
        foreach (var c in "1234567890") { v.Add(c.ToString()); l.Add(c.ToString()); }
        for (var c = 'A'; c <= 'Z'; c++) { v.Add(c.ToString()); l.Add(c.ToString()); }
        for (var i = 1; i <= 12; i++) { v.Add("F" + i); l.Add("F" + i); }
        Values = v.ToArray();
        Labels = l.ToArray();
    }

    public static int IndexOf(string key)
    {
        for (var i = 0; i < Values.Length; i++)
            if (string.Equals(Values[i], key, StringComparison.OrdinalIgnoreCase)) return i;
        return -1;
    }

    public static string Describe(string key)
    {
        var i = IndexOf(key);
        return i switch
        {
            0 => "自動",
            1 => "なし",
            > 1 => Values[i],
            _ => key,
        };
    }
}
