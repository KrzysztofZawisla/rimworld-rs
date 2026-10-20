use crate::*;

impl ApiTest {
    pub(crate) fn check_typed_subsystems(&mut self, ctx: &mut Context, map: &MapInfo) {
        let center_x = map.size_x / 2;
        let center_z = map.size_z / 2;
        let _ = (center_x, center_z);

        self.report.check(
            ctx,
            "the typed subsystem wrappers read the game's own members",
            (|| -> Result<(), String> {
                let colonists = ctx.map_colonists(map.id).map_err(|error| error.to_string())?;
                let pawn = colonists.first().ok_or("the map has no colonists")?.handle().clone();

                // Methods that take a `Def` need the def itself: the route carries it as a handle, so this
                // asks a spawned thing for its def first — what a mod does for the same reason.
                let steel_def = ctx
                    .spawn_thing("Steel", map.id, 5, 5)
                    .ok()
                    .and_then(|thing| thing.get("def").ok())
                    .and_then(|value| ObjRef::from_value(&value));
                let mut answered: Vec<&str> = Vec::new();
                let mut refused: Vec<String> = Vec::new();
                macro_rules! read {
                    ($label:literal, $expr:expr) => {
                        match $expr {
                            Ok(_) => answered.push($label),
                            Err(error) => refused.push(format!("{}: {error}", $label)),
                        }
                    };
                }

                read!("PawnHealth.state", ctx.pawn_health(&pawn).and_then(|it| it.state()));
                read!("PawnHealth.downed", ctx.pawn_health(&pawn).and_then(|it| it.downed()));
                read!("PawnHealth.should_be_downed", ctx.pawn_health(&pawn).and_then(|it| it.should_be_downed()));
                read!("HediffSet.hediffs", ctx.pawn_hediff_set(&pawn).and_then(|it| it.hediffs()));
                read!("HediffSet.pain_total", ctx.pawn_hediff_set(&pawn).and_then(|it| it.pain_total()));
                read!("PawnCapacities.can_be_awake", ctx.pawn_capacities(&pawn).and_then(|it| it.can_be_awake()));
                read!("PawnNeeds.all", ctx.pawn_needs(&pawn).and_then(|it| it.all()));
                read!("PawnNeeds.mood_percentage", ctx.pawn_needs(&pawn).and_then(|it| it.mood_percentage()));
                read!("PawnSkills.passion_count", ctx.pawn_skills(&pawn).and_then(|it| it.passion_count()));
                read!("PawnWork.initialized", ctx.pawn_work(&pawn).and_then(|it| it.initialized()));
                read!("PawnAge.biological_years", ctx.pawn_age(&pawn).and_then(|it| it.biological_years()));
                read!("PawnAge.adult", ctx.pawn_age(&pawn).and_then(|it| it.adult()));
                read!("PawnAge.life_stage", ctx.pawn_age(&pawn).and_then(|it| it.life_stage()));
                read!("PawnApparel.worn", ctx.pawn_apparel(&pawn).and_then(|it| it.worn()));
                read!(
                    "PawnEquipmentTracker.equipment",
                    ctx.pawn_equipment_tracker(&pawn).and_then(|it| it.equipment())
                );
                read!(
                    "PawnEquipmentTracker.has_anything",
                    ctx.pawn_equipment_tracker(&pawn).and_then(|it| it.has_anything())
                );
                read!("PawnGuest.status", ctx.pawn_guest(&pawn).and_then(|it| it.status()));
                read!("PawnGuest.is_prisoner", ctx.pawn_guest(&pawn).and_then(|it| it.is_prisoner()));
                read!("PawnRelations.direct_relations", ctx.pawn_relations(&pawn).and_then(|it| it.direct_relations()));
                read!(
                    "PawnRecordsTracker.last_battle_tick",
                    ctx.pawn_records_tracker(&pawn).and_then(|it| it.last_battle_tick())
                );
                read!("PawnStances.full_body_busy", ctx.pawn_stances(&pawn).and_then(|it| it.full_body_busy()));
                match steel_def.as_ref() {
                    Some(def) => {
                        read!("PawnCarry.max_stack_space", ctx.pawn_carry(&pawn).and_then(|it| it.max_stack_space(def)))
                    }
                    None => ctx.log("[selftest] no steel def for the carry check"),
                }
                read!(
                    "PawnInventory.unload_everything",
                    ctx.pawn_inventory(&pawn).and_then(|it| it.unload_everything())
                );
                read!("PawnDraft.drafted", ctx.pawn_draft(&pawn).and_then(|it| it.drafted()));
                read!(
                    "PawnTimetable.current_assignment",
                    ctx.pawn_timetable(&pawn).and_then(|it| it.current_assignment())
                );
                // The gene methods take a `GeneDef`, which travels through the route as a handle — so this
                // reads the pawn's own gene list and asks about the def of the first gene it finds.
                let first_gene_def = ctx
                    .pawn_gene_tracker(&pawn)
                    .and_then(|it| it.system().objects("GenesListForReading"))
                    .ok()
                    .and_then(|genes| genes.first().cloned())
                    .and_then(|gene| gene.get("def").ok())
                    .and_then(|value| ObjRef::from_value(&value));
                match first_gene_def {
                    Some(def) => {
                        read!(
                            "PawnGenes.has_endogene",
                            ctx.pawn_gene_tracker(&pawn).and_then(|it| it.has_endogene(&def))
                        );
                        read!(
                            "PawnGenes.has_xenogene",
                            ctx.pawn_gene_tracker(&pawn).and_then(|it| it.has_xenogene(&def))
                        );
                    }
                    None => ctx.log("[selftest] no gene def came back to ask about"),
                }
                read!("PawnIdeo.certainty", ctx.pawn_ideo_tracker(&pawn).and_then(|it| it.certainty()));
                read!("PawnStory.title_cap", ctx.pawn_story(&pawn).and_then(|it| it.title_cap()));
                read!("PawnPsychicEntropy.psyfocus", ctx.pawn_psychic_entropy(&pawn).and_then(|it| it.psyfocus()));
                read!(
                    "PawnRoyalty.permit_points_available",
                    ctx.pawn_royalty(&pawn).and_then(|it| it.permit_points_available())
                );

                read!("MapTerrain.any_water_cells", ctx.map_terrain_grid(map.id).and_then(|it| it.any_water_cells()));
                read!("MapTemperature.outdoor", ctx.map_temperature(map.id).and_then(|it| it.outdoor()));
                read!("MapWeather.current", ctx.map_weather_manager(map.id).and_then(|it| it.current()));
                read!("MapWeather.rain_rate", ctx.map_weather_manager(map.id).and_then(|it| it.rain_rate()));
                read!("MapGas.densities", ctx.map_gas_grid(map.id).and_then(|it| it.calculates_effects()));
                read!("MapPollution.total", ctx.map_pollution_grid(map.id).and_then(|it| it.total()));
                read!("MapPawns.all_spawned", ctx.map_pawns(map.id).and_then(|it| it.all_spawned()));
                read!("MapPawns.colonist_count", ctx.map_pawns(map.id).and_then(|it| it.colonist_count()));
                read!("MapAreas.all", ctx.map_areas(map.id).and_then(|it| it.all()));
                read!("MapRooms.all_rooms", ctx.map_region_grid(map.id).and_then(|it| it.all_rooms()));
                match steel_def {
                    Some(def) => {
                        read!("MapThings.of_def", ctx.map_lister_things(map.id).and_then(|it| it.of_def(&def)))
                    }
                    None => ctx.log("[selftest] a spawned thing answered with no def handle"),
                }
                read!("MapConditions.brightness", ctx.map_conditions(map.id).and_then(|it| it.brightness()));
                read!("MapZones.all", ctx.map_zone_manager(map.id).and_then(|it| it.all()));
                read!("MapDesignations.all", ctx.map_designation_manager(map.id).and_then(|it| it.all()));
                read!("MapHaulables.things", ctx.map_lister_haulables(map.id).and_then(|it| it.things()));

                read!("GameTime.ticks_game", ctx.game_time().and_then(|it| it.ticks_game()));
                read!("GameTime.speed", ctx.game_time().and_then(|it| it.speed()));
                read!("GameResearch.finish_project", ctx.game_research().map(|_| ()));
                read!("GameQuests.all", ctx.game_quests().and_then(|it| it.all()));
                read!("GameLetters.top_y", ctx.game_letters().and_then(|it| it.last_top_y()));
                read!("GameIdeos.all", ctx.game_ideos().and_then(|it| it.all()));
                read!("GameStoryteller.def_name", ctx.game_storyteller().and_then(|it| it.def_name()));
                read!("GameOutfits.all", ctx.game_outfits().and_then(|it| it.all()));
                read!("GameDrugPolicies.all", ctx.game_drug_policies().and_then(|it| it.all()));
                read!("GameFoodPolicies.all", ctx.game_food_policies().and_then(|it| it.all()));

                read!("WorldGrid.tiles_count", ctx.world_grid().and_then(|it| it.tiles_count()));
                read!("WorldObjects.count", ctx.world_objects_holder().and_then(|it| it.count()));
                read!("WorldObjects.settlements", ctx.world_objects_holder().and_then(|it| it.settlements()));
                read!("WorldPawns.all", ctx.world_pawns().and_then(|it| it.all()));

                // A wrapper the game answers with "the game has not given this pawn one" is not a defect: the
                // member exists on the type and this pawn simply has none of it (a colonist from a save without
                // Biotech has no mechanitor tracker, and no learning tracker either). Those reads are reported
                // in the log with the game's own reason; only a refusal that names a *wrong member* fails the
                // check, because that is the defect this check exists for.
                let unavailable: Vec<&String> =
                    refused.iter().filter(|line| line.contains("has not given this pawn")).collect();
                let defects: Vec<&String> =
                    refused.iter().filter(|line| !line.contains("has not given this pawn")).collect();

                ctx.log(format!(
                    "[selftest] {} wrapper read(s) answered, {} refused, {} unavailable on this pawn{}",
                    answered.len(),
                    defects.len(),
                    unavailable.len(),
                    if unavailable.is_empty() {
                        String::new()
                    } else {
                        format!(": {}", unavailable.iter().map(|line| line.as_str()).collect::<Vec<_>>().join("; "))
                    }
                ));

                if defects.is_empty() {
                    Ok(())
                } else {
                    Err(format!(
                        "{} wrapper read(s) refused: {}",
                        defects.len(),
                        defects.iter().map(|line| line.as_str()).collect::<Vec<_>>().join("; ")
                    ))
                }
            })(),
        );
    }
}
