use serde_json::{json, Value};

use super::*;
use crate::{
    host,
    types::{
        ArtTitle, BatterySet, BedSet, BillChange, BillInfo, BillMove, BillSet, BuildingCompleted, ConstructionCost,
        ContainmentRead, Deconstructed, DesignationChange, EquipResult, FlickResult, ForbiddenState, FuelSet,
        HitPointState, IngredientAdded, LabelSet, ObjRef, PlantInfo, PositionSet, PriceInfo, QualitySet, RotSet,
        RotationState, StackSet, StuffSet, ThingComponent, ThingPower, Uninstalled,
    },
    values::Thing,
    Result,
};

mod building;
mod designation;
mod equipment;
mod plant;
mod query;
mod write;
