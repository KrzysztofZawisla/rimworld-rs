#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::{count_reader, def_reader, flag_reader, object_reader, subsystem, text_reader, void_call, writer, *};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// What a pawn is hauling in its hands — `Pawn.carryTracker`, the game's `Verse.Pawn_CarryTracker`.
    PawnCarry,
    "Verse.Pawn",
    "Verse.Pawn_CarryTracker",
    "carryTracker",
    [
        "CarriedThing",
        "Full",
        "ParentHolder",
        "GetDirectlyHeldThings",
        "AvailableStackSpace",
        "MaxStackSpaceEver",
        "CarriedCount",
        "TryStartCarry",
        "DestroyCarriedThing",
        "CarryHandsTickInterval",
        "pawn",
    ]
);

impl<'a> PawnCarry<'a> {
    object_reader!(carried, "CarriedThing", "What the pawn is carrying (`CarriedThing`).");
}

impl<'a> PawnCarry<'a> {
    object_reader!(
        held_things,
        "GetDirectlyHeldThings",
        "The game's own holder for what is carried (`GetDirectlyHeldThings()`)."
    );
}

impl<'a> PawnCarry<'a> {
    flag_reader!(full, "Full", "Whether the hands are full (`Full`).");
}

impl<'a> PawnCarry<'a> {
    /// The most of one def the hands can hold (`MaxStackSpaceEver(def)`), by def handle.
    pub fn max_stack_space(&self, def: &ObjRef) -> Result<i64> {
        Ok(count_of(&self.system.call("MaxStackSpaceEver", vec![arg(def)])?))
    }
}

impl<'a> PawnCarry<'a> {
    /// How much room is left in the hands (`AvailableStackSpace(def)`).
    pub fn available_space(&self, thing_def: &str) -> Result<i64> {
        Ok(count_of(&self.system.call("AvailableStackSpace", vec![Value::from(thing_def.to_string())])?))
    }
}

impl<'a> PawnCarry<'a> {
    /// How many of one thing the pawn carries (`CarriedCount(def)`).
    pub fn carried_count(&self, thing_def: &str) -> Result<i64> {
        Ok(count_of(&self.system.call("CarriedCount", vec![Value::from(thing_def.to_string())])?))
    }
}

impl<'a> PawnCarry<'a> {
    /// Picks one thing up the way the game does (`TryStartCarry(thing, count)`).
    pub fn try_start_carry(&self, thing: &ObjRef, count: i64) -> Result<i64> {
        Ok(count_of(&self.system.call("TryStartCarry", vec![arg(thing), Value::from(count)])?))
    }
}

impl<'a> PawnCarry<'a> {
    /// Destroys what the pawn carries (`DestroyCarriedThing()`).
    pub fn destroy_carried(&self) -> Result<()> {
        self.system.call("DestroyCarriedThing", Vec::new())?;
        Ok(())
    }
}

subsystem!(
    /// A pawn's inventory — `Pawn.inventory`, the game's `Verse.Pawn_InventoryTracker`.
    PawnInventory,
    "Verse.Pawn",
    "Verse.Pawn_InventoryTracker",
    "inventory",
    [
        "GetDirectlyHeldThings",
        "UnloadEverything",
        "HasAnyUnloadableThing",
        "HasAnyUnpackedCaravanItems",
        "FirstUnloadableThing",
        "DrugTex",
        "Contains",
        "Count",
        "DropCount",
        "RemoveCount",
        "DestroyAll",
        "DropAllNearPawn",
        "TryAddAndUnforbid",
        "FindCombatEnhancingDrug",
        "Notify_ItemRemoved",
        "pawn",
    ]
);

impl<'a> PawnInventory<'a> {
    object_reader!(held_things, "GetDirectlyHeldThings", "Everything in the inventory (`GetDirectlyHeldThings()`).");
}

impl<'a> PawnInventory<'a> {
    object_reader!(
        first_unloadable,
        "FirstUnloadableThing",
        "The first thing the game wants unloaded (`FirstUnloadableThing`)."
    );
}

impl<'a> PawnInventory<'a> {
    flag_reader!(
        unload_everything,
        "UnloadEverything",
        "Whether the pawn should unload everything (`UnloadEverything`)."
    );
}

impl<'a> PawnInventory<'a> {
    flag_reader!(
        has_unloadable,
        "HasAnyUnloadableThing",
        "Whether anything wants unloading (`HasAnyUnloadableThing`)."
    );
}

impl<'a> PawnInventory<'a> {
    flag_reader!(
        has_unpacked_caravan_items,
        "HasAnyUnpackedCaravanItems",
        "Whether caravan items are still packed (`HasAnyUnpackedCaravanItems`)."
    );
}

impl<'a> PawnInventory<'a> {
    writer!(
        set_unload_everything,
        "UnloadEverything",
        "Asks the pawn to unload everything (`UnloadEverything = value`)."
    );
}

impl<'a> PawnInventory<'a> {
    /// Whether one thing is in the inventory (`Contains(thing)`).
    pub fn contains(&self, thing: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("Contains", vec![arg(thing)])?))
    }
}

impl<'a> PawnInventory<'a> {
    /// How many of one thing the pawn carries (`Count(thing)`).
    pub fn count(&self, thing: &ObjRef) -> Result<i64> {
        Ok(count_of(&self.system.call("Count", vec![arg(thing)])?))
    }
}

impl<'a> PawnInventory<'a> {
    /// Adds one thing to the inventory the way the game does (`TryAddAndUnforbid(thing)`).
    pub fn try_add(&self, thing: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("TryAddAndUnforbid", vec![arg(thing)])?))
    }
}

impl<'a> PawnInventory<'a> {
    /// Drops one stack on the ground near the pawn (`DropCount(thing, count, pos, forbid)`).
    pub fn drop_count(&self, thing: &ObjRef, count: i64, position: &ObjRef, forbid: bool) -> Result<()> {
        self.system.call("DropCount", vec![arg(thing), Value::from(count), arg(position), Value::from(forbid)])?;
        Ok(())
    }
}

impl<'a> PawnInventory<'a> {
    /// Removes one stack silently (`RemoveCount(thing, count)`), no dropping.
    pub fn remove_count(&self, thing: &ObjRef, count: i64) -> Result<()> {
        self.system.call("RemoveCount", vec![arg(thing), Value::from(count)])?;
        Ok(())
    }
}

impl<'a> PawnInventory<'a> {
    /// Destroys everything in the inventory (`DestroyAll()`).
    pub fn destroy_all(&self) -> Result<()> {
        self.system.call("DestroyAll", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnInventory<'a> {
    /// Drops everything near the pawn (`DropAllNearPawn(pos, forbid)`).
    pub fn drop_all_near(&self, position: &ObjRef, forbid: bool) -> Result<()> {
        self.system.call("DropAllNearPawn", vec![arg(position), Value::from(forbid)])?;
        Ok(())
    }
}

impl<'a> PawnInventory<'a> {
    /// The combat drug the game would pick for this pawn (`FindCombatEnhancingDrug()`), if any.
    pub fn find_combat_drug(&self) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("FindCombatEnhancingDrug", Vec::new())?))
    }
}

subsystem!(
    /// Ropes and hitching — `Pawn.roping`, the game's `RimWorld.Pawn_RopeTracker`.
    PawnRope,
    "Verse.Pawn",
    "RimWorld.Pawn_RopeTracker",
    "roping",
    [
        "IsRoped",
        "IsRopedByPawn",
        "IsRopedToSpot",
        "IsRopedToHitchingPost",
        "IsRopingOthers",
        "HasAnyRope",
        "RopedByPawn",
        "RopedToSpot",
        "RopedTo",
        "RopedToHitchingSpot",
        "AnyRopeesFenceBlocked",
        "InspectLine",
        "BreakAllRopes",
        "RopeToSpot",
        "TryUnropeFromHitchingSpot",
        "RopePawn",
        "DropRope",
        "DropRopes",
        "DebugString",
    ]
);

impl<'a> PawnRope<'a> {
    object_reader!(roped_by, "RopedByPawn", "The pawn roping this one (`RopedByPawn`).");
}

impl<'a> PawnRope<'a> {
    object_reader!(
        hitching_spot,
        "RopedToHitchingSpot",
        "The hitching spot the pawn is tied to (`RopedToHitchingSpot`)."
    );
}

impl<'a> PawnRope<'a> {
    text_reader!(inspect_line, "InspectLine", "The game's own rope inspect line (`InspectLine`).");
}

impl<'a> PawnRope<'a> {
    text_reader!(debug_string, "DebugString", "The game's own dump of the rope state (`DebugString()`).");
}

impl<'a> PawnRope<'a> {
    flag_reader!(is_roped, "IsRoped", "Whether the pawn is roped at all (`IsRoped`).");
}

impl<'a> PawnRope<'a> {
    flag_reader!(is_roped_by_pawn, "IsRopedByPawn", "Whether a pawn holds the rope (`IsRopedByPawn`).");
}

impl<'a> PawnRope<'a> {
    flag_reader!(is_roped_to_spot, "IsRopedToSpot", "Whether the rope is tied to a spot (`IsRopedToSpot`).");
}

impl<'a> PawnRope<'a> {
    flag_reader!(
        is_roped_to_hitching_post,
        "IsRopedToHitchingPost",
        "Whether the rope is tied to a hitching post (`IsRopedToHitchingPost`)."
    );
}

impl<'a> PawnRope<'a> {
    flag_reader!(is_roping_others, "IsRopingOthers", "Whether this pawn is roping others (`IsRopingOthers`).");
}

impl<'a> PawnRope<'a> {
    flag_reader!(has_any_rope, "HasAnyRope", "Whether any rope is involved (`HasAnyRope`).");
}

impl<'a> PawnRope<'a> {
    flag_reader!(
        any_ropees_fence_blocked,
        "AnyRopeesFenceBlocked",
        "Whether a fence blocks a ropee (`AnyRopeesFenceBlocked`)."
    );
}

impl<'a> PawnRope<'a> {
    void_call!(break_all_ropes, "BreakAllRopes", "Breaks every rope the way the game does (`BreakAllRopes()`).");
}

impl<'a> PawnRope<'a> {
    void_call!(drop_ropes, "DropRopes", "Drops every rope (`DropRopes()`).");
}

impl<'a> PawnRope<'a> {
    void_call!(drop_rope, "DropRope", "Drops one rope (`DropRope()`).");
}

impl<'a> PawnRope<'a> {
    /// Ropes the pawn to a spot (`RopeToSpot(spot)`).
    pub fn rope_to_spot(&self, spot: &ObjRef) -> Result<()> {
        self.system.call("RopeToSpot", vec![arg(spot)])?;
        Ok(())
    }
}

impl<'a> PawnRope<'a> {
    /// Ropes another pawn to this one (`RopePawn(pawn)`).
    pub fn rope_pawn(&self, other: &ObjRef) -> Result<()> {
        self.system.call("RopePawn", vec![arg(other)])?;
        Ok(())
    }
}

impl<'a> PawnRope<'a> {
    /// Unties the pawn from a hitching spot (`TryUnropeFromHitchingSpot()`).
    pub fn try_unrope_from_hitching_spot(&self) -> Result<bool> {
        Ok(flag_of(&self.system.call("TryUnropeFromHitchingSpot", Vec::new())?))
    }
}

subsystem!(
    /// A travelling trader's own business — `Pawn.trader`, the game's `RimWorld.Pawn_TraderTracker`.
    PawnTrader,
    "Verse.Pawn",
    "RimWorld.Pawn_TraderTracker",
    "trader",
    [
        "CanTradeNow",
        "TraderName",
        "RandomPriceFactorSeed",
        "traderKind",
        "GiveSoldThingToTrader",
        "GiveSoldThingToPlayer",
        "pawn",
    ]
);

impl<'a> PawnTrader<'a> {
    text_reader!(trader_name, "TraderName", "The trader's name (`TraderName`).");
}

impl<'a> PawnTrader<'a> {
    flag_reader!(can_trade_now, "CanTradeNow", "Whether the trader will trade right now (`CanTradeNow`).");
}

impl<'a> PawnTrader<'a> {
    count_reader!(
        price_seed,
        "RandomPriceFactorSeed",
        "The seed behind this trader's prices (`RandomPriceFactorSeed`)."
    );
}

impl<'a> PawnTrader<'a> {
    def_reader!(trader_kind, "traderKind", "The trader kind def (`traderKind.defName`).");
}

impl<'a> PawnTrader<'a> {
    /// Moves a sold thing to the trader (`GiveSoldThingToTrader(thing, count, seller)`).
    pub fn give_sold_thing_to_trader(&self, thing: &ObjRef, count: i64, seller: &ObjRef) -> Result<()> {
        self.system.call("GiveSoldThingToTrader", vec![arg(thing), Value::from(count), arg(seller)])?;
        Ok(())
    }
}

impl<'a> PawnTrader<'a> {
    /// Moves a sold thing to the player (`GiveSoldThingToPlayer(thing, count, buyer)`).
    pub fn give_sold_thing_to_player(&self, thing: &ObjRef, count: i64, buyer: &ObjRef) -> Result<()> {
        self.system.call("GiveSoldThingToPlayer", vec![arg(thing), Value::from(count), arg(buyer)])?;
        Ok(())
    }
}
