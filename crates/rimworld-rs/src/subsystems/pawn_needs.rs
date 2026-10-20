#![allow(unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::{
    count_reader, def_reader, flag_reader, number_reader, object_reader, objects_reader, subsystem, text_reader,
    writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// A pawn's whole need list — `Pawn.needs`, the game's `RimWorld.Pawn_NeedsTracker`. The typed fields
    /// are the needs the game itself keeps by name (`mood`, `food`, `rest`, ...), `need` looks any other up.
    PawnNeeds,
    "Verse.Pawn",
    "RimWorld.Pawn_NeedsTracker",
    "needs",
    [
        "AllNeeds",
        "mood",
        "food",
        "rest",
        "joy",
        "beauty",
        "roomsize",
        "outdoors",
        "indoors",
        "comfort",
        "learning",
        "play",
        "energy",
        "drugsDesire",
        "PrefersOutdoors",
        "PrefersIndoors",
        "TryGetNeed",
        "SetInitialLevels",
        "AddOrRemoveNeedsAsAppropriate",
    ]
);

impl<'a> PawnNeeds<'a> {
    objects_reader!(all, "AllNeeds", "Every need this pawn has (`AllNeeds`).");
}

impl<'a> PawnNeeds<'a> {
    flag_reader!(prefers_outdoors, "PrefersOutdoors", "Whether this pawn likes being outside (`PrefersOutdoors`).");
}

impl<'a> PawnNeeds<'a> {
    flag_reader!(prefers_indoors, "PrefersIndoors", "Whether this pawn likes being inside (`PrefersIndoors`).");
}

impl<'a> PawnNeeds<'a> {
    object_reader!(mood, "mood", "The mood need (`mood`).");
}

impl<'a> PawnNeeds<'a> {
    object_reader!(food, "food", "The food need (`food`).");
}

impl<'a> PawnNeeds<'a> {
    object_reader!(rest, "rest", "The rest need (`rest`).");
}

impl<'a> PawnNeeds<'a> {
    object_reader!(joy, "joy", "The joy need (`joy`).");
}

impl<'a> PawnNeeds<'a> {
    object_reader!(beauty, "beauty", "The beauty need (`beauty`).");
}

impl<'a> PawnNeeds<'a> {
    object_reader!(room_size, "roomsize", "The room size need (`roomsize`).");
}

impl<'a> PawnNeeds<'a> {
    object_reader!(outdoors, "outdoors", "The outdoors need (`outdoors`).");
}

impl<'a> PawnNeeds<'a> {
    object_reader!(indoors, "indoors", "The indoors need (`indoors`).");
}

impl<'a> PawnNeeds<'a> {
    object_reader!(comfort, "comfort", "The comfort need (`comfort`).");
}

impl<'a> PawnNeeds<'a> {
    object_reader!(learning, "learning", "The learning need (`learning`).");
}

impl<'a> PawnNeeds<'a> {
    object_reader!(play, "play", "The play need (`play`).");
}

impl<'a> PawnNeeds<'a> {
    object_reader!(energy, "energy", "The mech energy need (`energy`).");
}

impl<'a> PawnNeeds<'a> {
    object_reader!(drug_desire, "drugsDesire", "The chemical desire need (`drugsDesire`).");
}

impl<'a> PawnNeeds<'a> {
    /// One need by def name (`TryGetNeed(needDef)`), for the needs the game keeps somewhere else.
    pub fn need(&self, def_name: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("TryGetNeed", vec![Value::from(def_name.to_string())])?))
    }
}

impl<'a> PawnNeeds<'a> {
    /// Sets every need to its initial level (`SetInitialLevels()`).
    pub fn set_initial_levels(&self) -> Result<()> {
        self.system.call("SetInitialLevels", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnNeeds<'a> {
    /// Adds or removes needs as this pawn's kind, genes and hediffs require
    /// (`AddOrRemoveNeedsAsAppropriate()`).
    pub fn add_or_remove_as_appropriate(&self) -> Result<()> {
        self.system.call("AddOrRemoveNeedsAsAppropriate", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnNeeds<'a> {
    /// The mood need as a percentage, the number a player watches (the game's own `mood.CurLevelPercentage`).
    pub fn mood_percentage(&self) -> Result<f64> {
        Ok(match self.mood()? {
            Some(mood) => Need::new(self.ctx(), mood).percentage()?,
            None => 0.0,
        })
    }
}

subsystem!(
    /// A pawn's skills — `Pawn.skills`, the game's `RimWorld.Pawn_SkillTracker`.
    PawnSkills,
    "Verse.Pawn",
    "RimWorld.Pawn_SkillTracker",
    "skills",
    [
        "GetSkill",
        "Learn",
        "AverageOfRelevantSkillsFor",
        "MaxPassionOfRelevantSkillsFor",
        "PassionCount",
        "DirtyAptitudes",
        "Notify_SkillDisablesChanged",
        "SkillsTickInterval",
    ]
);

impl<'a> PawnSkills<'a> {
    count_reader!(passion_count, "PassionCount", "How many skills the pawn has a passion for (`PassionCount`).");
}

impl<'a> PawnSkills<'a> {
    /// One skill record by def name (`GetSkill(skillDef)`), for example `Shooting`.
    pub fn record(&self, skill_def: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetSkill", vec![Value::from(skill_def.to_string())])?))
    }
}

impl<'a> PawnSkills<'a> {
    /// Teaches one skill the way the game does (`Learn(skillDef, xp, direct, ignoreLearnRate)`).
    pub fn learn(&self, skill_def: &str, xp: f64, direct: bool, ignore_learn_rate: bool) -> Result<()> {
        self.system.call(
            "Learn",
            vec![
                Value::from(skill_def.to_string()),
                Value::from(xp),
                Value::from(direct),
                Value::from(ignore_learn_rate),
            ],
        )?;
        Ok(())
    }
}

impl<'a> PawnSkills<'a> {
    /// The average of the skills one work type needs (`AverageOfRelevantSkillsFor(workTypeDef)`).
    pub fn average_for_work(&self, work_type_def: &str) -> Result<f64> {
        Ok(number_of(&self.system.call("AverageOfRelevantSkillsFor", vec![Value::from(work_type_def.to_string())])?))
    }
}

impl<'a> PawnSkills<'a> {
    /// The best passion among the skills one work type needs
    /// (`MaxPassionOfRelevantSkillsFor(workTypeDef)`).
    pub fn best_passion_for_work(&self, work_type_def: &str) -> Result<Option<String>> {
        Ok(text_of(&self.system.call("MaxPassionOfRelevantSkillsFor", vec![Value::from(work_type_def.to_string())])?))
    }
}

impl<'a> PawnSkills<'a> {
    /// Rebuilds the disabled-skill caches (`DirtyAptitudes()`).
    pub fn dirty_aptitudes(&self) -> Result<()> {
        self.system.call("DirtyAptitudes", Vec::new())?;
        Ok(())
    }
}

subsystem!(
    /// One skill of a pawn — `Pawn.skills.GetSkill(...)`, the game's `RimWorld.SkillRecord`. Reached
    /// through the tracker's own `GetSkill`, so it declares no single accessor.
    SkillRecord,
    "RimWorld.Pawn_SkillTracker",
    "RimWorld.SkillRecord",
    "",
    [
        "Level",
        "GetLevel",
        "GetUnclampedLevel",
        "Learn",
        "XpProgressPercent",
        "XpTotalEarned",
        "XpRequiredForLevelUp",
        "TotallyDisabled",
        "PermanentlyDisabled",
        "LevelDescriptor",
        "Aptitude",
        "passion",
        "def",
        "EnsureMinLevelWithMargin",
        "pawn",
    ]
);

impl<'a> SkillRecord<'a> {
    count_reader!(level, "Level", "The skill's level (`Level`).");
}

impl<'a> SkillRecord<'a> {
    count_reader!(
        unclamped_level,
        "GetUnclampedLevel",
        "The level without the game's clamping (`GetUnclampedLevel()`)."
    );
}

impl<'a> SkillRecord<'a> {
    count_reader!(aptitude, "Aptitude", "The pawn's aptitude for this skill (`Aptitude`).");
}

impl<'a> SkillRecord<'a> {
    number_reader!(
        progress_percent,
        "XpProgressPercent",
        "Progress towards the next level, 0..1 (`XpProgressPercent`)."
    );
}

impl<'a> SkillRecord<'a> {
    number_reader!(xp_total, "XpTotalEarned", "Every XP ever earned in this skill (`XpTotalEarned`).");
}

impl<'a> SkillRecord<'a> {
    number_reader!(xp_required, "XpRequiredForLevelUp", "How much XP the next level costs (`XpRequiredForLevelUp`).");
}

impl<'a> SkillRecord<'a> {
    flag_reader!(
        disabled,
        "TotallyDisabled",
        "Whether the game has this skill disabled for the pawn (`TotallyDisabled`)."
    );
}

impl<'a> SkillRecord<'a> {
    flag_reader!(
        permanently_disabled,
        "PermanentlyDisabled",
        "Whether the skill is permanently disabled (`PermanentlyDisabled`)."
    );
}

impl<'a> SkillRecord<'a> {
    text_reader!(descriptor, "LevelDescriptor", "The game's own word for this level (`LevelDescriptor`).");
}

impl<'a> SkillRecord<'a> {
    text_reader!(passion, "passion", "The passion the pawn has here (`passion`), `None`/`Minor`/`Major`.");
}

impl<'a> SkillRecord<'a> {
    def_reader!(def_name, "def", "The skill def's name (`def.defName`).");
}

impl<'a> SkillRecord<'a> {
    writer!(set_level, "Level", "Sets the skill's level the way the game does (`Level = value`).");
}

impl<'a> SkillRecord<'a> {
    /// Teaches XP the way the game does (`Learn(xp, direct, ignoreLearnRate)`).
    pub fn learn(&self, xp: f64, direct: bool, ignore_learn_rate: bool) -> Result<()> {
        self.system.call("Learn", vec![Value::from(xp), Value::from(direct), Value::from(ignore_learn_rate)])?;
        Ok(())
    }
}

impl<'a> SkillRecord<'a> {
    /// Raises the skill to at least a level, with the game's own margin (`EnsureMinLevelWithMargin(minLevel)`).
    pub fn ensure_min_level(&self, min_level: i64) -> Result<()> {
        self.system.call("EnsureMinLevelWithMargin", vec![Value::from(min_level)])?;
        Ok(())
    }
}
