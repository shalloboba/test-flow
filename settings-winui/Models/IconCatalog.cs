namespace RadialMenu.Settings.Models;

public sealed class IconEntry
{
    public IconEntry(string hex, string name) { Hex = hex; Name = name; }
    public string Hex { get; }
    public string Name { get; }
    public string Glyph => char.ConvertFromUtf32(Convert.ToInt32(Hex, 16));
}

/// <summary>Segoe Fluent Icons から、ランチャーで使いやすいものを抜粋。</summary>
public static class IconCatalog
{
    public static readonly IconEntry[] All =
    {
        new("E710", "追加"), new("E70F", "編集"), new("E713", "設定"), new("E715", "メール"),
        new("E716", "ユーザー"), new("E717", "電話"), new("E71B", "リンク"), new("E721", "検索"),
        new("E722", "カメラ"), new("E72C", "更新"), new("E72D", "共有"), new("E72E", "ロック"),
        new("E734", "お気に入り"), new("E74D", "削除"), new("E74E", "保存"), new("E756", "ターミナル"),
        new("E765", "キーボード"), new("E767", "音量"), new("E768", "再生"), new("E74F", "ミュート"),
        new("E774", "インターネット"), new("E77B", "連絡先"), new("E77F", "貼り付け"), new("E787", "カレンダー"),
        new("E7C4", "タスクビュー"), new("E7E8", "電源"), new("E7F4", "デスクトップ"), new("E80F", "ホーム"),
        new("E81C", "履歴"), new("E838", "フォルダー"), new("E892", "前の曲"), new("E893", "次の曲"),
        new("E8A5", "ドキュメント"), new("E8C8", "コピー"), new("E706", "明るさ"), new("E701", "Wi-Fi"),
        new("E702", "Bluetooth"), new("E720", "マイク"), new("E749", "印刷"), new("E895", "同期"),
    };

    public static int IndexOf(string glyph)
    {
        for (var i = 0; i < All.Length; i++)
            if (All[i].Glyph == glyph) return i;
        return -1;
    }
}
