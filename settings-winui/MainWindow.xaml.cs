using System.Runtime.InteropServices;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Animation;
using RadialMenu.Settings.Pages;
using RadialMenu.Settings.Services;
using WinRT.Interop;

namespace RadialMenu.Settings;

public sealed partial class MainWindow : Window
{
    sealed class SearchEntry
    {
        public SearchEntry(string title, string page, params string[] words)
        {
            Title = title;
            Page = page;
            Words = words;
        }

        public string Title { get; }
        public string Page { get; }
        string[] Words { get; }
        public bool Matches(string q) =>
            Title.Contains(q, StringComparison.CurrentCultureIgnoreCase) ||
            Words.Any(w => w.Contains(q, StringComparison.CurrentCultureIgnoreCase));
        public override string ToString() => Title;
    }

    static readonly SearchEntry[] Entries =
    {
        new("ホーム", "home", "状態", "クイック"),
        new("メニューを有効にする", "general", "有効", "無効", "オン", "オフ"),
        new("長押しの時間", "general", "長押し", "ミリ秒", "ms", "中ボタン", "ホイール"),
        new("メニューのサイズ", "general", "サイズ", "大きさ", "スケール", "拡大"),
        new("テーマ", "general", "ダーク", "ライト", "色"),
        new("アクセント色", "general", "色", "カラー"),
        new("Windows 起動時に自動で実行", "general", "自動起動", "スタートアップ", "常駐"),
        new("設定ファイルの場所", "general", "config", "json", "フォルダー"),
        new("既定に戻す", "general", "リセット", "初期化"),
        new("段を追加・削除・並べ替え", "layers", "レイヤー", "段", "追加", "削除"),
        new("段を切り替えるキー", "layers", "キー", "割り当て", "数字", "ショートカット"),
        new("項目の追加・編集", "layers", "アイテム", "項目", "アイコン", "動作", "キー送信", "URL", "プログラム"),
        new("更新を確認", "about", "アップデート", "更新", "バージョン", "最新"),
        new("バージョン情報", "about", "情報", "バージョン"),
    };

    [DllImport("user32.dll")]
    static extern uint GetDpiForWindow(IntPtr hwnd);

    public MainWindow()
    {
        InitializeComponent();
        Title = App.Title;
        ExtendsContentIntoTitleBar = true;
        SetTitleBar(AppTitleBar);
        SystemBackdrop = new MicaBackdrop();

        try
        {
            var hwnd = WindowNative.GetWindowHandle(this);
            var scale = GetDpiForWindow(hwnd) / 96.0;
            AppWindow.Resize(new Windows.Graphics.SizeInt32((int)(1100 * scale), (int)(760 * scale)));
            AppWindow.SetIcon(Path.Combine(AppContext.BaseDirectory, "app.ico"));
        }
        catch
        {
        }

        Closed += (_, _) => App.Store.FlushIfDirty();
        Root.Loaded += async (_, _) =>
        {
            await UpdateService.CheckAsync(App.Store.Config.UpdateUrl);
        };

        Nav.SelectedItem = Nav.MenuItems[0];
    }

    public void NavigateTo(string tag)
    {
        foreach (var o in Nav.MenuItems)
        {
            if (o is NavigationViewItem item && item.Tag as string == tag)
            {
                Nav.SelectedItem = item;
                return;
            }
        }
    }

    void Nav_SelectionChanged(NavigationView sender, NavigationViewSelectionChangedEventArgs args)
    {
        var tag = args.SelectedItemContainer?.Tag as string ?? "home";
        var page = tag switch
        {
            "layers" => typeof(LayersPage),
            "general" => typeof(GeneralPage),
            "about" => typeof(AboutPage),
            _ => typeof(HomePage),
        };
        if (ContentFrame.CurrentSourcePageType != page)
            ContentFrame.Navigate(page, null, new EntranceNavigationTransitionInfo());
    }

    void Search_TextChanged(AutoSuggestBox sender, AutoSuggestBoxTextChangedEventArgs args)
    {
        if (args.Reason != AutoSuggestionBoxTextChangeReason.UserInput) return;
        var q = sender.Text.Trim();
        sender.ItemsSource = q.Length == 0 ? null : Entries.Where(e => e.Matches(q)).ToList();
    }

    void Search_SuggestionChosen(AutoSuggestBox sender, AutoSuggestBoxSuggestionChosenEventArgs args)
    {
        if (args.SelectedItem is SearchEntry e) NavigateTo(e.Page);
    }

    void Search_QuerySubmitted(AutoSuggestBox sender, AutoSuggestBoxQuerySubmittedEventArgs args)
    {
        var e = args.ChosenSuggestion as SearchEntry ?? Entries.FirstOrDefault(x => x.Matches(args.QueryText.Trim()));
        if (e != null) NavigateTo(e.Page);
    }
}
