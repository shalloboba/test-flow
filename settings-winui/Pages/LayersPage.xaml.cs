using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using RadialMenu.Settings.Models;
using Windows.Storage.Pickers;

namespace RadialMenu.Settings.Pages;

public sealed partial class LayersPage : Page
{
    public Config Cfg => App.Store.Config;

    public LayersPage()
    {
        InitializeComponent();
        Loaded += (_, _) =>
        {
            if (LayerList.SelectedIndex < 0 && Cfg.Layers.Count > 0) LayerList.SelectedIndex = 0;
        };
    }

    void AddLayer_Click(object sender, RoutedEventArgs e)
    {
        if (Cfg.Layers.Count >= Config.MaxLayers)
        {
            _ = ShowMessageAsync("段は最大 9 つまでです。");
            return;
        }
        var layer = Layer.CreateBlank(Cfg.Layers.Count + 1);
        Cfg.Layers.Add(layer);
        LayerList.SelectedItem = layer;
    }

    async void DeleteLayer_Click(object sender, RoutedEventArgs e)
    {
        if (LayerList.SelectedItem is not Layer layer) return;
        if (Cfg.Layers.Count <= 1)
        {
            await ShowMessageAsync("最後の段は削除できません。");
            return;
        }
        var dlg = new ContentDialog
        {
            Title = "段を削除しますか?",
            Content = $"「{layer.Name}」とその項目をすべて削除します。",
            PrimaryButtonText = "削除",
            CloseButtonText = "キャンセル",
            DefaultButton = ContentDialogButton.Close,
            XamlRoot = XamlRoot,
        };
        if (await dlg.ShowAsync() != ContentDialogResult.Primary) return;

        var index = Cfg.Layers.IndexOf(layer);
        Cfg.Layers.Remove(layer);
        if (Cfg.Layers.Count > 0) LayerList.SelectedIndex = Math.Min(index, Cfg.Layers.Count - 1);
    }

    void MoveLayerUp_Click(object sender, RoutedEventArgs e) => MoveLayer(-1);
    void MoveLayerDown_Click(object sender, RoutedEventArgs e) => MoveLayer(1);

    void MoveLayer(int delta)
    {
        if (LayerList.SelectedItem is not Layer layer) return;
        var from = Cfg.Layers.IndexOf(layer);
        var to = from + delta;
        if (to < 0 || to >= Cfg.Layers.Count) return;
        Cfg.Layers.Move(from, to);
        LayerList.SelectedItem = layer;
    }

    /// <summary>「プログラムを実行」の参照ボタン。選んだファイルのパスを項目へ入れる。</summary>
    async void BrowseRunPath_Click(object sender, RoutedEventArgs e)
    {
        if (sender is not FrameworkElement { DataContext: Item item }) return;
        if (App.MainWindow is not { } window) return;
        var picker = new FileOpenPicker
        {
            ViewMode = PickerViewMode.List,
            SuggestedStartLocation = PickerLocationId.ComputerFolder,
        };
        picker.FileTypeFilter.Add(".exe");
        picker.FileTypeFilter.Add(".lnk");
        picker.FileTypeFilter.Add(".bat");
        picker.FileTypeFilter.Add(".cmd");
        picker.FileTypeFilter.Add("*");
        WinRT.Interop.InitializeWithWindow.Initialize(picker, WinRT.Interop.WindowNative.GetWindowHandle(window));
        var file = await picker.PickSingleFileAsync();
        if (file is not null) item.RunPath = file.Path;
    }

    async Task ShowMessageAsync(string text)
    {
        var dlg = new ContentDialog
        {
            Title = "Radial Menu",
            Content = text,
            CloseButtonText = "OK",
            XamlRoot = XamlRoot,
        };
        await dlg.ShowAsync();
    }
}
