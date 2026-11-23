using System.Collections.Specialized;
using System.Text;
using System.Text.Encodings.Web;
using System.Text.Json;
using System.Text.Json.Serialization;
using RadialMenu.Settings.Models;

namespace RadialMenu.Settings.Services;

/// <summary>
/// config.json の読み書き。画面で何かを変えると 250ms 後に自動保存し、本体へ再読み込みを通知する
/// (「設定を再読み込み」の操作は不要)。
/// </summary>
public sealed class ConfigStore
{
    public static readonly JsonSerializerOptions Json = new()
    {
        WriteIndented = true,
        PropertyNameCaseInsensitive = true,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        Encoder = JavaScriptEncoder.UnsafeRelaxedJsonEscaping,
    };

    SynchronizationContext? _ui;
    Timer? _timer;
    volatile bool _dirty;

    public Config Config { get; private set; } = Config.CreateDefault();

    /// <summary>直近の保存エラー(成功すると null)。</summary>
    public string? SaveError { get; private set; }

    public event Action? SaveStateChanged;

    /// <summary>Rust 側 Config::path() と同じ決め方。</summary>
    public static string ConfigPath()
    {
        var portable = Path.Combine(HostLink.HostDir, "config.json");
        if (File.Exists(portable)) return portable;
        var appData = Environment.GetEnvironmentVariable("APPDATA");
        if (!string.IsNullOrEmpty(appData))
        {
            var dir = Path.Combine(appData, "RadialMenu");
            Directory.CreateDirectory(dir);
            return Path.Combine(dir, "config.json");
        }
        return portable;
    }

    /// <summary>UI スレッドから呼ぶこと(保存を UI スレッドで行うため)。</summary>
    public void Load()
    {
        _ui = SynchronizationContext.Current;
        Config cfg;
        var path = ConfigPath();
        try
        {
            if (File.Exists(path))
            {
                var text = File.ReadAllText(path, Encoding.UTF8).TrimStart('\uFEFF');
                cfg = JsonSerializer.Deserialize<Config>(text, Json) ?? Config.CreateDefault();
            }
            else
            {
                cfg = Config.CreateDefault();
            }
        }
        catch
        {
            cfg = Config.CreateDefault();
        }
        cfg.Sanitize();
        Config = cfg;
        Hook(cfg);
        if (!File.Exists(path)) Flush();
    }

    public void ResetToDefaults()
    {
        Config.CopyFrom(Config.CreateDefault());
        Schedule();
    }

    // ---- 変更の監視 ----

    void Hook(Config c)
    {
        c.PropertyChanged += (_, _) => Schedule();
        c.Layers.CollectionChanged += (_, e) =>
        {
            if (e.Action == NotifyCollectionChangedAction.Add && e.NewItems != null)
                foreach (Layer l in e.NewItems) HookLayer(l);
            Schedule();
        };
        foreach (var l in c.Layers) HookLayer(l);
    }

    void HookLayer(Layer l)
    {
        l.PropertyChanged += (_, _) => Schedule();
        l.Items.CollectionChanged += (_, e) =>
        {
            if (e.Action == NotifyCollectionChangedAction.Add && e.NewItems != null)
                foreach (Item i in e.NewItems) i.PropertyChanged += (_, _) => Schedule();
            Schedule();
        };
        foreach (var i in l.Items) i.PropertyChanged += (_, _) => Schedule();
    }

    void Schedule()
    {
        _dirty = true;
        _timer?.Dispose();
        _timer = new Timer(_ =>
        {
            if (_ui != null) _ui.Post(__ => FlushIfDirty(), null);
            else FlushIfDirty();
        }, null, 250, Timeout.Infinite);
    }

    public void FlushIfDirty()
    {
        if (_dirty) Flush();
    }

    public void Flush()
    {
        _dirty = false;
        try
        {
            var path = ConfigPath();
            var tmp = path + ".tmp";
            File.WriteAllText(tmp, JsonSerializer.Serialize(Config, Json), new UTF8Encoding(false));
            File.Move(tmp, path, true);
            SaveError = null;
            HostLink.NotifyReload();
        }
        catch (Exception e)
        {
            SaveError = e.Message;
        }
        SaveStateChanged?.Invoke();
    }
}
