//! Theme: companion. Split from `context.rs`.
#![allow(unused_imports)]

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

use super::*;
use crate::{
    host,
    types::{
        AbilityChange, AgeSet, AlertInfo, AlertPriority, AllowedArea, AnimalInfo, ApparelDropped, ApparelPolicySet,
        ApparelStowed, ApparelWorn, AreaCells, AreaInfo, ArtTitle, AttackOrder, BackstorySet, BatterySet, BedSet,
        BillChange, BillInfo, BillMove, BillSet, BuildingCompleted, CaravanChange, CaravanInfo, CaravanPause,
        CaravanRoute, CellInfo, CellTemperature, ColorSpec, ConditionAdded, ConstructionCost, ContainmentRead,
        Deconstructed, DefInfo, DesignationChange, DesignationInfo, DevelopmentInfo, DownCheck, DraftState,
        EquipResult, EquipmentAdded, EquipmentDropped, FactionCreated, FactionRelationSet, FactionSet, FactionSnapshot,
        FilthInfo, FilthMade, FireInfo, FireStart, FlickResult, FogState, ForbiddenState, FuelSet, GameInfo, GameSpeed,
        GameTypeList, GasSnapshot, GeneChange, GeneSnapshot, HarmonyPatches, HeatPush, HediffAdded, HediffRemoved,
        HediffSeveritySet, HediffsCleared, HitPointState, IdeoSet, IdeoSnapshot, IngredientAdded, InspirationCleared,
        JobTarget, Jumped, KindSet, LabelSet, LetterKind, LookSet, LordChange, LordInfo, MechanitorInfo, MemberScan,
        MentalStateForced, MessageKind, MethodRef, NameSet, NeedChange, ObjRef, PartRestored, PatchEntry, PatchKind,
        PawnEquipment, PawnRecords, PlantInfo, PocketMap, PolicySet, PollutionSet, PositionSet, PowerSnapshot,
        PreceptAdded, PreceptChange, PriceInfo, PrisonerInteraction, PrisonerStatus, PsyfocusSnapshot, PsylinkSet,
        QualitySet, QuestChange, QuestGenerated, QuestSnapshot, RadiusQuery, RecordValue, RelationSet,
        ResearchProgress, ResearchSnapshot, ResearchStarted, RitualVerdict, RoofState, RoomInfo, RoomTemperature,
        RotSet, RotationState, SaveResult, SaveScope, ScheduleSet, SettingSpec, SettlementSnapshot, SkillLearned,
        SkillPassion, SkillSet, SnowSet, StackSet, StorageAllowChange, StoragePriority, StoragePriorityChange,
        StorageSettingsInfo, StoryTitleSet, StuffSet, SurgeryAdded, TameOrder, TerrainSet, ThingComponent,
        ThingInRadius, ThingPower, ThoughtChange, TileFeature, TileMutators, TimeSnapshot, TitleChange, TitleSnapshot,
        TraderGood, TraderInfo, TrainableState, TrainingInfo, TraitAdded, TraitRemoved, Translation, TypeMembers,
        Uninstalled, UnpatchResult, VerbInfo, WealthInfo, WeatherSnapshot, WorkPrioritySet, WorldLayer,
        WorldObjectCreated, WorldObjectInfo, WorldObjectRemoved, WorldRoute, WorldSnapshot, WorldTilePollution,
        WorldTileSet, WorldTileSnapshot, XenotypeChange, ZoneInfo, ZonePlant,
    },
    values::{IntVec3Ref, Map, Pawn, Thing},
    Error, Result,
};

impl Context {
    /// Calls a static member of this mod's companion C# assembly. Use it for behaviour that needs
    /// managed game types (ThingComp, JobDriver, ITab, Window, ...) or for code that is easier in
    /// C#. See `docs/companion.md`.
    pub fn companion_call(&self, type_name: &str, member: &str, args: &[Value]) -> Result<Value> {
        self.companion_call_with_params(type_name, member, &[], args)
    }
}

impl Context {
    /// Same as [`Context::companion_call`] but picks one overload by parameter type names
    /// (`&["string", "int"]`), which is required when the member has several overloads.
    pub fn companion_call_with_params(
        &self,
        type_name: &str,
        member: &str,
        param_types: &[&str],
        args: &[Value],
    ) -> Result<Value> {
        let mut request = json!({ "type": type_name, "member": member, "args": args });
        if !param_types.is_empty() {
            request["paramTypes"] = json!(param_types);
        }

        host::call("companion_call", request)
    }
}
