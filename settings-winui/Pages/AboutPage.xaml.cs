using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using RadialMenu.Settings.Models;
using RadialMenu.Settings.Services;

namespace RadialMenu.Settings.Pages;

public sealed partial class AboutPage : Page
{
    public Config Cfg => App.Store.Config;
    public string VersionText => $"バージョン {UpdateService.CurrentVersion}";

    public AboutPage()
    {
        InitializeComponent();
        Loaded += (_, _) =>
        {
            UpdateService.Changed += OnChanged;
            Refresh();
        };
        Unloaded += (_, _) => UpdateService.Changed -= OnChanged;
    }

    void OnChanged() => DispatcherQueue.TryEnqueue(Refresh);

    void Refresh()
    {
        Ring.IsActive = UpdateService.Checking;
        CheckButton.IsEnabled = !UpdateService.Checking;
        var latest = UpdateService.Latest;
        UpdateButton.Visibility = latest is { IsNewer: true } ? Visibility.Visible : Visibility.Collapsed;

        if (UpdateService.Checking) Message.Text = "確認しています…";
        else if (UpdateService.Error != null) Message.Text = $"確認できませんでした: {UpdateService.Error}";
        else if (latest == null) Message.Text = "まだ確認していません。";
        else if (latest.IsNewer) Message.Text = $"新しい版 {latest.Version} があります(現在 {UpdateService.CurrentVersion})。";
        else Message.Text = "最新の版です。";
    }

    async void Check_Click(object sender, RoutedEventArgs e) =>
        await UpdateService.CheckAsync(Cfg.UpdateUrl);

    async void Update_Click(object sender, RoutedEventArgs e)
    {
        if (UpdateService.Latest is not { IsNewer: true } info) return;
        UpdateButton.IsEnabled = false;
        CheckButton.IsEnabled = false;
        DownloadBar.Value = 0;
        DownloadBar.Visibility = Visibility.Visible;
        Message.Text = "ダウンロードしています…";

        var progress = new Progress<double>(p => DownloadBar.Value = p);
        var error = await UpdateService.DownloadAndRunAsync(info, progress);

        DownloadBar.Visibility = Visibility.Collapsed;
        UpdateButton.IsEnabled = true;
        CheckButton.IsEnabled = true;
        if (error != null)
        {
            Message.Text = $"更新できませんでした: {error}";
            return;
        }
        Message.Text = "インストーラーを起動しました。";
        App.Store.FlushIfDirty();
        Application.Current.Exit();
    }
}
