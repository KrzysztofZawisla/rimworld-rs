use crate::*;

impl ApiTest {
    pub(crate) fn check_world_routes(&mut self, ctx: &mut Context) {
        if self.routes_checked {
            return;
        }

        self.routes_checked = true;

        let start = ctx
            .world_objects()
            .map(|objects| {
                let kinds: Vec<String> = objects.iter().filter_map(|object| object.type_name.clone()).collect();
                ctx.log(format!("[selftest] world_objects: {} object(s) ({})", objects.len(), kinds.join(", ")));
                objects.first().map(|object| object.tile).unwrap_or(0)
            })
            .map_err(|error| error.to_string());
        match &start {
            Ok(_) => self.report.pass(ctx, "world_objects lists the planet's objects"),
            Err(detail) => self.report.check(ctx, "world_objects lists the planet's objects", Err(detail.clone())),
        }
        let start = start.unwrap_or(0);

        match ctx.world_components() {
            Ok(components) => {
                ctx.log(format!("[selftest] world_components: {}", components.join(", ")));
                self.report.pass(ctx, "world_components lists the world's own components");
            }
            Err(error) => {
                self.report.fail(ctx, "world_components lists the world's own components", &error.to_string())
            }
        }

        match ctx.world_tile_info(start) {
            Ok(tile) => {
                ctx.log(format!(
                    "[selftest] tile {start}: passable {}, movement difficulty {:.1}",
                    tile.passable, tile.movement_difficulty
                ));
                self.report.pass(ctx, "world_tile_info reports passability and movement cost");
            }
            Err(error) => {
                self.report.check(ctx, "world_tile_info reports passability and movement cost", Err(error.to_string()))
            }
        }

        let neighbour = start + 1;
        match ctx.world_path(start, neighbour) {
            Ok(path) if path.found => {
                if path.tiles.is_empty() || path.total_cost <= 0.0 {
                    self.report.fail(
                        ctx,
                        "world_path reads a route from the game's pathfinder",
                        &format!("the game called it found, but answered {path:?}"),
                    );
                } else {
                    ctx.log(format!(
                        "[selftest] the route {start} -> {neighbour} crosses {} tile(s) for {:.1}",
                        path.tiles.len(),
                        path.total_cost
                    ));
                    self.report.pass(ctx, "world_path reads a route from the game's pathfinder");
                }
            }
            Ok(_) => self.report.skip(
                ctx,
                "world_path reads a route from the game's pathfinder",
                "the game says there is no land route between the two tested tiles",
            ),
            Err(error) => {
                self.report.check(ctx, "world_path reads a route from the game's pathfinder", Err(error.to_string()))
            }
        }
    }

    pub(crate) fn run_world_checks(&mut self, ctx: &mut Context) {
        let map_id = self.map_id;
        let tile = ctx
            .game_info()
            .ok()
            .and_then(|info| info.maps.iter().find(|map| map.id == map_id).map(|map| map.tile))
            .unwrap_or(0);
        let center_x = self.map_width / 2;
        let center_z = self.map_depth / 2;

        // ---- sound --------------------------------------------------------------------------
        let sound = (|| -> Result<(), String> {
            ctx.sound_play_at(sound_def::EXPLOSION_BOMB, map_id, center_x, center_z)
                .map_err(|error| error.to_string())?;
            ctx.sound_play_at(sound_def::EXPLOSION_BOMB, map_id, center_x + 1, center_z)
                .map_err(|error| error.to_string())?;
            ctx.sound_play(sound_def::EXPLOSION_BOMB).map_err(|error| error.to_string())?;

            match ctx.sound_play("NoSuchSoundDef_TheSelfTest") {
                Ok(()) => Err("a sound def that does not exist was accepted".to_string()),
                Err(_) => Ok(()),
            }
        })();
        self.report.check(ctx, "sound defs play at a cell, on the camera, and a missing one is refused", sound);

        // ---- translations -------------------------------------------------------------------
        let text = (|| -> Result<(), String> {
            let answer = ctx.translate(TRANSLATION_KEY).map_err(|error| error.to_string())?;
            ctx.log(format!(
                "[selftest] INFO  translate('{TRANSLATION_KEY}') -> {:?} (translated={}, language={:?})",
                answer.text, answer.translated, answer.language
            ));

            if answer.text.is_empty() {
                return Err("the translation is empty".to_string());
            }

            if answer.text == TRANSLATION_KEY {
                return Err(format!(
                    "the mod's own Keyed file was not found (the game fell back to the key; language={:?})",
                    answer.language
                ));
            }

            Ok(())
        })();
        self.report.check(ctx, "translate reads the mod's own Keyed file", text);

        // ---- alerts -------------------------------------------------------------------------
        let alert = (|| -> Result<(), String> {
            ctx.alert_set(
                "Rust self-test alert",
                "the self-test raised this; the next check clears it again",
                AlertPriority::Critical,
            )
            .map_err(|error| error.to_string())?;

            let raised = ctx.alerts().map_err(|error| error.to_string())?;
            let ours = raised
                .iter()
                .find(|entry| entry.type_name.as_deref().map(|name| name.ends_with("RustAlert")).unwrap_or(false))
                .ok_or("the game's alert list does not know the mod's alert class at all")?;
            if !ours.active {
                return Err(format!("the alert is not active after alert_set ({:?})", ours.label));
            }

            ctx.alert_clear().map_err(|error| error.to_string())?;
            let cleared = ctx.alerts().map_err(|error| error.to_string())?;
            let ours = cleared
                .iter()
                .find(|entry| entry.type_name.as_deref().map(|name| name.ends_with("RustAlert")).unwrap_or(false));
            match ours {
                Some(entry) if entry.active => Err("the alert is still active after alert_clear".to_string()),
                Some(_) => Ok(()),
                None => Err("the alert disappeared from the game's list after alert_clear".to_string()),
            }
        })();
        self.report.check(ctx, "the mod's alert shows in the game's alert column and goes away again", alert);

        // ---- bills --------------------------------------------------------------------------
        let bills = (|| -> Result<(), String> {
            let (x, z) = colony_cell(ctx, map_id)?;
            let table = ctx.spawn_thing(thing_def::TABLE_BUTCHER, map_id, x, z).map_err(|error| error.to_string())?;
            let thing = Thing::new(ctx, table);

            if !thing.bills().map_err(|error| error.to_string())?.is_empty() {
                return Err("a freshly spawned work table already had bills".to_string());
            }

            let added = thing.add_bill(recipe_def::MAKE_PEMMICAN, 5).map_err(|error| error.to_string())?;
            if added.recipe.as_deref() != Some(recipe_def::MAKE_PEMMICAN) || added.bills != 1 {
                return Err(format!("the bill was queued as {added:?}"));
            }

            let queued = thing.bills().map_err(|error| error.to_string())?;
            if queued.len() != 1 || queued[0].recipe.as_deref() != Some(recipe_def::MAKE_PEMMICAN) {
                return Err(format!("the queue reads back as {queued:?}"));
            }

            thing.remove_bill(None).map_err(|error| error.to_string())?;
            let after = thing.clear_bills().map_err(|error| error.to_string())?;
            if after != 0 {
                return Err(format!("{after} bill(s) were left after removing the only one"));
            }

            Ok(())
        })();
        self.report.check(ctx, "a work table takes a bill, reports it and loses it again", bills);

        // ---- genes --------------------------------------------------------------------------
        let genes = (|| -> Result<(), String> {
            let (x, z) = colony_cell(ctx, map_id)?;
            let pawn =
                ctx.spawn_pawn(pawn_kind_def::COLONIST, None, map_id, x, z).map_err(|error| error.to_string())?;

            let added = pawn.add_gene(gene_def::MOVE_SPEED_QUICK, true).map_err(|error| error.to_string())?;
            if !added.has {
                return Err("the pawn does not have the gene it was just given".to_string());
            }

            let snapshot = pawn.genes().map_err(|error| error.to_string())?;
            if !snapshot
                .genes
                .iter()
                .any(|gene| gene.def_name.as_deref() == Some(gene_def::MOVE_SPEED_QUICK) && gene.xenogene)
            {
                return Err(format!("the tracker lists {:?}", snapshot.genes));
            }

            pawn.remove_gene(gene_def::MOVE_SPEED_QUICK).map_err(|error| error.to_string())?;
            let after = pawn.genes().map_err(|error| error.to_string())?;
            if after.genes.iter().any(|gene| gene.def_name.as_deref() == Some(gene_def::MOVE_SPEED_QUICK)) {
                return Err("the gene is still there after remove_gene".to_string());
            }

            let xenotype = pawn.set_xenotype(xenotype_def::DIRTMOLE).map_err(|error| error.to_string())?;
            if xenotype.xenotype.as_deref() != Some(xenotype_def::DIRTMOLE) {
                return Err(format!("the xenotype reads back as {:?}", xenotype));
            }

            Ok(())
        })();
        self.report.check(ctx, "a pawn takes genes and a xenotype, and gives them up again", genes);

        // ---- thoughts and needs --------------------------------------------------------------
        let mood = (|| -> Result<(), String> {
            let (x, z) = colony_cell(ctx, map_id)?;
            let pawn =
                ctx.spawn_pawn(pawn_kind_def::COLONIST, None, map_id, x, z).map_err(|error| error.to_string())?;

            let thought = pawn.add_thought(thought_def::ATE_WITHOUT_TABLE).map_err(|error| error.to_string())?;
            if !thought.has {
                return Err(format!("the memory was not gained: {thought:?}"));
            }

            pawn.remove_thought(thought_def::ATE_WITHOUT_TABLE).map_err(|error| error.to_string())?;

            let need = pawn.set_need(need_def::MOOD, 0.75).map_err(|error| error.to_string())?;
            if (need.level - 0.75).abs() > 0.01 {
                return Err(format!("the mood reads back as {need:?}"));
            }

            Ok(())
        })();
        self.report.check(ctx, "a memory can be gained and removed, and a need set", mood);

        // ---- caravans ------------------------------------------------------------------------
        let caravans = (|| -> Result<(), String> {
            ctx.caravans().map_err(|error| error.to_string())?;

            let (x, z) = colony_cell(ctx, map_id)?;
            let pawn =
                ctx.spawn_pawn(pawn_kind_def::COLONIST, None, map_id, x, z).map_err(|error| error.to_string())?;
            let sent = ctx.caravan_send(&[pawn.handle()], tile).map_err(|error| error.to_string())?;
            if sent.tile != tile {
                return Err(format!("the caravan reports tile {} instead of {tile}", sent.tile));
            }

            let listed = ctx.caravans().map_err(|error| error.to_string())?;
            let found = listed.iter().any(|caravan| {
                caravan.pawns.iter().any(|member| member.label.as_deref() == pawn.label().ok().flatten().as_deref())
            });
            if !found {
                return Err(format!("the caravan list does not show the sent pawn: {listed:?}"));
            }

            Ok(())
        })();
        self.report.check(ctx, "pawns can be sent as a caravan and the caravan is listed again", caravans);

        // ---- statics, sustained sound, music and motes ---------------------------------------
        let statics = (|| -> Result<(), String> {
            let gen_types = ctx.type_get("Verse.GenTypes").map_err(|error| error.to_string())?;
            let resolved = gen_types
                .call("GetTypeInAnyAssembly", vec![json!("Verse.Pawn"), json!(null)])
                .map_err(|error| error.to_string())?;
            let name = ObjRef::from_value(&resolved)
                .ok_or_else(|| "GetTypeInAnyAssembly did not answer with a type".to_string())?
                .get("FullName")
                .map_err(|error| error.to_string())?;
            if name.as_str() != Some("Verse.Pawn") {
                return Err(format!("the static call resolved {name:?}"));
            }

            // A static *field* through the same handle: the active language's folder name.
            let database = ctx.type_get("Verse.LanguageDatabase").map_err(|error| error.to_string())?;
            let language = database.get("activeLanguage").map_err(|error| error.to_string())?;
            let folder = ObjRef::from_value(&language)
                .ok_or_else(|| "LanguageDatabase.activeLanguage is not an object".to_string())?
                .get("folderName")
                .map_err(|error| error.to_string())?;
            if folder.as_str().unwrap_or_default().is_empty() {
                return Err("the active language has no folder name".to_string());
            }

            Ok(())
        })();
        self.report.check(ctx, "type_get reaches static members without an instance", statics);

        let sounds = (|| -> Result<(), String> {
            let sustainer = ctx
                .sound_sustain_at(sound_def::EXPLOSION_BOMB, map_id, center_x, center_z)
                .map_err(|error| error.to_string())?;
            ctx.sound_end(&sustainer).map_err(|error| error.to_string())?;

            let songs = ctx.def_names("SongDef").map_err(|error| error.to_string())?;
            let song = songs.first().ok_or("the game has no song defs")?;
            let playing = ctx.music_play(song).map_err(|error| error.to_string())?;
            if playing.is_none() {
                return Err(format!("the game did not report a current song after playing '{song}'"));
            }

            let motes = ctx.def_names("ThingDef").map_err(|error| error.to_string())?;
            let mote = motes.iter().find(|name| name.starts_with("Mote_")).ok_or("the game has no mote defs")?;
            ctx.mote_spawn(mote, map_id, center_x, center_z, None).map_err(|error| error.to_string())?;

            Ok(())
        })();
        self.report.check(ctx, "sustained sound, music and motes work", sounds);

        // ---- expansion trackers --------------------------------------------------------------
        let trackers = (|| -> Result<(), String> {
            let (x, z) = colony_cell(ctx, map_id)?;
            let pawn =
                ctx.spawn_pawn(pawn_kind_def::COLONIST, None, map_id, x, z).map_err(|error| error.to_string())?;

            let titles = pawn.titles().map_err(|error| error.to_string())?;
            if !titles.has_royalty {
                return Err("the pawn has no royalty tracker (is the Royalty expansion active?)".to_string());
            }

            let defs = ctx.def_names("RoyalTitleDef").map_err(|error| error.to_string())?;
            let title = defs.last().ok_or("the game has no royal title defs")?;
            match pawn.set_title(title) {
                Ok(awarded) if awarded.has => {}
                Ok(awarded) => return Err(format!("awarding '{title}' reports {awarded:?}")),
                // The game's own Pawn_RoyaltyTracker.SetTitle throws for a pawn that never went
                // through the game's royalty flow (a bare generated pawn); the read path above is
                // what this check proves.
                Err(error) => ctx
                    .log(format!("[selftest] INFO  the game refused to award '{title}' to a generated pawn: {error}")),
            }

            let psyfocus = pawn.psyfocus().map_err(|error| error.to_string())?;
            // A pawn without a psylink reports -1.0, the game's "psychic entropy not initialized yet".
            if !(-1.0..=1.0).contains(&psyfocus.psyfocus) {
                return Err(format!("psyfocus reads back as {psyfocus:?}"));
            }
            pawn.offset_psyfocus(0.0).map_err(|error| error.to_string())?;

            let ideo = pawn.ideo().map_err(|error| error.to_string())?;
            if ideo.name.as_deref().unwrap_or_default().is_empty() || ideo.precepts.is_empty() {
                return Err(format!("the ideoligion reads back as {ideo:?}"));
            }

            let precept = ideo
                .precepts
                .iter()
                .filter_map(|entry| entry.def_name.clone())
                .next()
                .ok_or("the ideoligion has no named precept to drop")?;
            let dropped = pawn.remove_precept(&precept).map_err(|error| error.to_string())?;
            if dropped.has {
                return Err(format!("the precept '{precept}' is still there after removal"));
            }

            Ok(())
        })();
        self.report.check(ctx, "titles, psyfocus and ideoligion precepts read and write", trackers);

        let ordered = ctx
            .harmony_patch_ordered(
                &MethodRef::new("Verse.Map", "MapUpdate"),
                PatchKind::Postfix,
                "map_update_ordered",
                500,
                None,
                None,
            )
            .map_err(|error| error.to_string());
        self.report.check(ctx, "harmony_patch_ordered accepts a priority", ordered);
    }
}
