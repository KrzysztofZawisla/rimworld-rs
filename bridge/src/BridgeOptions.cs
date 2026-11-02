namespace RustBridge;

/// <summary>
/// The bridge's own options, stored in the mod's settings under keys Rust never declared. The mod options
/// window draws them above the settings the mod registered, and a write here stays silent: Rust did not
/// declare the key, so there is nothing for it to be told about.
/// </summary>
internal static class BridgeOptions
{
    /// <summary>Writes the stack trace of a failed host call into <c>Player.log</c>.</summary>
    public const string VerboseErrorsKey = "rustbridge.verboseErrors";

    /// <summary>
    /// Off by default: a failed call still answers Rust with the game's own message, and the trace is the
    /// extra detail a mod author needs while the failure is being diagnosed.
    /// </summary>
    public static bool VerboseErrors => IsTrue(RustMod.Current?.Settings?.Get(VerboseErrorsKey));

    public static void SetVerboseErrors(bool enabled)
    {
        RustMod.Current?.Settings?.SetSilent(VerboseErrorsKey, enabled ? "true" : "false");
    }

    private static bool IsTrue(string value)
    {
        return string.Equals(value, "true", System.StringComparison.OrdinalIgnoreCase);
    }
}
