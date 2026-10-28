using System;
using System.Collections.Generic;
using System.Text.Json;
using Verse;

namespace RustBridge;

/// <summary>
/// The gas half of <see cref="EnvironmentOps"/>: reading and adding blind smoke, tox gas, rot stink
/// and deadlife dust on a cell.
/// </summary>
internal static partial class EnvironmentOps
{
    public static object Gas(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        return GasAt(map, cell);
    }

    public static object AddGas(JsonElement request)
    {
        Map map = ResolveMap(request);
        IntVec3 cell = ResolveCell(request);
        RequireInBounds(map, cell);
        GasType type = ResolveGasType(request);
        int amount = JsonArgs.GetInt(request, "amount", 100);

        GasGrid grid = map.gasGrid;
        if (grid == null)
        {
            throw new BridgeException($"map {map.uniqueID} has no gas grid");
        }

        grid.AddGas(cell, type, amount, canOverflow: true);
        return GasAt(map, cell);
    }

    private static Dictionary<string, object> GasAt(Map map, IntVec3 cell)
    {
        GasGrid grid = map.gasGrid;
        if (grid == null)
        {
            throw new BridgeException($"map {map.uniqueID} has no gas grid");
        }

        return new Dictionary<string, object>
        {
            ["blindSmoke"] = grid.DensityAt(cell, GasType.BlindSmoke),
            ["toxGas"] = grid.DensityAt(cell, GasType.ToxGas),
            ["rotStink"] = grid.DensityAt(cell, GasType.RotStink),
            ["deadlifeDust"] = grid.DensityAt(cell, GasType.DeadlifeDust),
            ["any"] = grid.AnyGasAt(cell),
        };
    }

    private static GasType ResolveGasType(JsonElement request)
    {
        string name = JsonArgs.GetString(request, "gasType");
        if (!Enum.TryParse(name, ignoreCase: true, out GasType type))
        {
            throw new BridgeException(
                $"unknown gas '{name}' (expected one of {string.Join(", ", Enum.GetNames(typeof(GasType)))})");
        }

        return type;
    }
}
