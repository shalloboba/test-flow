using Microsoft.Win32;

namespace RadialMenu.Settings.Services;

/// <summary>Windows 起動時の自動実行(HKCU の Run キー。本体と同じ登録名)。</summary>
public static class StartupService
{
    const string RunKey = @"Software\Microsoft\Windows\CurrentVersion\Run";
    const string ValueName = "RadialMenu";

    public static bool IsEnabled
    {
        get
        {
            try
            {
                using var k = Registry.CurrentUser.OpenSubKey(RunKey);
                return k?.GetValue(ValueName) != null;
            }
            catch
            {
                return false;
            }
        }
    }

    public static void Set(bool on)
    {
        using var k = Registry.CurrentUser.CreateSubKey(RunKey);
        if (on) k.SetValue(ValueName, $"\"{HostLink.HostExe}\"");
        else k.DeleteValue(ValueName, false);
    }
}
