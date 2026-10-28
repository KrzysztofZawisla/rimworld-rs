//! Rust SDK for RimWorld mods.
//!
//! A Rust mod is a `cdylib` that exports a tiny C ABI consumed by the `RimWorldRustBridge`
//! assembly, which RimWorld loads like any other mod DLL. The bridge forwards game lifecycle
//! events into Rust and executes the host commands Rust asks for (logging, defs, spawning,
//! letters, settings, Harmony patches).
//!
//! ```no_run
//! use rimworld_rs::prelude::*;
//!
//! #[derive(Default)]
//! struct MyMod;
//!
//! impl Mod for MyMod {
//!     fn on_tick(&mut self, ctx: &mut Context) {
//!         ctx.log("ticking");
//!     }
//! }
//!
//! rimworld_rs::export_mod!(MyMod);
//! ```
//!
//! The protocol is documented in `docs/abi.md`; the C# counterpart lives in `bridge/`.

mod abi;
mod context;
mod dispatch;
mod host;
mod subsystems;
mod subsystems_context;
mod subsystems_extra;
mod subsystems_macros;
mod traits;
mod types;
mod values;

pub use abi::{Buf, HostApi, ABI_VERSION};
pub use context::Context;
pub use host::Error;
pub use subsystems::{
    GameDrugPolicies, GameFoodPolicies, GameIdeos, GameLetters, GameOutfits, GameQuests, GameResearch, GameStoryteller,
    GameTime, HediffSet, MapAreas, MapConditions, MapDesignations, MapFog, MapGas, MapHaulables, MapPawns,
    MapPollution, MapRoof, MapRooms, MapSnow, MapTemperature, MapTerrain, MapThingGrid, MapThings, MapWeather,
    MapZones, Need, PawnAbilities, PawnAge, PawnApparel, PawnCapacities, PawnCarry, PawnDraft, PawnDrugPolicy,
    PawnEquipmentTracker, PawnFoodPolicy, PawnGenes, PawnGuest, PawnHealth, PawnIdeo, PawnInteractions, PawnInventory,
    PawnNeeds, PawnOutfit, PawnOwnership, PawnPlayerSettings, PawnPsychicEntropy, PawnRecordsTracker, PawnRelations,
    PawnRope, PawnRoyalty, PawnSkills, PawnStances, PawnStory, PawnTimetable, PawnTrader, PawnTraining, PawnWork,
    SkillRecord, Subsystem, WorldGrid, WorldObjects, WorldPawns,
};
pub use subsystems_extra::*;
pub use traits::Mod;
pub use types::{
    AbilityChange, AbilitySnapshot, AgeSet, AlertInfo, AlertPriority, AllowedArea, AnimalInfo, ApparelDropped,
    ApparelPolicySet, ApparelStowed, ApparelWorn, AreaCells, AreaInfo, ArtTitle, AttackOrder, BackstorySet, BatterySet,
    BedSet, BillChange, BillCompleted, BillInfo, BillMove, BillSet, BillStarted, BuildingCompleted, CaravanChange,
    CaravanInfo, CaravanPause, CaravanPawn, CaravanRoute, CellInfo, CellTemperature, ColorSpec, ConditionAdded,
    ConstructionCost, ContainmentRead, Deconstructed, DefInfo, DesignationChange, DesignationInfo, DevelopmentInfo,
    DraftState, EquipResult, EquipmentAdded, EquipmentDropped, EquipmentInfo, FactionCreated, FactionRelationSet,
    FactionSet, FactionSnapshot, FilthInfo, FilthMade, FireInfo, FireStart, FlickResult, FogState, ForbiddenState,
    FuelSet, GameInfo, GameSpeed, GameTypeList, GasSnapshot, GeneChange, GeneInfo, GeneSnapshot, GizmoSpec,
    HarmonyPatches, HealthSnapshot, HeatPush, HediffAdded, HediffRemoved, HediffSeveritySet, HediffSnapshot,
    HediffsCleared, HitPointState, IdeoSet, IdeoSnapshot, IncidentFired, IngredientAdded, InspirationCleared,
    JobTarget, Jumped, KindSet, LabelSet, LetterKind, LetterShown, LookSet, LordChange, LordDestroyed, LordInfo,
    MapInfo, MechanitorInfo, MemberScan, MentalStateEnded, MentalStateForced, MentalStateStarted, MessageKind,
    MessageShown, MethodRef, MyPatch, NameSet, NeedChange, NeedSnapshot, ObjRef, PartRestored, PatchCall, PatchEntry,
    PatchKind, PatchOutcome, PawnDamaged, PawnDied, PawnDown, PawnEquipment, PawnInfo, PawnRecords, PlantInfo,
    PocketMap, PolicySet, PollutionSet, PositionSet, PowerNetInfo, PowerSnapshot, PreceptAdded, PreceptChange,
    PreceptInfo, PriceInfo, PrisonerInteraction, PrisonerStatus, PsyfocusSnapshot, PsylinkSet, QualitySet, QuestChange,
    QuestEnded, QuestGenerated, QuestSnapshot, RadiusQuery, RaidStarted, RecordValue, RedirectCall, RelationSet,
    RelationSnapshot, ResearchFinished, ResearchProgress, ResearchSnapshot, ResearchStarted, RitualVerdict, RoofState,
    RoomInfo, RoomTemperature, RotSet, RotationState, SaveResult, SaveScope, ScheduleSet, SettingKind, SettingSpec,
    SettlementSnapshot, SkillLearned, SkillPassion, SkillSet, SkillSnapshot, SnowSet, StackSet, StorageAllowChange,
    StoragePriority, StoragePriorityChange, StorageSettingsInfo, StoryTitleSet, StuffSet, SurgeryAdded, TameOrder,
    TerrainSet, ThingComponent, ThingDestroyed, ThingInRadius, ThingPower, ThoughtChange, TileFeature, TileMutators,
    TimeSnapshot, TitleChange, TitleInfo, TitleSnapshot, TraderGood, TraderInfo, TrainableState, TrainingInfo,
    TraitAdded, TraitRemoved, Translation, TypeMember, TypeMemberAccessors, TypeMembers, UiAction, UiWidget, UiWindow,
    Uninstalled, UnpatchResult, VerbInfo, WealthInfo, WeatherSet, WeatherSnapshot, WorkPrioritySet, WorkSnapshot,
    WorldInfo, WorldLayer, WorldObjectCreated, WorldObjectInfo, WorldObjectRemoved, WorldRoute, WorldSnapshot,
    WorldTilePollution, WorldTileSet, WorldTileSnapshot, XenotypeChange, ZoneInfo, ZoneOccupancy, ZonePlant,
};
pub use values::{IntVec3Ref, Map, Pawn, Thing, ThingComp};

/// Convenience alias for results returned by host commands.
pub type Result<T> = std::result::Result<T, Error>;

#[doc(hidden)]
pub mod __private {
    pub use crate::{
        abi::{free_buf, layout_fingerprint, Buf, HostApi, ABI_VERSION},
        dispatch::{dispatch, initialize, ModuleCell},
    };
}

/// The items a mod normally needs.
pub mod prelude {
    pub use crate::{
        export_mod, AbilityChange, AbilitySnapshot, AgeSet, AlertInfo, AlertPriority, AllowedArea, AnimalInfo,
        ApparelDropped, ApparelPolicySet, ApparelStowed, ApparelWorn, AreaCells, AreaInfo, ArtTitle, AttackOrder,
        BackstorySet, BatterySet, BedSet, BillChange, BillCompleted, BillInfo, BillMove, BillSet, BillStarted,
        BuildingCompleted, CaravanChange, CaravanInfo, CaravanPause, CaravanPawn, CaravanRoute, CellInfo,
        CellTemperature, ColorSpec, ConditionAdded, ConstructionCost, ContainmentRead, Context, Deconstructed, DefInfo,
        DesignationChange, DesignationInfo, DevelopmentInfo, DraftState, EquipResult, EquipmentAdded, EquipmentDropped,
        EquipmentInfo, FactionCreated, FactionRelationSet, FactionSet, FactionSnapshot, FilthInfo, FilthMade, FireInfo,
        FireStart, FlickResult, FogState, ForbiddenState, FuelSet, GameDrugPolicies, GameFoodPolicies, GameIdeos,
        GameInfo, GameLetters, GameOutfits, GameQuests, GameResearch, GameSpeed, GameStoryteller, GameTime,
        GameTypeList, GasSnapshot, GeneChange, GeneInfo, GeneSnapshot, GizmoSpec, HarmonyPatches, HealthSnapshot,
        HeatPush, HediffAdded, HediffRemoved, HediffSet, HediffSeveritySet, HediffSnapshot, HediffsCleared,
        HitPointState, IdeoSet, IdeoSnapshot, IncidentFired, IngredientAdded, InspirationCleared, IntVec3Ref,
        JobTarget, Jumped, KindSet, LabelSet, LetterKind, LetterShown, LookSet, LordChange, LordDestroyed, LordInfo,
        Map, MapAreas, MapConditions, MapDesignations, MapFog, MapGas, MapHaulables, MapInfo, MapPawns, MapPollution,
        MapRoof, MapRooms, MapSnow, MapTemperature, MapTerrain, MapThingGrid, MapThings, MapWeather, MapZones,
        MechanitorInfo, MemberScan, MentalStateEnded, MentalStateForced, MentalStateStarted, MessageKind, MessageShown,
        MethodRef, Mod, MyPatch, NameSet, Need, NeedChange, NeedSnapshot, ObjRef, PartRestored, PatchCall, PatchEntry,
        PatchKind, PatchOutcome, Pawn, PawnAbilities, PawnAge, PawnApparel, PawnCapacities, PawnCarry, PawnDamaged,
        PawnDied, PawnDown, PawnDraft, PawnDrugPolicy, PawnEquipment, PawnEquipmentTracker, PawnFoodPolicy, PawnGenes,
        PawnGuest, PawnHealth, PawnIdeo, PawnInfo, PawnInteractions, PawnInventory, PawnNeeds, PawnOutfit,
        PawnOwnership, PawnPlayerSettings, PawnPsychicEntropy, PawnRecords, PawnRecordsTracker, PawnRelations,
        PawnRope, PawnRoyalty, PawnSkills, PawnStances, PawnStory, PawnTimetable, PawnTrader, PawnTraining, PawnWork,
        PlantInfo, PocketMap, PolicySet, PollutionSet, PositionSet, PowerNetInfo, PowerSnapshot, PreceptAdded,
        PreceptChange, PreceptInfo, PriceInfo, PrisonerInteraction, PrisonerStatus, PsyfocusSnapshot, PsylinkSet,
        QualitySet, QuestChange, QuestEnded, QuestGenerated, QuestSnapshot, RadiusQuery, RaidStarted, RecordValue,
        RedirectCall, RelationSet, RelationSnapshot, ResearchFinished, ResearchProgress, ResearchSnapshot,
        ResearchStarted, RitualVerdict, RoofState, RoomInfo, RoomTemperature, RotSet, RotationState, SaveResult,
        SaveScope, ScheduleSet, SettingKind, SettingSpec, SettlementSnapshot, SkillLearned, SkillPassion, SkillRecord,
        SkillSet, SkillSnapshot, SnowSet, StackSet, StorageAllowChange, StoragePriority, StoragePriorityChange,
        StorageSettingsInfo, StoryTitleSet, StuffSet, Subsystem, SurgeryAdded, TameOrder, TerrainSet, Thing, ThingComp,
        ThingComponent, ThingDestroyed, ThingInRadius, ThingPower, ThoughtChange, TileFeature, TileMutators,
        TimeSnapshot, TitleChange, TitleInfo, TitleSnapshot, TraderGood, TraderInfo, TrainableState, TrainingInfo,
        TraitAdded, TraitRemoved, Translation, TypeMember, TypeMemberAccessors, TypeMembers, UiAction, UiWidget,
        UiWindow, Uninstalled, UnpatchResult, VerbInfo, WealthInfo, WeatherSet, WeatherSnapshot, WorkPrioritySet,
        WorkSnapshot, WorldGrid, WorldInfo, WorldLayer, WorldObjectCreated, WorldObjectInfo, WorldObjectRemoved,
        WorldObjects, WorldPawns, WorldRoute, WorldSnapshot, WorldTilePollution, WorldTileSet, WorldTileSnapshot,
        XenotypeChange, ZoneInfo, ZoneOccupancy, ZonePlant,
    };
}

/// Exports the module for the bridge. Use once per cdylib.
///
/// The type must implement [`Mod`]; one instance is created lazily on the first event.
#[macro_export]
macro_rules! export_mod {
    ($ty:ty) => {
        static __RWRS_MODULE: $crate::__private::ModuleCell<$ty> = $crate::__private::ModuleCell::new();

        #[no_mangle]
        pub extern "C" fn rwrs_abi_version() -> u32 {
            $crate::__private::ABI_VERSION
        }

        #[no_mangle]
        pub extern "C" fn rwrs_abi_layout() -> u64 {
            $crate::__private::layout_fingerprint()
        }

        #[no_mangle]
        pub unsafe extern "C" fn rwrs_init(host_api: *const $crate::__private::HostApi) -> u32 {
            $crate::__private::initialize(host_api)
        }

        #[no_mangle]
        pub unsafe extern "C" fn rwrs_dispatch(
            event_ptr: *const u8,
            event_len: u32,
            payload_ptr: *const u8,
            payload_len: u32,
            response_out: *mut $crate::__private::Buf,
        ) -> u32 {
            $crate::__private::dispatch::<$ty>(
                &__RWRS_MODULE,
                event_ptr,
                event_len,
                payload_ptr,
                payload_len,
                response_out,
            )
        }

        #[no_mangle]
        pub unsafe extern "C" fn rwrs_free_buf(ptr: *mut u8, len: u32) {
            $crate::__private::free_buf(ptr, len);
        }
    };
}
