//! Theme: pawn_body. Split from `context.rs`.
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
    /// What a pawn carries: its equipment, the weapon in hand first.
    pub fn pawn_equipment(&self, pawn: &ObjRef) -> Result<PawnEquipment> {
        read(host::call("pawn_equipment", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// Every verb a pawn can use, its own and the ones its equipment gives it.
    pub fn pawn_verbs(&self, pawn: &ObjRef) -> Result<Vec<VerbInfo>> {
        read(host::call("pawn_verbs", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// Sends a pawn at a target: the game's `AttackMelee` job, queued the same way a player order is.
    pub fn pawn_attack(&self, pawn: &ObjRef, target: &ObjRef) -> Result<AttackOrder> {
        read(host::call("pawn_attack", json!({ "ref": pawn.id, "targetRef": target.id }))?)
    }
}

impl Context {
    /// Turns one trainable on or off through the game's own `SetWantedRecursive` (the public route ÔÇö bare
    /// `SetWanted` is not public in this build) and answers with the trainable's state afterwards.
    pub fn pawn_set_training(&self, pawn: &ObjRef, trainable: &str, wanted: bool) -> Result<TrainableState> {
        read(host::call("pawn_set_training", json!({ "ref": pawn.id, "trainable": trainable, "wanted": wanted }))?)
    }
}

impl Context {
    /// Runs one training step through the game's own `Train`, with the given trainer and optionally
    /// completing the trainable, and answers with the steps and the learned flag the game reports.
    pub fn pawn_train_step(
        &self,
        pawn: &ObjRef,
        trainable: &str,
        trainer: &ObjRef,
        complete: bool,
    ) -> Result<TrainableState> {
        read(host::call(
            "pawn_train_step",
            json!({
                "ref": pawn.id,
                "trainable": trainable,
                "trainerRef": trainer.id,
                "complete": complete,
            }),
        )?)
    }
}

impl Context {
    /// The records the game keeps about a pawn that are not zero ÔÇö the same numbers the game itself uses for
    /// raids, studies and history, read through `Pawn_RecordsTracker.GetValue`.
    pub fn pawn_records(&self, pawn: &ObjRef) -> Result<PawnRecords> {
        read(host::call("pawn_records", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// Adds to one of a pawn's records through the game's own `AddTo`; the answer carries the value the game
    /// kept and how much actually moved, so a silent no-op is impossible.
    pub fn pawn_record_add(&self, pawn: &ObjRef, record: &str, amount: f32) -> Result<RecordValue> {
        read(host::call("pawn_record_add", json!({ "ref": pawn.id, "record": record, "amount": amount }))?)
    }
}

impl Context {
    /// Sets which drug policy a pawn follows through the game's own `CurrentPolicy` setter, named by the
    /// label the game's dialog shows; an empty label asks for the game's default, and a label it does not
    /// have is refused **with the ones it does**.
    pub fn pawn_set_drug_policy(&self, pawn: &ObjRef, policy: &str) -> Result<PolicySet> {
        read(host::call("pawn_set_drug_policy", json!({ "ref": pawn.id, "drugPolicy": policy }))?)
    }
}

impl Context {
    /// Sets which food policy a pawn follows through the game's own `CurrentFoodPolicy` (the member really is
    /// named that, not `CurrentFoodRestriction`), named the same way; a label the game does not have is
    /// refused **with the ones it does**.
    pub fn pawn_set_food_policy(&self, pawn: &ObjRef, policy: &str) -> Result<PolicySet> {
        read(host::call("pawn_set_food_policy", json!({ "ref": pawn.id, "foodPolicy": policy }))?)
    }
}

impl Context {
    /// Adds a hediff to a pawn through the game's own health tracker and answers with what the game
    /// created. `part` is a `BodyPartDef` name; a pawn without that part is refused with the reason.
    pub fn pawn_hediff_add(&self, pawn: &ObjRef, hediff: &str, part: Option<&str>) -> Result<HediffAdded> {
        read(host::call("pawn_hediff_add", json!({ "ref": pawn.id, "hediff": hediff, "part": part }))?)
    }
}

impl Context {
    /// Removes every hediff of one definition and answers with how many went and how many are left.
    pub fn pawn_hediff_remove(&self, pawn: &ObjRef, hediff: &str) -> Result<HediffRemoved> {
        read(host::call("pawn_hediff_remove", json!({ "ref": pawn.id, "hediff": hediff }))?)
    }
}

impl Context {
    /// Puts a freshly made apparel item on a pawn with the game's own apparel tracker.
    pub fn pawn_wear_apparel(&self, pawn: &ObjRef, apparel: &str) -> Result<ApparelWorn> {
        read(host::call("pawn_wear_apparel", json!({ "ref": pawn.id, "apparel": apparel }))?)
    }
}

impl Context {
    /// Takes apparel off a pawn and answers with whether it still wears it.
    pub fn pawn_remove_apparel(&self, pawn: &ObjRef, apparel: &ObjRef) -> Result<ApparelWorn> {
        read(host::call("pawn_remove_apparel", json!({ "ref": pawn.id, "apparelRef": apparel.id }))?)
    }
}

impl Context {
    /// Sets one hour of a pawn's day to a time assignment (`TimeAssignmentDef`).
    pub fn pawn_set_schedule(&self, pawn: &ObjRef, hour: i32, assignment: &str) -> Result<ScheduleSet> {
        read(host::call("pawn_set_schedule", json!({ "ref": pawn.id, "hour": hour, "assignment": assignment }))?)
    }
}

impl Context {
    /// Renames a pawn; a triple keeps its first and last name and takes the new one as its nickname.
    pub fn pawn_set_name(&self, pawn: &ObjRef, name: &str) -> Result<NameSet> {
        read(host::call("pawn_set_name", json!({ "ref": pawn.id, "name": name }))?)
    }
}

impl Context {
    /// Sets a pawn's biological age in years through the game's own age tracker.
    pub fn pawn_set_age(&self, pawn: &ObjRef, years: f32) -> Result<AgeSet> {
        read(host::call("pawn_set_age", json!({ "ref": pawn.id, "years": years }))?)
    }
}

impl Context {
    /// Moves a pawn into another faction (named the way the faction reads answer) and answers where it
    /// ended up.
    pub fn pawn_set_faction(&self, pawn: &ObjRef, faction: &str) -> Result<FactionSet> {
        read(host::call("pawn_set_faction", json!({ "ref": pawn.id, "faction": faction }))?)
    }
}

impl Context {
    /// Drafts or undrafts a pawn through the game's own `Pawn_DraftController`.
    pub fn pawn_set_draft(&self, pawn: &ObjRef, drafted: bool) -> Result<DraftState> {
        read(host::call("pawn_set_draft", json!({ "ref": pawn.id, "drafted": drafted }))?)
    }
}

impl Context {
    /// Adds a trait of one def at one degree; a pawn that already has it is refused with the reason.
    pub fn pawn_add_trait(&self, pawn: &ObjRef, trait_def: &str, degree: i32) -> Result<TraitAdded> {
        read(host::call("pawn_add_trait", json!({ "ref": pawn.id, "trait": trait_def, "degree": degree }))?)
    }
}

impl Context {
    /// Removes every trait of one def and answers with how many went and how many are left.
    pub fn pawn_remove_trait(&self, pawn: &ObjRef, trait_def: &str) -> Result<TraitRemoved> {
        read(host::call("pawn_remove_trait", json!({ "ref": pawn.id, "trait": trait_def }))?)
    }
}

impl Context {
    /// Restricts a pawn to a named area of its map through the game's player settings; `None` lifts it.
    pub fn pawn_set_allowed_area(&self, pawn: &ObjRef, area: Option<&str>) -> Result<AllowedArea> {
        read(host::call("pawn_set_allowed_area", json!({ "ref": pawn.id, "area": area }))?)
    }
}

impl Context {
    /// Changes a pawn's kind through the game's own `ChangeKind` (its role, not its race). A kind of
    /// another race is refused with the reason: the game has no public way to change a living pawn's race.
    pub fn pawn_set_kind(&self, pawn: &ObjRef, kind: &str) -> Result<KindSet> {
        read(host::call("pawn_set_kind", json!({ "ref": pawn.id, "kind": kind }))?)
    }
}

impl Context {
    /// Sets a pawn's hair (an empty name clears it) and an optional `#RRGGBB` colour, and answers with
    /// the look the pawn has afterwards.
    pub fn pawn_set_hair(&self, pawn: &ObjRef, hair: Option<&str>, color: Option<&str>) -> Result<LookSet> {
        read(host::call("pawn_set_hair", json!({ "ref": pawn.id, "hair": hair, "color": color }))?)
    }
}

impl Context {
    /// Sets a pawn's beard (an empty name clears it) and answers with the look it has afterwards.
    pub fn pawn_set_beard(&self, pawn: &ObjRef, beard: Option<&str>) -> Result<LookSet> {
        read(host::call("pawn_set_beard", json!({ "ref": pawn.id, "beard": beard }))?)
    }
}

impl Context {
    /// Grants one ability through the game's own ability tracker and answers with the list.
    pub fn pawn_gain_ability(&self, pawn: &ObjRef, ability: &str) -> Result<AbilityChange> {
        read(host::call("pawn_gain_ability", json!({ "ref": pawn.id, "ability": ability }))?)
    }
}

impl Context {
    /// Takes one ability away through the game's own ability tracker and answers with the list.
    pub fn pawn_remove_ability(&self, pawn: &ObjRef, ability: &str) -> Result<AbilityChange> {
        read(host::call("pawn_remove_ability", json!({ "ref": pawn.id, "ability": ability }))?)
    }
}

impl Context {
    /// Gives two pawns a direct relation through the game's own `AddDirectRelation`.
    pub fn pawn_set_relation(&self, pawn: &ObjRef, relation: &str, other: &ObjRef) -> Result<RelationSet> {
        read(host::call("pawn_set_relation", json!({ "ref": pawn.id, "relation": relation, "otherRef": other.id }))?)
    }
}

impl Context {
    /// Marks a prisoner released (`true`) or taken back, through the game's own guest tracker.
    pub fn pawn_set_released(&self, pawn: &ObjRef, released: bool) -> Result<PrisonerStatus> {
        read(host::call("pawn_set_released", json!({ "ref": pawn.id, "released": released }))?)
    }
}

impl Context {
    /// Allows or forbids recruiting a prisoner, through the game's own guest tracker.
    pub fn pawn_set_recruitable(&self, pawn: &ObjRef, recruitable: bool) -> Result<PrisonerStatus> {
        read(host::call("pawn_set_recruitable", json!({ "ref": pawn.id, "recruitable": recruitable }))?)
    }
}

impl Context {
    /// Moves everything a pawn wears into its own inventory through the game's own
    /// `DropAllOrMoveAllToInventory`, and answers with what it wears afterwards ÔÇö `stowed` is counted before
    /// and after, so a move the game refused is visible.
    pub fn pawn_stow_apparel(&self, pawn: &ObjRef) -> Result<ApparelStowed> {
        read(host::call("pawn_stow_apparel", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// Puts a weapon into a pawn's hands through the game's own `AddEquipment`. The weapon is made and
    /// spawned at the pawn's feet first, so nothing appears from nowhere; a def the game does not have, and
    /// a def that is no weapon, are both refused by name.
    pub fn pawn_add_equipment(&self, pawn: &ObjRef, equipment: &str) -> Result<EquipmentAdded> {
        read(host::call("pawn_add_equipment", json!({ "ref": pawn.id, "equipment": equipment }))?)
    }
}

impl Context {
    /// Asks the game whether a hediff would put a pawn down ÔÇö through `WouldBeDownedAfterAddingHediff`, so
    /// **nothing is applied** ÔÇö and answers with that verdict plus whether the pawn is down right now. The
    /// question mods ask before harming a pawn.
    pub fn pawn_would_be_downed(&self, pawn: &ObjRef, hediff: &str, severity: f32) -> Result<DownCheck> {
        read(host::call("pawn_would_be_downed", json!({ "ref": pawn.id, "hediff": hediff, "severity": severity }))?)
    }
}

impl Context {
    /// Queues a surgery on a pawn through the game's own `surgeryBills` stack: the recipe is named, the bill
    /// is built here, and the game's own stack takes it. A recipe the game does not have, and one that is no
    /// surgery, are both refused by name.
    pub fn pawn_add_surgery(&self, pawn: &ObjRef, recipe: &str) -> Result<SurgeryAdded> {
        read(host::call("pawn_add_surgery", json!({ "ref": pawn.id, "recipe": recipe }))?)
    }
}

impl Context {
    /// Takes every hediff off a pawn through the game's own `RemoveAllHediffs` ÔÇö a full heal ÔÇö and answers
    /// with how many the game took and how many it still carries.
    pub fn pawn_clear_hediffs(&self, pawn: &ObjRef) -> Result<HediffsCleared> {
        read(host::call("pawn_clear_hediffs", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// Makes a pawn drop what it is holding through the game's own `TryDropEquipment` and answers with the
    /// game's own verdict on whether it took, what went and what the pawn holds now. A pawn with no
    /// equipment tracker is refused with its type name.
    pub fn pawn_drop_equipment(&self, pawn: &ObjRef, forbid: bool) -> Result<EquipmentDropped> {
        read(host::call("pawn_drop_equipment", json!({ "ref": pawn.id, "forbid": forbid }))?)
    }
}

impl Context {
    /// Hands back a pawn's own tracker as a handle ÔÇö `skills`, `needs`, `health`, `story`, `guest`, `ideo`,
    /// `relations`, `records`, `training` and the rest of its public members. Everything the tracker exposes
    /// then works through [`ObjRef::get`]/[`ObjRef::set`]/[`ObjRef::call`], so one call opens
    /// a hundred members instead of one op per member. A name the pawn does not have is refused by name.
    pub fn pawn_tracker(&self, pawn: &ObjRef, tracker: &str) -> Result<ObjRef> {
        let value = host::call("pawn_tracker", json!({ "ref": pawn.id, "tracker": tracker }))?;
        ObjRef::from_value(&value)
            .ok_or_else(|| crate::Error::InvalidResponse(format!("pawn_tracker returned no object for {tracker}")))
    }
}

impl Context {
    /// Makes a pawn drop everything it wears through the game's own `DropAll`, at its own feet, optionally
    /// forbidden, and answers with how many pieces went and how many it still wears ÔÇö so a drop the game
    /// refused is visible.
    pub fn pawn_drop_apparel(&self, pawn: &ObjRef, forbid: bool) -> Result<ApparelDropped> {
        read(host::call("pawn_drop_apparel", json!({ "ref": pawn.id, "forbid": forbid }))?)
    }
}

impl Context {
    /// Restores one body part ÔÇö or every part, when `part` is `None`, which is the game's own meaning of a
    /// missing part ÔÇö through `Pawn_HealthTracker.RestorePart`, and answers with how many hediffs the pawn
    /// still carries. A part the pawn does not have is refused by name.
    pub fn pawn_restore_part(&self, pawn: &ObjRef, part: Option<&str>) -> Result<PartRestored> {
        read(host::call("pawn_restore_part", json!({ "ref": pawn.id, "part": part.unwrap_or("") }))?)
    }
}

impl Context {
    /// Sets a pawn's childhood and/or adulthood backstory through the game's own setters. Empty names are
    /// left alone ÔÇö the game always wants both ÔÇö and a name it does not know is refused by name. The answer
    /// says what the pawn carries afterwards.
    pub fn pawn_set_backstory(&self, pawn: &ObjRef, childhood: &str, adulthood: &str) -> Result<BackstorySet> {
        read(host::call(
            "pawn_set_backstory",
            json!({ "ref": pawn.id, "childhood": childhood, "adulthood": adulthood }),
        )?)
    }
}

impl Context {
    /// Sets the story title a pawn carries ÔÇö the word the game shows next to the name, which is **not** the
    /// royal title [`Context::pawn_set_title`] grants ÔÇö and answers with what the game kept; an empty title
    /// clears it. A pawn with no story tracker is refused with its type name.
    pub fn pawn_set_story_title(&self, pawn: &ObjRef, title: &str) -> Result<StoryTitleSet> {
        read(host::call("pawn_set_story_title", json!({ "ref": pawn.id, "title": title }))?)
    }
}

impl Context {
    /// Sets the severity of every hediff of one def through the game's own setter ÔÇö the number the game
    /// scales an injury with ÔÇö and answers with what the first one reports. A pawn that carries no such
    /// hediff is refused by name.
    pub fn pawn_hediff_set_severity(&self, pawn: &ObjRef, hediff: &str, severity: f32) -> Result<HediffSeveritySet> {
        read(host::call("pawn_hediff_set_severity", json!({ "ref": pawn.id, "hediff": hediff, "severity": severity }))?)
    }
}

impl Context {
    /// Sets which apparel policy a pawn follows, through the game's own `CurrentApparelPolicy` setter. The
    /// policy is named by the label the game's outfit dialog shows; an empty label asks for the game's
    /// default, and a label the game does not have is refused with the ones it does.
    pub fn pawn_set_apparel_policy(&self, pawn: &ObjRef, outfit: &str) -> Result<ApparelPolicySet> {
        read(host::call("pawn_set_apparel_policy", json!({ "ref": pawn.id, "outfit": outfit }))?)
    }
}

impl Context {
    /// Puts a mech under a mechanitor's control through the game's own `AssignPawnControlGroup`, with the
    /// work mode named (or the game's own default when the name is empty), and answers with the state
    /// afterwards ÔÇö the same shape [`Context::pawn_mechanitor`] reports.
    pub fn pawn_mechanitor_assign(
        &self,
        mechanitor: &ObjRef,
        mech: &ObjRef,
        work_mode: &str,
    ) -> Result<MechanitorInfo> {
        read(host::call(
            "pawn_mechanitor_assign",
            json!({ "ref": mechanitor.id, "mechRef": mech.id, "workMode": work_mode }),
        )?)
    }
}

impl Context {
    /// Takes a mech out of whatever control group it is in, through the game's own
    /// `UnassignPawnFromAnyControlGroup`, and answers with the state afterwards.
    pub fn pawn_mechanitor_unassign(&self, mechanitor: &ObjRef, mech: &ObjRef) -> Result<MechanitorInfo> {
        read(host::call("pawn_mechanitor_unassign", json!({ "ref": mechanitor.id, "mechRef": mech.id }))?)
    }
}

impl Context {
    /// What a mechanitor commands, through the game's own `Pawn_MechanitorTracker`: the mechs it oversees
    /// and controls and the bandwidth they cost. A pawn that is no mechanitor is refused with its type
    /// name. A read, so it is allowed while the game saves.
    pub fn pawn_mechanitor(&self, pawn: &ObjRef) -> Result<MechanitorInfo> {
        read(host::call("pawn_mechanitor", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// Where a pawn is in life (newborn, child, adult) and whether the game would let it feed a baby right
    /// now, with the game's own reason when not. A read, so it is allowed while the game saves ÔÇö the acts
    /// themselves are the game's jobs.
    pub fn pawn_development(&self, pawn: &ObjRef) -> Result<DevelopmentInfo> {
        read(host::call("pawn_development", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// What a pawn has learned and what it is set to learn, through the game's own training tracker reads.
    /// Setting the wish is not public in this build, so this is a read. A pawn without the tracker is
    /// refused with its type name.
    pub fn pawn_training(&self, pawn: &ObjRef) -> Result<TrainingInfo> {
        read(host::call("pawn_training", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// Sets what the colony may do with a prisoner through the game's own guest tracker. `exclusive` picks
    /// `SetExclusiveInteraction` over `ToggleNonExclusiveInteraction`. The answer carries whether the pawn
    /// is a prisoner at all, so a colonist's no-op is visible instead of silent.
    pub fn pawn_set_prisoner_interaction(
        &self,
        pawn: &ObjRef,
        interaction: &str,
        exclusive: bool,
    ) -> Result<PrisonerInteraction> {
        read(host::call(
            "pawn_set_prisoner_interaction",
            json!({ "ref": pawn.id, "interaction": interaction, "exclusive": exclusive }),
        )?)
    }
}

impl Context {
    /// What a pawn is as an animal: its kind, whether it is tamed, whether the game would let it be
    /// tamed, its life stage and gender.
    pub fn pawn_animal(&self, pawn: &ObjRef) -> Result<AnimalInfo> {
        read(host::call("pawn_animal", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// Sends a pawn to tame an animal with the game's own `Tame` job. The game's `TameUtility.CanTame`
    /// decides: an animal it says no to is refused with the reason.
    pub fn pawn_tame_order(&self, tamer: &ObjRef, animal: &ObjRef) -> Result<TameOrder> {
        read(host::call("pawn_tame_order", json!({ "ref": tamer.id, "animalRef": animal.id }))?)
    }
}

impl Context {
    /// What a trader is: its kind, its name, its currency and how much it offers. A pawn that is not a
    /// trader says so.
    pub fn pawn_trader(&self, pawn: &ObjRef) -> Result<TraderInfo> {
        read(host::call("pawn_trader", json!({ "ref": pawn.id }))?)
    }
}

impl Context {
    /// What a trader has on offer, up to `limit` (512 at most).
    pub fn pawn_trader_goods(&self, pawn: &ObjRef, limit: i32) -> Result<Vec<TraderGood>> {
        read(host::call("pawn_trader_goods", json!({ "ref": pawn.id, "limit": limit }))?)
    }
}
