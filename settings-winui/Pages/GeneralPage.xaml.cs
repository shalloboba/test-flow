using System.Diagnostics;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using RadialMenu.Settings.Models;
using RadialMenu.Settings.Services;

namespace RadialMenu.Settings.Pages;

public sealed partial class GeneralPage : Page
{
    public Config Cfg => App.Store.Config;
    public string ConfigPath => ConfigStore.ConfigPath();

    public GeneralPage()
    {
        InitializeComponent();
        Loaded += (_, _) =>
        {
            StartupToggle.IsOn = StartupService.IsEnabled;
            StartupToggle.Toggled += StartupToggle_Toggled;
        };
        Unloaded += (_, _) => StartupToggle.Toggled -= StartupToggle_Toggled;
    }

    void StartupToggle_Toggled(object sender, RoutedEventArgs e)
    {
        try
        {
            StartupService.Set(StartupToggle.IsOn);
        }
        catch
        {
            StartupToggle.Toggled -= StartupToggle_Toggled;
            StartupToggle.IsOn = StartupService.IsEnabled;
            StartupToggle.Toggled += StartupToggle_Toggled;
        }
    }

    void RevealConfig_Click(object sender, RoutedEventArgs e)
    {
        try
        {
            Process.Start("explorer.exe", $"/select,\"{ConfigStore.ConfigPath()}\"");
        }
        catch
        {
        }
    }

    async void Reset_Click(object sender, RoutedEventArgs e)
    {
        var dlg = new ContentDialog
        {
            Title = "既定に戻しますか?",
            Content = "項目・段・キー割り当てを含むすべての設定が最初の状態に戻ります。",
            PrimaryButtonText = "既定に戻す",
            CloseButtonText = "キャンセル",
            DefaultButton = ContentDialogButton.Close,
            XamlRoot = XamlRoot,
        };
        if (await dlg.ShowAsync() == ContentDialogResult.Primary) App.Store.ResetToDefaults();
    }
}
