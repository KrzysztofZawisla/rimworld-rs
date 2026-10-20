use crate::*;

mod harness;
mod render;

pub(crate) use harness::*;
pub(crate) use render::*;

/// Regenerates the metadata-driven subsystem wrappers (`crates/rimworld-rs/src/subsystems_extra/`) from the
/// installed game. The names come from `tools/abi-smoke --members-json`, which reads the assembly once per
/// pass and answers with every member's name, kind, argument count, declaring type and visibility — so this
/// generator never parses a layout meant for a human, and no member name is ever guessed.
///
/// With `check`, it renders the same files in memory and compares them with the recorded directory instead
/// of writing: `cargo xtask test` uses that so a stale generated module fails the gate.
pub(crate) fn gen_subsystems(check: bool) -> Result<()> {
    // A broken intra-doc link or a missing doc comment is a defect in an SDK, so rustdoc runs as a gate too.
    // The gate is the defect, not the style: a broken intra-doc link is wrong, while a missing doc comment
    // is a to-do (it stays a warning, and `cargo doc` lists what is left).

    let references = reference_source()?;
    if !references.join("Assembly-CSharp.dll").exists() {
        return Err(missing(format!(
            "no game reference assemblies in {} (run `cargo xtask setup`)",
            references.display()
        )));
    }

    let parents = ["Verse.Pawn", "Verse.Map", "Verse.Find", "Verse.Game", "RimWorld.Planet.World"];

    // Pass one: which member of which parent holds a subsystem this repository does not wrap by hand yet.
    let mut plan: Vec<(String, String, String, String)> = Vec::new();
    let mut wanted: Vec<String> = Vec::new();
    for dump in harness_members(&references, &parents.iter().map(|name| name.to_string()).collect::<Vec<_>>())? {
        for member in &dump.members {
            if member.is_static || member.kind == "method" || !member.member_type.contains('.') {
                continue;
            }

            if !wanted_subsystem(&dump.name, &member.name, &member.member_type) {
                continue;
            }

            let wrapper = wrapper_name(&member.member_type);
            if DONE.contains(&member.member_type.as_str()) || plan.iter().any(|entry| entry.0 == wrapper) {
                continue;
            }

            plan.push((wrapper, dump.name.clone(), member.member_type.clone(), member.name.clone()));
            wanted.push(member.member_type.clone());
        }
    }

    // Pass two: those subsystems, plus the objects a handle points at (which hang off no single member).
    let objects: Vec<String> = OBJECTS.iter().map(|name| name.to_string()).collect();
    wanted.extend(objects.clone());
    wanted.sort();
    wanted.dedup();

    let dumps = harness_members(&references, &wanted)?;
    let mut rows: Vec<(String, String, String, String, Vec<MemberDump>)> = Vec::new();
    for (wrapper, parent, kind, accessor) in plan {
        if let Some(dump) = find_dump(&dumps, &kind) {
            rows.push((wrapper, parent, kind, accessor, dump.members.clone()));
        }
    }

    for kind in &objects {
        if let Some(dump) = find_dump(&dumps, kind) {
            rows.push((wrapper_name(kind), String::new(), kind.clone(), String::new(), dump.members.clone()));
        }
    }

    write_subsystems(&rows, check)
}

/// The `cargo xtask test` gate for the generated wrappers: a machine without the game says why it skips,
/// and a machine with the game fails when the recorded directory no longer matches the generator.
pub(crate) fn check_generated_subsystems() -> Result<()> {
    match reference_source() {
        Ok(references) if references.join("Assembly-CSharp.dll").exists() => {
            println!("==> checking generated subsystem wrappers");
            gen_subsystems(true)
        }
        Ok(references) => {
            println!("SKIP: generated subsystem wrappers: no Assembly-CSharp.dll in {}", references.display());
            Ok(())
        }
        Err(error) => {
            println!("SKIP: generated subsystem wrappers: {error}");
            Ok(())
        }
    }
}

/// The game's types the generator wraps: an object a handle points at hangs off no single member.
pub(crate) const OBJECTS: [&str; 24] = [
    "Verse.Hediff",
    "Verse.Gene",
    "RimWorld.Ability",
    "Verse.AI.Job",
    "Verse.AI.MentalState",
    "RimWorld.Bill",
    "Verse.Zone",
    "Verse.Area",
    "Verse.Room",
    "Verse.Designation",
    "RimWorld.Fire",
    "RimWorld.Filth",
    "RimWorld.Plant",
    "RimWorld.Apparel",
    "Verse.Verb",
    "Verse.BodyPartRecord",
    "RimWorld.RoyalTitle",
    "RimWorld.Ideo",
    "RimWorld.Precept",
    "RimWorld.Quest",
    "RimWorld.Faction",
    "RimWorld.Planet.Caravan",
    "RimWorld.Planet.Settlement",
    "RimWorld.Planet.WorldObject",
];

/// The types the hand-written wrappers in `crates/rimworld-rs/src/subsystems.rs` already cover.
pub(crate) const DONE: [&str; 30] = [
    "Verse.Pawn_HealthTracker",
    "RimWorld.Pawn_NeedsTracker",
    "RimWorld.Pawn_SkillTracker",
    "RimWorld.Pawn_WorkSettings",
    "Verse.Pawn_EquipmentTracker",
    "RimWorld.Pawn_ApparelTracker",
    "Verse.Pawn_AgeTracker",
    "RimWorld.Pawn_GuestTracker",
    "RimWorld.Pawn_Ownership",
    "RimWorld.Pawn_RelationsTracker",
    "RimWorld.Pawn_RecordsTracker",
    "RimWorld.Pawn_AbilityTracker",
    "RimWorld.Pawn_TrainingTracker",
    "RimWorld.Pawn_StoryTracker",
    "RimWorld.Pawn_GeneTracker",
    "RimWorld.Pawn_IdeoTracker",
    "RimWorld.Pawn_DraftController",
    "RimWorld.Pawn_PlayerSettings",
    "RimWorld.Pawn_OutfitTracker",
    "RimWorld.Pawn_DrugPolicyTracker",
    "RimWorld.Pawn_FoodRestrictionTracker",
    "RimWorld.Pawn_RoyaltyTracker",
    "RimWorld.Pawn_PsychicEntropyTracker",
    "RimWorld.Pawn_InteractionsTracker",
    "RimWorld.Pawn_TimetableTracker",
    "RimWorld.Pawn_TraderTracker",
    "RimWorld.Pawn_RopeTracker",
    "Verse.Pawn_CarryTracker",
    "Verse.Pawn_InventoryTracker",
    "Verse.Pawn_StanceTracker",
];

/// Whether a member of a parent type is a subsystem this generator should wrap.
pub(crate) fn wanted_subsystem(parent: &str, member: &str, member_type: &str) -> bool {
    match parent {
        "Verse.Pawn" => member_type.contains("Pawn_"),
        "Verse.Map" => [
            "lordManager",
            "listerBuildings",
            "glowGrid",
            "pathing",
            "reachability",
            "cellIndices",
            "reservationManager",
            "regionAndRoomUpdater",
            "listerFilthInHomeArea",
            "physicalInteractionReservationManager",
        ]
        .contains(&member),
        "Verse.Find" => [
            "FactionManager",
            "Scenario",
            "WorldPathGrid",
            "WorldPathing",
            "AlertsReadout",
            "IdeoManager",
            "History",
            "UniqueIDsManager",
            "MusicManagerPlay",
        ]
        .contains(&member),
        "Verse.Game" => [
            "rules",
            "playSettings",
            "uniqueIDs",
            "history",
            "outfitDatabase",
            "drugPolicyDatabase",
            "foodRestrictionDatabase",
            "scenario",
        ]
        .contains(&member),
        "RimWorld.Planet.World" => {
            ["features", "components", "info", "worldPawns", "worldObjects", "grid", "factionManager", "pathing"]
                .contains(&member)
        }
        _ => false,
    }
}
