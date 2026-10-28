use serde_json::json;

use super::*;
use crate::{
    host,
    types::{
        AbilityChange, AgeSet, AllowedArea, AnimalInfo, ApparelDropped, ApparelPolicySet, ApparelStowed, ApparelWorn,
        AttackOrder, BackstorySet, DevelopmentInfo, DownCheck, DraftState, EquipmentAdded, EquipmentDropped,
        FactionSet, HediffAdded, HediffRemoved, HediffSeveritySet, HediffsCleared, KindSet, LookSet, MechanitorInfo,
        NameSet, ObjRef, PartRestored, PawnEquipment, PawnRecords, PolicySet, PrisonerInteraction, PrisonerStatus,
        RecordValue, RelationSet, ScheduleSet, StoryTitleSet, SurgeryAdded, TameOrder, TraderGood, TraderInfo,
        TrainableState, TrainingInfo, TraitAdded, TraitRemoved, VerbInfo,
    },
    Result,
};

mod animals;
mod health;
mod identity;
mod items;
mod social;
