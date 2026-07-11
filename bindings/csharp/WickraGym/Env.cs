using System.Runtime.InteropServices;
using System.Text;

namespace Wickra.Gym;

/// <summary>
/// A deterministic, Gymnasium-compatible backtest environment over the wickra-gym
/// C ABI. Construct from a spec JSON, drive it with command JSONs (<c>load</c>,
/// <c>reset</c>, <c>step</c>, <c>spec</c>, <c>version</c>), and dispose it.
/// </summary>
public sealed class Env : IDisposable
{
    private readonly EnvHandle _handle;

    /// <summary>Construct an environment from an <c>EnvSpec</c> JSON string.</summary>
    /// <exception cref="ArgumentException">The spec fails to parse or validate.</exception>
    public Env(string specJson)
    {
        IntPtr raw = Native.wickra_gym_new(Utf8(specJson));
        if (raw == IntPtr.Zero)
        {
            throw new ArgumentException("wickra-gym: invalid spec", nameof(specJson));
        }
        _handle = new EnvHandle(raw);
    }

    /// <summary>Apply a command JSON and return the response JSON string.</summary>
    public string Command(string cmdJson)
    {
        byte[] cmd = Utf8(cmdJson);
        bool added = false;
        try
        {
            _handle.DangerousAddRef(ref added);
            IntPtr handle = _handle.DangerousGetHandle();

            // Start with a large buffer so a single call suffices. On the rare
            // re-alloc path, the core serves the cached response of the same
            // not-yet-delivered command, so a mutating command is never re-run.
            byte[] buf = new byte[65536];
            int n = Native.wickra_gym_command(handle, cmd, buf, (nuint)buf.Length);
            if (n < 0)
            {
                throw new InvalidOperationException("wickra-gym: command error");
            }
            if (n >= buf.Length)
            {
                buf = new byte[n + 1];
                n = Native.wickra_gym_command(handle, cmd, buf, (nuint)buf.Length);
                if (n < 0)
                {
                    throw new InvalidOperationException("wickra-gym: command error");
                }
            }
            return Encoding.UTF8.GetString(buf, 0, n);
        }
        finally
        {
            if (added)
            {
                _handle.DangerousRelease();
            }
        }
    }

    /// <summary>The wickra-gym version string.</summary>
    public static string Version() =>
        Marshal.PtrToStringUTF8(Native.wickra_gym_version()) ?? string.Empty;

    /// <summary>Release the native environment handle.</summary>
    public void Dispose() => _handle.Dispose();

    private static byte[] Utf8(string s)
    {
        byte[] b = Encoding.UTF8.GetBytes(s);
        Array.Resize(ref b, b.Length + 1); // NUL-terminate for the C ABI.
        return b;
    }
}

/// <summary>A <see cref="SafeHandle"/> owning a <c>WickraGymEnv*</c>.</summary>
internal sealed class EnvHandle : SafeHandle
{
    public EnvHandle(IntPtr handle)
        : base(IntPtr.Zero, ownsHandle: true) => SetHandle(handle);

    public override bool IsInvalid => handle == IntPtr.Zero;

    protected override bool ReleaseHandle()
    {
        Native.wickra_gym_free(handle);
        return true;
    }
}
