using System.Diagnostics;
using System.Runtime.InteropServices;

namespace RadialMenu.Settings.Services;

/// <summary>常駐本体(RadialMenu.exe)との連絡。</summary>
public static class HostLink
{
    const uint MsgReload = 0x8000 + 2; // Rust 側の MSG_RELOAD
    const string HostClass = "RadialMenuHostWindow";

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    static extern IntPtr FindWindowW(string? cls, string? title);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    static extern bool PostMessageW(IntPtr hwnd, uint msg, IntPtr wParam, IntPtr lParam);

    /// <summary>RadialMenu.exe があるフォルダー(設定アプリと同じ場所、または一つ上)。</summary>
    public static string HostDir
    {
        get
        {
            var here = AppContext.BaseDirectory;
            if (File.Exists(Path.Combine(here, "RadialMenu.exe"))) return here;
            var parent = Directory.GetParent(here.TrimEnd(Path.DirectorySeparatorChar));
            if (parent != null && File.Exists(Path.Combine(parent.FullName, "RadialMenu.exe")))
                return parent.FullName;
            return here;
        }
    }

    public static string HostExe => Path.Combine(HostDir, "RadialMenu.exe");

    public static bool IsRunning
    {
        get
        {
            try { return FindWindowW(HostClass, null) != IntPtr.Zero; }
            catch { return false; }
        }
    }

    /// <summary>設定が変わったことを本体に知らせる(本体が再読み込みする)。</summary>
    public static void NotifyReload()
    {
        try
        {
            var h = FindWindowW(HostClass, null);
            if (h != IntPtr.Zero) PostMessageW(h, MsgReload, IntPtr.Zero, IntPtr.Zero);
        }
        catch
        {
            // 通知に失敗しても保存自体は成功しているので無視する
        }
    }

    public static bool Start()
    {
        try
        {
            if (!File.Exists(HostExe)) return false;
            Process.Start(new ProcessStartInfo(HostExe) { UseShellExecute = true, WorkingDirectory = HostDir });
            return true;
        }
        catch
        {
            return false;
        }
    }
}
