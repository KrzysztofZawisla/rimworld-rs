using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using RimWorld;
using UnityEngine;
using Verse;
using Verse.AI;

namespace RustBridge;

/// <summary>
/// The expansion trackers a mod asks about: Royalty titles and psyfocus, Ideology precepts. Reads are
/// cheap and always available; a game without the expansion answers with an empty list or a clear
/// error instead of a null reference.
///
/// Two game details are handled here on purpose: `Def.LabelCap` throws when a def has no `label`
/// (`RoyalTitleDef` carries gendered labels instead), so labels are read from the field with the def
/// name as a fallback, and every failure is reported with the operation and the pawn it was about.
/// </summary>
internal static partial class Trackers
{














































































}
