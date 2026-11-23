using System.Collections.ObjectModel;
using System.Diagnostics;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using RadialMenu.Settings.Models;
using RadialMenu.Settings.Services;
using Windows.System;

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

    // ---- 起動方法(トリガー)の編集 ----

    async void DeleteTrigger_Click(object sender, RoutedEventArgs e)
    {
        if (sender is not FrameworkElement { DataContext: Trigger trigger }) return;
        if (Cfg.Activation.Triggers.Count <= 1)
        {
            await ShowMessageAsync("起動方法は最低 1 つ必要です。");
            return;
        }
        Cfg.Activation.Triggers.Remove(trigger);
    }

    void AddMouseTrigger_Click(object sender, RoutedEventArgs e)
    {
        var button = MouseButtonCombo.SelectedIndex switch
        {
            1 => "right",
            2 => "x1",
            3 => "x2",
            _ => "middle",
        };
        AddTrigger(new Trigger { Type = "mouse", Button = button });
    }

    void AddTrigger(Trigger candidate)
    {
        var triggers = Cfg.Activation.Triggers;
        if (triggers.Count >= 8) return;
        foreach (var t in triggers)
        {
            if (t.Type != candidate.Type) continue;
            if (candidate.Type == "mouse" && t.Button == candidate.Button) return;
            if (candidate.Type == "keys" && t.Keys.SequenceEqual(candidate.Keys)) return;
        }
        triggers.Add(candidate);
    }

    // キーの組み合わせの記録

    readonly HashSet<VirtualKey> _captureDown = new();
    readonly List<string> _captureNames = new();
    bool _capturing;

    void StartCapture_Click(object sender, RoutedEventArgs e)
    {
        _capturing = true;
        _captureDown.Clear();
        _captureNames.Clear();
        CaptureStatus.Text = "キーを押してください。組み合わせを全部離すと確定します。Esc でキャンセルします。";
        CaptureBox.Visibility = Visibility.Visible;
        CaptureBox.Focus(FocusState.Programmatic);
    }

    void StopCapture()
    {
        _capturing = false;
        CaptureBox.Visibility = Visibility.Collapsed;
    }

    void CaptureBox_KeyDown(object sender, KeyRoutedEventArgs e)
    {
        if (!_capturing) return;
        e.Handled = true;
        if (e.Key == VirtualKey.Escape)
        {
            StopCapture();
            return;
        }
        if (_captureDown.Add(e.Key))
        {
            var name = KeyName(e.Key);
            if (!_captureNames.Contains(name)) _captureNames.Add(name);
            CaptureStatus.Text = string.Join(" + ", _captureNames);
        }
    }

    void CaptureBox_KeyUp(object sender, KeyRoutedEventArgs e)
    {
        if (!_capturing) return;
        e.Handled = true;
        _captureDown.Remove(e.Key);
        if (_captureDown.Count == 0 && _captureNames.Count > 0)
        {
            var trigger = new Trigger { Type = "keys" };
            foreach (var n in _captureNames) trigger.Keys.Add(n);
            AddTrigger(trigger);
            StopCapture();
        }
    }

    static string KeyName(VirtualKey key) => key switch
    {
        VirtualKey.LeftControl => "LControl",
        VirtualKey.RightControl => "RControl",
        VirtualKey.LeftShift => "LShift",
        VirtualKey.RightShift => "RShift",
        VirtualKey.LeftMenu => "LAlt",
        VirtualKey.RightMenu => "RAlt",
        VirtualKey.LeftWindows => "LWin",
        VirtualKey.RightWindows => "RWin",
        VirtualKey.Control => "Control",
        VirtualKey.Shift => "Shift",
        VirtualKey.Menu => "Alt",
        >= VirtualKey.Number0 and <= VirtualKey.Number9 =>
            ((char)('0' + ((int)key - (int)VirtualKey.Number0))).ToString(),
        >= VirtualKey.NumberPad0 and <= VirtualKey.NumberPad9 =>
            "Numpad" + ((int)key - (int)VirtualKey.NumberPad0),
        _ => key.ToString(),
    };

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
