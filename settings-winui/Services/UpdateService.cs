using System.Diagnostics;
using System.Reflection;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;

namespace RadialMenu.Settings.Services;

public sealed record UpdateInfo(string Version, string Url, string Sha256, bool IsNewer);

/// <summary>
/// 更新確認と実行。更新元は「{URL}/latest.json」で、中身は {"version","url","sha256"}。
/// url は Inno Setup で作ったインストーラー(exe)。
/// </summary>
public static class UpdateService
{
    static readonly HttpClient Http = CreateClient();

    static HttpClient CreateClient()
    {
        var c = new HttpClient { Timeout = TimeSpan.FromMinutes(10) };
        c.DefaultRequestHeaders.UserAgent.ParseAdd("RadialMenu-Settings");
        return c;
    }

    public static UpdateInfo? Latest { get; private set; }
    public static string? Error { get; private set; }
    public static bool Checking { get; private set; }
    public static event Action? Changed;

    public static string CurrentVersion
    {
        get
        {
            var v = typeof(UpdateService).Assembly.GetName().Version ?? new Version(0, 0, 0);
            return $"{v.Major}.{v.Minor}.{Math.Max(v.Build, 0)}";
        }
    }

    public static async Task CheckAsync(string baseUrl)
    {
        if (Checking) return;
        Checking = true;
        Error = null;
        Changed?.Invoke();
        try
        {
            var b = (baseUrl ?? "").Trim().TrimEnd('/');
            if (b.Length == 0) throw new InvalidOperationException("更新元の URL が設定されていません");
            var bytes = await Http.GetByteArrayAsync($"{b}/latest.json");
            var text = Encoding.UTF8.GetString(bytes).TrimStart('\uFEFF');
            using var doc = JsonDocument.Parse(text);
            var root = doc.RootElement;
            var version = root.GetProperty("version").GetString() ?? "";
            var url = root.GetProperty("url").GetString() ?? "";
            var sha = root.TryGetProperty("sha256", out var s) ? s.GetString() ?? "" : "";
            if (!url.StartsWith("http://") && !url.StartsWith("https://"))
                url = $"{b}/{url.TrimStart('/')}";
            Latest = new UpdateInfo(version, url, sha, IsNewer(version, CurrentVersion));
        }
        catch (Exception e)
        {
            Latest = null;
            Error = e.Message;
        }
        finally
        {
            Checking = false;
            Changed?.Invoke();
        }
    }

    public static bool IsNewer(string latest, string current)
    {
        var a = Parts(latest);
        var b = Parts(current);
        for (var i = 0; i < Math.Max(a.Length, b.Length); i++)
        {
            var x = i < a.Length ? a[i] : 0;
            var y = i < b.Length ? b[i] : 0;
            if (x != y) return x > y;
        }
        return false;
    }

    static long[] Parts(string v) =>
        v.TrimStart('v', 'V').Split('.').Select(p =>
        {
            var digits = new string(p.TakeWhile(char.IsDigit).ToArray());
            return long.TryParse(digits, out var n) ? n : 0;
        }).ToArray();

    /// <summary>インストーラーをダウンロードして(sha256 を確認して)起動する。成功すると true。</summary>
    public static async Task<string?> DownloadAndRunAsync(UpdateInfo info, IProgress<double>? progress)
    {
        try
        {
            var name = Path.GetFileName(new Uri(info.Url).LocalPath);
            if (!name.EndsWith(".exe", StringComparison.OrdinalIgnoreCase))
                return "更新ファイルがインストーラー(.exe)ではありません。latest.json の url を確認してください。";

            var dir = Path.Combine(Path.GetTempPath(), "RadialMenu-update");
            Directory.CreateDirectory(dir);
            var path = Path.Combine(dir, name);

            using (var res = await Http.GetAsync(info.Url, HttpCompletionOption.ResponseHeadersRead))
            {
                res.EnsureSuccessStatusCode();
                var total = res.Content.Headers.ContentLength ?? 0;
                await using var src = await res.Content.ReadAsStreamAsync();
                await using var dst = File.Create(path);
                var buf = new byte[81920];
                long done = 0;
                int n;
                while ((n = await src.ReadAsync(buf)) > 0)
                {
                    await dst.WriteAsync(buf.AsMemory(0, n));
                    done += n;
                    if (total > 0) progress?.Report((double)done / total);
                }
            }

            if (info.Sha256.Length > 0)
            {
                await using var f = File.OpenRead(path);
                var hash = Convert.ToHexString(await SHA256.HashDataAsync(f));
                if (!hash.Equals(info.Sha256.Trim(), StringComparison.OrdinalIgnoreCase))
                {
                    File.Delete(path);
                    return "ダウンロードしたファイルのチェックサムが一致しません。";
                }
            }

            Process.Start(new ProcessStartInfo(path)
            {
                UseShellExecute = true,
                Arguments = "/SILENT /CLOSEAPPLICATIONS /RESTARTAPPLICATIONS",
            });
            return null;
        }
        catch (Exception e)
        {
            return e.Message;
        }
    }
}
