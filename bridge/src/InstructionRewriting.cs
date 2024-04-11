using System;
using System.Collections;
using System.Collections.Generic;
using System.Reflection;
using System.Reflection.Emit;

namespace RustBridge;

/// <summary>One planned call rewrite: calls to <see cref="From"/> become calls to <see cref="To"/>.</summary>
internal sealed class InstructionRewrite
{
    public MethodInfo From;

    public MethodInfo To;

    /// <summary>Name of the Rust handler, for log lines only.</summary>
    public string Handler = string.Empty;

    public override string ToString()
    {
        return $"{From?.Name} -> {To?.Name ?? Handler}";
    }
}

/// <summary>
/// The pure part of the call-rewriting transpiler: walks a Harmony-style instruction list and
/// points matching <c>call</c> sites at another method. Harmony's own types are passed in
/// (<paramref name="instructionType"/>), so this logic is testable without Harmony installed.
/// </summary>
internal static class InstructionRewriting
{
    /// <summary>An instruction list with the shape of Harmony's <c>CodeInstruction</c>.</summary>
    private const string OpcodeField = "opcode";
    private const string OperandField = "operand";

    /// <summary>
    /// Rewrites the matching calls. Returns the number of replacements, or <c>-1</c> when the
    /// instruction type does not have the expected layout (the caller then leaves the method alone).
    /// </summary>
    public static int Apply(
        IList instructions,
        Type instructionType,
        IReadOnlyList<InstructionRewrite> rewrites,
        Action<string> log)
    {
        FieldInfo opcodeField = instructionType?.GetField(OpcodeField, BindingFlags.Public | BindingFlags.Instance);
        FieldInfo operandField = instructionType?.GetField(OperandField, BindingFlags.Public | BindingFlags.Instance);
        if (opcodeField == null || operandField == null)
        {
            log($"the installed Harmony version uses an unexpected CodeInstruction layout (no {OpcodeField}/{OperandField} fields); transpilers are disabled");
            return -1;
        }

        object callOpcode = CallOpcode();
        int replacements = 0;
        foreach (object instruction in instructions)
        {
            if (instruction == null || operandField.GetValue(instruction) is not MethodInfo called)
            {
                continue;
            }

            foreach (InstructionRewrite rewrite in rewrites)
            {
                if (rewrite.To == null || !SameMethod(called, rewrite.From))
                {
                    continue;
                }

                operandField.SetValue(instruction, rewrite.To);
                if (rewrite.To.IsStatic)
                {
                    // an instance call site would leave its "this" on the stack
                    opcodeField.SetValue(instruction, callOpcode);
                }

                replacements++;
                break;
            }
        }

        return replacements;
    }

    /// <summary>The `call` opcode, read reflectively because Harmony's OpCodes are not referenced.</summary>
    public static object CallOpcode()
    {
        return typeof(OpCodes).GetField(nameof(OpCodes.Call)).GetValue(null);
    }

    /// <summary>Compares two method handles: the same reflection metadata counts as the same method.</summary>
    public static bool SameMethod(MethodInfo left, MethodInfo right)
    {
        if (left == null || right == null)
        {
            return false;
        }

        if (left.Equals(right))
        {
            return true;
        }

        return left.Module == right.Module && left.MetadataToken == right.MetadataToken;
    }
}
