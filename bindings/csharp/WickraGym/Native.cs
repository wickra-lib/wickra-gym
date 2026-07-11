using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Wickra.Gym;

/// <summary>Raw P/Invoke surface for the wickra-gym C ABI.</summary>
internal static partial class Native
{
    internal const string Lib = "wickra_gym";

    /// <summary>Construct an environment handle from a spec JSON (NUL-terminated
    /// UTF-8). Returns <c>IntPtr.Zero</c> if the spec fails to parse or validate.</summary>
    [LibraryImport(Lib)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial IntPtr wickra_gym_new(byte[] specUtf8);

    /// <summary>Free an environment handle.</summary>
    [LibraryImport(Lib)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void wickra_gym_free(IntPtr handle);

    /// <summary>
    /// Apply a command JSON (NUL-terminated UTF-8), writing the response into a
    /// caller-owned buffer. Returns the response length, or a negative error code.
    /// </summary>
    [LibraryImport(Lib)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int wickra_gym_command(IntPtr handle, byte[] cmdUtf8, byte[]? outBuf, nuint cap);

    /// <summary>The library version as a static NUL-terminated string.</summary>
    [LibraryImport(Lib)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial IntPtr wickra_gym_version();
}
