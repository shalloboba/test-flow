using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using RadialMenu.Settings.Models;
using RadialMenu.Settings.Services;

namespace RadialMenu.Settings.Pages;

public sealed partial class HomePage : Page
{
    public Config Cfg => App.Store.Config;

    public HomePage()
    {
        InitializeComponent();
        Loaded += (_, _) =>
        {
            UpdateService.Changed += OnUpdateChanged;
            RefreshUpdate();
            HostBar.IsOpen = !HostLink.IsRunning;
        };
        Unloaded += (_, _) => UpdateService.Changed -= OnUpdateChanged;
    }

    void OnUpdateChanged() => DispatcherQueue.TryEnqueue(RefreshUpdate);

    void RefreshUpdate()
    {
        var latest = UpdateService.Latest;
        UpdateBar.IsOpen = latest is { IsNewer: true };
        if (latest != null) UpdateBar.Message = $"{UpdateService.CurrentVersion} → {latest.Version}";
    }

    void OpenAbout_Click(object sender, RoutedEventArgs e) => App.MainWindow?.NavigateTo("about");
    void EditLayers_Click(object sender, RoutedEventArgs e) => App.MainWindow?.NavigateTo("layers");
    void EditGeneral_Click(object sender, RoutedEventArgs e) => App.MainWindow?.NavigateTo("general");

    async void StartHost_Click(object sender, RoutedEventArgs e)
    {
        HostLink.Start();
        await Task.Delay(700);
        HostBar.IsOpen = !HostLink.IsRunning;
    }
}
