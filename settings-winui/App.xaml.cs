using System.Runtime.InteropServices;
using Microsoft.UI.Xaml;
using RadialMenu.Settings.Services;

namespace RadialMenu.Settings;

public partial class App : Application
{
    public static MainWindow? MainWindow { get; private set; }
    public static ConfigStore Store { get; } = new();

    const string WindowTitle = "Radial Menu の設定";
    Mutex? _mutex;

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    static extern IntPtr FindWindowW(string? cls, string? title);
    [DllImport("user32.dll")]
    static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")]
    static extern bool ShowWindow(IntPtr hwnd, int cmd);

    public static string Title => WindowTitle;

    public App()
    {
        InitializeComponent();
        UnhandledException += (_, e) => LogError(e.Exception);
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        _mutex = new Mutex(true, @"Local\RadialMenuSettingsMutex", out var created);
        if (!created)
        {
            // すでに開いている設定画面を前に出して終了する
            var h = FindWindowW(null, WindowTitle);
            if (h != IntPtr.Zero)
            {
                ShowWindow(h, 9); // SW_RESTORE
                SetForegroundWindow(h);
            }
            Environment.Exit(0);
            return;
        }

        Store.Load();
        MainWindow = new MainWindow();
        MainWindow.Activate();
    }

    public static void LogError(Exception? e)
    {
        try
        {
            var dir = Path.Combine(
                Environment.GetEnvironmentVariable("LOCALAPPDATA") ?? Path.GetTempPath(), "RadialMenu");
            Directory.CreateDirectory(dir);
            File.AppendAllText(Path.Combine(dir, "settings-error.log"), $"[{DateTime.Now:s}] {e}\n\n");
        }
        catch
        {
        }
    }
}
