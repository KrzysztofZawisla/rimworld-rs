//! Theme: pawn_mind. Split from `subsystems.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde_json::Value;

use super::{
    count_reader, def_reader, flag_call, flag_reader, number_reader, object_reader, objects_reader, subsystem,
    text_reader, void_call, writer, *,
};
use crate::{context::Context, types::ObjRef, Result};

subsystem!(
    /// A pawn's genes and xenotype — `Pawn.genes`, the game's `RimWorld.Pawn_GeneTracker`.
    ///
    /// The `pawn_*` gene operations ([`Context::pawn_add_gene`], [`Context::pawn_remove_gene`]) do the
    /// common work with the game's own checks; this wrapper is for everything else the tracker exposes.
    PawnGenes,
    "Verse.Pawn",
    "RimWorld.Pawn_GeneTracker",
    "genes",
    [
        "Xenotype",
        "XenotypeLabel",
        "XenotypeLabelCap",
        "XenotypeDescShort",
        "XenotypeIcon",
        "UniqueXenotype",
        "CustomXenotype",
        "HasGene",
        "HasActiveGene",
        "HasXenogene",
        "HasEndogene",
        "IsXenogene",
        "GetGene",
        "GetMelaninGene",
        "GetHairColorGene",
        "AddGene",
        "RemoveGene",
        "ClearXenogenes",
        "SetXenotype",
        "SetXenotypeDirect",
        "TryGetNeedEnablingGene",
        "TryGetNeedDisablingGene",
        "EnablesNeed",
        "DisablesNeed",
        "FactorForDamage",
        "AddictionChanceFactor",
        "HediffGiversCanGive",
        "PainOffset",
        "PainFactor",
        "SocialFightChanceFactor",
        "AggroMentalBreakSelectionChanceFactor",
        "AffectedByDarkness",
        "EnjoysSunlight",
        "DontMindRawFood",
        "CanHaveBeard",
        "PrisonBreakIntervalFactor",
        "DisabledWorkTags",
        "TattoosVisible",
        "BiologicalAgeTickFactor",
        "StyleItemAllowed",
        "GeneTrackerTick",
        "Reset",
        "Notify_IngestedThing",
        "pawn",
    ]
);

impl<'a> PawnGenes<'a> {
    flag_reader!(unique_xenotype, "UniqueXenotype", "Whether the xenotype is a unique one (`UniqueXenotype`).");
}

impl<'a> PawnGenes<'a> {
    flag_reader!(
        affected_by_darkness,
        "AffectedByDarkness",
        "Whether darkness affects this pawn (`AffectedByDarkness`)."
    );
}

impl<'a> PawnGenes<'a> {
    flag_reader!(enjoys_sunlight, "EnjoysSunlight", "Whether the pawn enjoys sunlight (`EnjoysSunlight`).");
}

impl<'a> PawnGenes<'a> {
    flag_reader!(dont_mind_raw_food, "DontMindRawFood", "Whether raw food is fine for this pawn (`DontMindRawFood`).");
}

impl<'a> PawnGenes<'a> {
    flag_reader!(can_have_beard, "CanHaveBeard", "Whether this pawn can have a beard (`CanHaveBeard`).");
}

impl<'a> PawnGenes<'a> {
    flag_reader!(tattoos_visible, "TattoosVisible", "Whether tattoos show on this pawn (`TattoosVisible`).");
}

impl<'a> PawnGenes<'a> {
    number_reader!(pain_offset, "PainOffset", "Pain offset from genes (`PainOffset`).");
}

impl<'a> PawnGenes<'a> {
    number_reader!(pain_factor, "PainFactor", "Pain multiplier from genes (`PainFactor`).");
}

impl<'a> PawnGenes<'a> {
    number_reader!(
        social_fight_chance_factor,
        "SocialFightChanceFactor",
        "Social fight chance from genes (`SocialFightChanceFactor`)."
    );
}

impl<'a> PawnGenes<'a> {
    number_reader!(
        aggro_mental_break_factor,
        "AggroMentalBreakSelectionChanceFactor",
        "Aggressive break chance from genes (`AggroMentalBreakSelectionChanceFactor`)."
    );
}

impl<'a> PawnGenes<'a> {
    number_reader!(
        biological_age_tick_factor,
        "BiologicalAgeTickFactor",
        "Aging speed from genes (`BiologicalAgeTickFactor`)."
    );
}

impl<'a> PawnGenes<'a> {
    number_reader!(
        prison_break_interval_factor,
        "PrisonBreakIntervalFactor",
        "Prison break timing from genes (`PrisonBreakIntervalFactor`)."
    );
}

impl<'a> PawnGenes<'a> {
    text_reader!(xenotype_label, "XenotypeLabel", "The xenotype's label (`XenotypeLabel`).");
}

impl<'a> PawnGenes<'a> {
    text_reader!(xenotype_description, "XenotypeDescShort", "The xenotype's short description (`XenotypeDescShort`).");
}

impl<'a> PawnGenes<'a> {
    def_reader!(xenotype, "Xenotype", "The xenotype def's name (`Xenotype.defName`).");
}

impl<'a> PawnGenes<'a> {
    def_reader!(melanin_gene, "GetMelaninGene", "The gene that sets skin colour (`GetMelaninGene()`).");
}

impl<'a> PawnGenes<'a> {
    def_reader!(hair_color_gene, "GetHairColorGene", "The gene that sets hair colour (`GetHairColorGene()`).");
}

impl<'a> PawnGenes<'a> {
    /// One gene by def name (`GetGene(def)`).
    pub fn gene(&self, def_name: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetGene", vec![Value::from(def_name.to_string())])?))
    }
}

impl<'a> PawnGenes<'a> {
    /// Whether the pawn carries one endogene (`HasEndogene(geneDef)`), taking the gene def's handle.
    pub fn has_endogene(&self, gene: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("HasEndogene", vec![arg(gene)])?))
    }
}

impl<'a> PawnGenes<'a> {
    /// Whether the pawn carries one xenogene (`HasXenogene(geneDef)`), taking the gene def's handle —
    /// the game's own methods take a `GeneDef`, and a def travels through the route as a handle.
    pub fn has_xenogene(&self, gene: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("HasXenogene", vec![arg(gene)])?))
    }
}

impl<'a> PawnGenes<'a> {
    /// Whether the pawn has one gene at all (`HasGene(def)`), again by def handle.
    pub fn has_gene(&self, gene: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("HasGene", vec![arg(gene)])?))
    }
}

impl<'a> PawnGenes<'a> {
    /// Whether the pawn has one gene and it is active (`HasActiveGene(def)`).
    pub fn has_active_gene(&self, gene: &ObjRef) -> Result<bool> {
        Ok(flag_of(&self.system.call("HasActiveGene", vec![arg(gene)])?))
    }
}

impl<'a> PawnGenes<'a> {
    /// Adds one gene the way the game does (`AddGene(gene, xenogene)`).
    pub fn add_gene(&self, gene: &ObjRef, xenogene: bool) -> Result<()> {
        self.system.call("AddGene", vec![arg(gene), Value::from(xenogene)])?;
        Ok(())
    }
}

impl<'a> PawnGenes<'a> {
    /// Removes one gene (`RemoveGene(gene)`).
    pub fn remove_gene(&self, gene: &ObjRef) -> Result<()> {
        self.system.call("RemoveGene", vec![arg(gene)])?;
        Ok(())
    }
}

impl<'a> PawnGenes<'a> {
    /// Removes every xenogene (`ClearXenogenes()`).
    pub fn clear_xenogenes(&self) -> Result<()> {
        self.system.call("ClearXenogenes", Vec::new())?;
        Ok(())
    }
}

impl<'a> PawnGenes<'a> {
    /// Sets the xenotype by def name through the game's own setter (`SetXenotype(def)`).
    pub fn set_xenotype(&self, def_name: &str) -> Result<()> {
        self.system.call("SetXenotype", vec![Value::from(def_name.to_string())])?;
        Ok(())
    }
}

impl<'a> PawnGenes<'a> {
    /// Whether a need is enabled for this pawn by its genes (`EnablesNeed(def)`).
    pub fn enables_need(&self, need_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("EnablesNeed", vec![Value::from(need_def.to_string())])?))
    }
}

impl<'a> PawnGenes<'a> {
    /// Whether a need is disabled for this pawn by its genes (`DisablesNeed(def)`).
    pub fn disables_need(&self, need_def: &str) -> Result<bool> {
        Ok(flag_of(&self.system.call("DisablesNeed", vec![Value::from(need_def.to_string())])?))
    }
}

impl<'a> PawnGenes<'a> {
    /// Tells the game the pawn ate something that may touch its genes (`Notify_IngestedThing(thing, count)`).
    pub fn notify_ingested(&self, thing: &ObjRef, count: i64) -> Result<()> {
        self.system.call("Notify_IngestedThing", vec![arg(thing), Value::from(count)])?;
        Ok(())
    }
}

subsystem!(
    /// Who a pawn is: backstories, traits, title, skin and hair — `Pawn.story`, the game's
    /// `RimWorld.Pawn_StoryTracker`.
    PawnStory,
    "Verse.Pawn",
    "RimWorld.Pawn_StoryTracker",
    "story",
    [
        "Childhood",
        "Adulthood",
        "GetBackstory",
        "Title",
        "TitleCap",
        "TitleDefault",
        "TitleDefaultCap",
        "TitleShort",
        "TitleShortCap",
        "SkinColorBase",
        "SkinColor",
        "SkinColorOverriden",
        "HairColor",
        "CaresAboutOthersAppearance",
        "DisabledWorkTagsBackstoryAndTraits",
        "DisabledWorkTagsBackstoryTraitsAndGenes",
        "VoicePitchFactor",
        "IsDisturbing",
        "birthLastName",
        "favoriteColor",
        "furDef",
        "melanin",
        "traits",
        "pawn",
    ]
);

impl<'a> PawnStory<'a> {
    text_reader!(title, "Title", "The pawn's title (`Title`).");
}

impl<'a> PawnStory<'a> {
    text_reader!(title_cap, "TitleCap", "The title, capitalised (`TitleCap`).");
}

impl<'a> PawnStory<'a> {
    text_reader!(title_default, "TitleDefault", "The title the game would use by itself (`TitleDefault`).");
}

impl<'a> PawnStory<'a> {
    text_reader!(title_short, "TitleShort", "The short title (`TitleShort`).");
}

impl<'a> PawnStory<'a> {
    text_reader!(title_short_cap, "TitleShortCap", "The short title, capitalised (`TitleShortCap`).");
}

impl<'a> PawnStory<'a> {
    text_reader!(birth_last_name, "birthLastName", "The last name the pawn was born with (`birthLastName`).");
}

impl<'a> PawnStory<'a> {
    flag_reader!(
        skin_color_overridden,
        "SkinColorOverriden",
        "Whether something overrides skin colour (`SkinColorOverriden`)."
    );
}

impl<'a> PawnStory<'a> {
    flag_reader!(
        cares_about_appearance,
        "CaresAboutOthersAppearance",
        "Whether this pawn cares how others look (`CaresAboutOthersAppearance`)."
    );
}

impl<'a> PawnStory<'a> {
    flag_reader!(is_disturbing, "IsDisturbing", "Whether the pawn is disturbing to others (`IsDisturbing`).");
}

impl<'a> PawnStory<'a> {
    number_reader!(voice_pitch_factor, "VoicePitchFactor", "The pawn's voice pitch (`VoicePitchFactor`).");
}

impl<'a> PawnStory<'a> {
    number_reader!(melanin, "melanin", "The pawn's melanin (`melanin`).");
}

impl<'a> PawnStory<'a> {
    object_reader!(traits, "traits", "The pawn's trait set (`traits`).");
}

impl<'a> PawnStory<'a> {
    object_reader!(favorite_color, "favoriteColor", "The pawn's favourite colour (`favoriteColor`).");
}

impl<'a> PawnStory<'a> {
    def_reader!(childhood_backstory, "Childhood", "The childhood backstory def's name (`Childhood.defName`).");
}

impl<'a> PawnStory<'a> {
    def_reader!(adulthood_backstory, "Adulthood", "The adulthood backstory def's name (`Adulthood.defName`).");
}

impl<'a> PawnStory<'a> {
    def_reader!(fur_def, "furDef", "The pawn's fur def (`furDef`).");
}

impl<'a> PawnStory<'a> {
    writer!(set_title, "Title", "Sets the pawn's title (`Title = value`).");
}

impl<'a> PawnStory<'a> {
    writer!(set_hair_color, "HairColor", "Sets hair colour (`HairColor = value`).");
}

impl<'a> PawnStory<'a> {
    writer!(set_skin_color_base, "SkinColorBase", "Sets the base skin colour (`SkinColorBase = value`).");
}

impl<'a> PawnStory<'a> {
    /// One backstory by name — childhood or adulthood (`GetBackstory(category)`).
    pub fn backstory(&self, category: &str) -> Result<Option<ObjRef>> {
        Ok(ObjRef::from_value(&self.system.call("GetBackstory", vec![Value::from(category.to_string())])?))
    }
}

impl<'a> PawnStory<'a> {
    /// The work tags this pawn's backstory and traits disable
    /// (`DisabledWorkTagsBackstoryAndTraits`), as a number bit set.
    pub fn disabled_work_tags(&self) -> Result<i64> {
        Ok(count_of(&self.system.get("DisabledWorkTagsBackstoryAndTraits")?))
    }
}

impl<'a> PawnStory<'a> {
    /// The work tags backstory, traits and genes disable together
    /// (`DisabledWorkTagsBackstoryTraitsAndGenes`).
    pub fn disabled_work_tags_with_genes(&self) -> Result<i64> {
        Ok(count_of(&self.system.get("DisabledWorkTagsBackstoryTraitsAndGenes")?))
    }
}
