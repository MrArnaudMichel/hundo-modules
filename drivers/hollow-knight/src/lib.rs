//! Hollow Knight: decrypts `userN.dat` and checks it against the 112 % list.
//!
//! The "112 %" group holds exactly what the game counts, so its gaps are the
//! reader's missing percents; everything else sits in "Beyond 112 %".

mod journal;
mod pickups;
mod save;
mod shops;

use hundo_sdk::{Completion, DriverError, GameMap, Input, Item, MapMarker, Report, Section, Spoiler, Stat, Status};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::HashMap;

const WIKI: &str = "https://hollowknight.wiki/w/";
/// Essence the Seer asks for before her final words.
const SEER_ESSENCE: i64 = 2400;

struct HollowKnight;

impl hundo_sdk::Guest for HollowKnight {
    fn analyze(input: Input) -> Result<Report, DriverError> {
        analyze(&input)
    }
}

hundo_sdk::export!(HollowKnight);

pub fn analyze(input: &Input) -> Result<Report, DriverError> {
    let file = input
        .files
        .first()
        .ok_or_else(|| DriverError::MissingFile("userN.dat".into()))?;
    let save = save::decode(&file.bytes)?;
    let pd = save
        .get("playerData")
        .and_then(Value::as_object)
        .ok_or_else(|| DriverError::Corrupt("playerData absent".into()))?;
    // The map Hundo drew from the installed game, when there is one.
    let world: Option<World> = input.file("@maps/hallownest.json").ok().and_then(|f| serde_json::from_slice(&f.bytes).ok());
    // The game's own words, in the reader's language, English filling the gaps.
    let lang = if input.is_french() { "FR" } else { "EN" };
    let mut text = HashMap::new();
    for l in ["EN", lang] {
        for sheet in SHEETS {
            if let Ok(f) = input.file(&format!("@texts/{l}_{sheet}")) {
                text.extend(save::sheet(&f.bytes));
            }
        }
    }
    let scenes = save.get("sceneData").and_then(Value::as_object);
    let full_journal = input.flag("journal-notes", true);
    Ok(Analysis { pd, scenes, fr: input.is_french(), text: &text, full_journal }.report(world.as_ref()))
}

/// The rooms and pins of the game's own map, in its pixels.
#[derive(Deserialize)]
struct World {
    width: f32,
    height: f32,
    rooms: Vec<Room>,
    #[serde(default)]
    pins: Vec<Pin>,
}

#[derive(Deserialize)]
struct Room {
    area: String,
    scene: String,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Deserialize)]
struct Pin {
    sprite: String,
    scene: String,
    #[serde(default)]
    room: String,
    area: String,
    x: f32,
    y: f32,
}

/// The game's map pins that are places rather than things to collect: what
/// each is called and the layer it goes in.
fn point_of_interest(sprite: &str, fr: bool) -> Option<(&'static str, &'static str)> {
    let t = |f: &'static str, e: &'static str| if fr { f } else { e };
    Some(match sprite {
        "pin_bench" => (t("Banc", "Bench"), t("Bancs", "Benches")),
        "pin_stag_station" => (t("Station de cerf", "Stag station"), t("Stations de cerf", "Stag stations")),
        "pin_tram_location" => (t("Tramway", "Tram"), t("Tramways", "Trams")),
        "pin_spa" => (t("Source chaude", "Hot spring"), t("Sources chaudes", "Hot springs")),
        "pin_blue_health" => (t("Cocon de vie", "Lifeblood cocoon"), t("Cocons de vie", "Lifeblood cocoons")),
        "pin_shop_sly" => ("Sly", t("Marchands", "Merchants")),
        "pin_shop_mapper" => ("Iselda", t("Marchands", "Merchants")),
        "pin_shop_leg_eater" => (t("Mange-Jambes", "Leg Eater"), t("Marchands", "Merchants")),
        "pin_shop_jiji" => ("Confessor Jiji", t("Marchands", "Merchants")),
        "pin_shop_relic_dealer" => ("Lemm", t("Marchands", "Merchants")),
        "pin_shop_banker" => ("Millibelle", t("Marchands", "Merchants")),
        "pin_shop_nailsmith" => (t("Forgeron d'aiguillon", "Nailsmith"), t("Marchands", "Merchants")),
        "pin_shop_hunter" => (t("Le Chasseur", "The Hunter"), t("Personnages", "Characters")),
        "pin_shop_dream_moth" => (t("La Voyante", "The Seer"), t("Personnages", "Characters")),
        "pin_charm_slug" => ("Salubra", t("Marchands", "Merchants")),
        "pin_grub_king" => (t("Père des larves", "Grubfather"), t("Personnages", "Characters")),
        "pin_god_seeker" => (t("Chercheuse de dieux", "Godseeker"), t("Personnages", "Characters")),
        "pin_colosseum" => (t("Colisée des fous", "Colosseum of Fools"), t("Lieux", "Places")),
        "Pin_Black_Egg" => (t("Temple de l'Œuf noir", "Temple of the Black Egg"), t("Lieux", "Places")),
        _ => return None,
    })
}

/// The language sheets read: creatures, charms, area names.
const SHEETS: [&str; 3] = ["Journal", "UI", "Map Zones"];

struct Analysis<'a> {
    pd: &'a Map<String, Value>,
    /// What the save remembers room by room: objects taken, rocks broken.
    scenes: Option<&'a Map<String, Value>>,
    fr: bool,
    /// The game's own text by key (`NAME_CRAWLER`), when the host could read it.
    text: &'a HashMap<String, String>,
    /// The reader's option: an entry of the journal is done only with the
    /// Hunter's notes, rather than once the creature was met.
    full_journal: bool,
}

impl Analysis<'_> {
    fn t<'s>(&self, fr: &'s str, en: &'s str) -> &'s str {
        if self.fr { fr } else { en }
    }

    /// A line of the game's text, on one line.
    fn text(&self, key: &str) -> Option<String> {
        let t = self.text.get(key)?.replace("<br>", " ");
        let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
        (!t.is_empty()).then_some(t)
    }

    /// An area's name as the game words it in the reader's language.
    fn zone(&self, english: &str) -> String {
        ZONE_KEYS.iter().find(|(e, _)| *e == english).and_then(|(_, k)| self.text(k)).unwrap_or_else(|| english.to_string())
    }

    /// A list of `sceneData`, such as `persistentBoolItems`.
    fn save_scene_data(&self, key: &str) -> Option<&Vec<Value>> {
        self.scenes?.get(key)?.as_array()
    }

    fn bool(&self, key: &str) -> Option<bool> {
        self.pd.get(key)?.as_bool()
    }

    fn int(&self, key: &str) -> i64 {
        self.pd.get(key).and_then(Value::as_i64).unwrap_or(0)
    }

    fn num(&self, key: &str) -> f64 {
        self.pd.get(key).and_then(Value::as_f64).unwrap_or(0.0)
    }

    /// A checklist of boolean fields. A field missing from the save (an older
    /// game version) drops its row rather than claiming it is not done.
    fn flags(&self, list: &[(&str, &str)]) -> Vec<Item> {
        list.iter()
            .filter_map(|(key, name)| Some(Item::check(*key, *name, self.bool(key)?).link(wiki(name))))
            .collect()
    }

    fn report(&self, world: Option<&World>) -> Report {
        let mut report = Report::empty();
        report.game_version = self.pd.get("version").and_then(Value::as_str).map(str::to_string);
        report.completion = Some(Completion {
            label: self.t("Complétion du jeu", "Game completion").into(),
            value: self.num("completionPercentage"),
            max: 112.0,
        });
        report.stats = vec![
            Stat::new("playTime", self.t("Temps de jeu", "Play time"), hundo_sdk::format_duration(self.num("playTime"))),
            Stat::new("geo", "Geo", self.int("geo").to_string()),
            Stat::new("essence", self.t("Essence", "Essence"), self.int("dreamOrbs").to_string()),
            Stat::new("stations", self.t("Stations de cerf", "Stag stations"), self.int("stationsOpened").to_string()),
        ];
        match report.game_version.as_deref() {
            Some(v) if v.starts_with("1.5") => {}
            Some(v) => report.warnings.push(format!(
                "{} {v}",
                self.t("Driver vérifié sur la 1.5 ; sauvegarde écrite par la", "Driver checked on 1.5; save written by")
            )),
            None => {}
        }

        let pct = self.t("Complétion 112 %", "112 % completion");
        let more = self.t("Au-delà du 112 %", "Beyond 112 %");
        report.sections = vec![
            Section::new("charms", self.t("Charmes", "Charms"), self.charms(1..=36)).group(pct).gallery(),
            Section::new("equipment", self.t("Équipement", "Equipment"), self.equipment()).group(pct),
            Section::new("spells", self.t("Sorts", "Spells"), self.spells()).group(pct),
            Section::new("nail", self.t("Aiguillon et arts", "Nail and nail arts"), self.nail()).group(pct),
            Section::new("health", self.t("Masques et réceptacles", "Masks and vessels"), self.health()).group(pct),
            Section::new("dreamNail", self.t("Aiguillon des rêves", "Dream Nail"), self.dream_nail()).group(pct),
            Section::new("dreamers", self.t("Rêveurs", "Dreamers"), self.dreamers()).group(pct),
            Section::new("bosses", self.t("Boss", "Bosses"), self.bosses()).group(pct),
            Section::new("warriors", self.t("Rêves de guerriers", "Warrior dreams"), self.warrior_dreams()).group(pct),
            Section::new("colosseum", self.t("Colisée des fous", "Colosseum of Fools"), self.colosseum()).group(pct),
            Section::new("grimm", self.t("Troupe Grimm", "Grimm Troupe"), self.grimm()).group(pct).dlc("grimm-troupe"),
            Section::new("lifeblood", "Lifeblood", self.flags(&[("killedHiveKnight", "Hive Knight")])).group(pct).dlc("lifeblood"),
            Section::new("godmaster", "Godmaster", self.godmaster()).group(pct).dlc("godmaster"),
            Section::new("endings", self.t("Fins et panthéon final", "Endings and final pantheon"), self.endings()).group(more),
            Section::new("optional", self.t("Boss optionnels", "Optional bosses"), self.optional_bosses()).group(more),
            Section::new("collection", self.t("Objets", "Items"), self.collection(world.is_none())).group(more),
        ];
        let journal = self.journal();
        if !journal.is_empty() {
            report.sections.push(
                Section::new("journal", self.t("Journal du Chasseur", "Hunter's Journal"), journal).group(self.t("Bestiaire", "Bestiary")).gallery(),
            );
        }
        // With the game's map, each grub is its own item, at its pin, and
        // everything lying in the rooms is listed where it lies.
        if let Some(w) = world {
            let area_of_scene = |scene: &str| {
                // A room the map does not draw is in its door's area.
                let scene = pickups::DOORS.iter().find(|d| d.0 == scene).map_or(scene, |d| d.1);
                w.rooms.iter().find(|r| r.scene == scene).map(|r| game_area(&r.area).unwrap_or(&r.area).to_string())
            };
            let mut shards = self.pickups(pickups::Kind::Mask, &area_of_scene);
            shards.extend(self.pickups(pickups::Kind::Vessel, &area_of_scene));
            report.sections.push(Section::new("shards", self.t("Fragments au sol", "Shards on the ground"), shards).group(more));
            let finds = self.t("Trouvailles", "Finds");
            // Geo: needed for nothing, shown on demand.
            report.sections.push(Section::new("chests", self.t("Coffres de géo", "Geo chests"), self.pickups(pickups::Kind::Chest, &area_of_scene)).group(finds).optional());
            report.sections.push(Section::new("rocks", self.t("Rochers de géo", "Geo rocks"), self.pickups(pickups::Kind::Rock, &area_of_scene)).group(finds).optional());
            report.sections.push(Section::new("shinies", self.t("Objets au sol", "Items on the ground"), self.pickups(pickups::Kind::Shiny, &area_of_scene)).group(finds));
            let grubs = self.grubs(w);
            if !grubs.is_empty() {
                report.sections.push(Section::new("grubs", self.t("Larves", "Grubs"), grubs).group(more));
            }
            let roots = self.roots(w);
            if !roots.is_empty() {
                report.sections.push(Section::new("roots", self.t("Racines murmurantes", "Whispering roots"), roots).group(more));
            }
        }
        // What the merchants sell that is not listed elsewhere.
        let listed: Vec<String> = report.sections.iter().flat_map(|s| &s.items).map(|i| i.id.clone()).collect();
        let wares: Vec<Item> = self.wares().into_iter().filter(|i| !listed.contains(&i.id)).collect();
        if !wares.is_empty() {
            report.sections.push(Section::new("shops", self.t("Boutiques", "Shops"), wares).group(more));
        }
        for item in report.sections.iter_mut().flat_map(|s| s.items.iter_mut()) {
            if let Some(area) = area_of(&item.id) {
                item.location = Some(area.into());
            }
            // Bought: from whom, for how much, and where the merchant is.
            if let Some(w) = shops::ware(&item.id) {
                item.hint = Some(self.price(w));
                item.location = Some(
                    match w.pin {
                        "pin_charm_slug" => "Forgotten Crossroads",
                        "pin_shop_leg_eater" => "Fungal Wastes",
                        _ => "Dirtmouth",
                    }
                    .into(),
                );
            }
            // A boss: its Journal portrait and the game's own name for it.
            if let Some((text, image)) = journal::BOSSES.iter().find(|b| b.0 == item.id).and_then(|b| journal::entry(b.1)) {
                if item.image.is_none() {
                    item.image = Some(format!("game:{image}"));
                }
                if let Some(name) = self.text(&format!("NAME_{text}")) {
                    item.title = name;
                }
            }
            if item.id.starts_with("pantheon") && item.image.is_none() {
                item.image = Some("game:pin_god_seeker".into());
            }
            if item.image.is_none()
                && let Some((_, image)) = ITEM_IMAGES.iter().find(|(id, _)| *id == item.id)
            {
                item.image = Some(format!("game:{image}"));
            }
        }
        report.maps.push(match world {
            Some(w) => game_map(w, &report.sections, self.fr),
            None => hallownest(&report.sections),
        });
        // Areas by the game's own names, now that the map has placed things.
        for item in report.sections.iter_mut().flat_map(|s| s.items.iter_mut()) {
            if item.areas.is_empty()
                && let Some(l) = &item.location
            {
                item.areas.push(l.clone());
            }
            item.areas = item.areas.iter().map(|a| self.zone(a)).collect();
            item.location = item.location.as_deref().map(|l| self.zone(l));
        }
        for region in report.maps.iter_mut().flat_map(|m| m.regions.iter_mut()) {
            region.label = self.zone(&region.label);
        }
        report
    }

    /// One item per whispering root pin: done once all its essence is taken,
    /// in progress once found.
    fn roots(&self, w: &World) -> Vec<Item> {
        let list = |key: &str| -> Vec<&str> { self.pd.get(key).and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).collect() };
        let (done, found) = (list("scenesEncounteredDreamPlantC"), list("scenesEncounteredDreamPlant"));
        w.pins
            .iter()
            .filter(|p| p.sprite.starts_with("pin_dream_tree") && !p.room.is_empty())
            .map(|p| {
                let area = game_area(&p.area).unwrap_or(&p.area);
                let status = if done.contains(&p.room.as_str()) {
                    Status::Done
                } else if found.contains(&p.room.as_str()) {
                    Status::InProgress
                } else {
                    Status::Todo
                };
                let item = Item::new(format!("root-{}", p.room), format!("{} · {}", self.t("Racine", "Root"), self.zone(area)), status).location(area).link(wiki("Whispering_Root")).image("game:pin_dream_tree");
                if status == Status::InProgress { item.hint(self.t("Trouvée, essence pas toute récoltée", "Found, essence not all taken")) } else { item }
            })
            .collect()
    }

    /// One item per grub pin; a room with two grubs counts its rescues.
    fn grubs(&self, w: &World) -> Vec<Item> {
        let rescued: Vec<&str> = self.pd.get("scenesGrubRescued").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).collect();
        let mut seen: Vec<&str> = Vec::new();
        w.pins
            .iter()
            .filter(|p| p.sprite == "pin_grub_location")
            .map(|p| {
                let nth = seen.iter().filter(|s| **s == p.scene).count();
                seen.push(&p.scene);
                let done = rescued.iter().filter(|s| **s == p.scene).count() > nth;
                let area = game_area(&p.area).unwrap_or(&p.area);
                Item::check(format!("grub-{}-{nth}", p.scene), format!("{} · {}", self.t("Larve", "Grub"), self.zone(area)), done)
                    .location(area)
                    .link(wiki("Grub"))
                    .image("game:pin_grub_location")
            })
            .collect()
    }

    fn charms(&self, range: std::ops::RangeInclusive<usize>) -> Vec<Item> {
        range
            .filter_map(|n| {
                let key = format!("gotCharm_{n}");
                let name = CHARMS[n - 1];
                // Kingsoul and Void Heart share a number: the driver's name says which.
                let title = if n == 36 { None } else { self.text(&format!("CHARM_NAME_{n}")) };
                let mut item = Item::check(key.clone(), title.unwrap_or_else(|| name.into()), self.bool(&key)?)
                    .link(wiki(name))
                    .image(format!("game:{}", CHARM_SPRITES[n]));
                if let Some(d) = self.text(&format!("CHARM_DESC_{n}")) {
                    item = item.details(d);
                }
                Some(item)
            })
            .collect()
    }

    fn equipment(&self) -> Vec<Item> {
        self.flags(&[
            ("hasDash", "Mothwing Cloak"),
            ("hasWalljump", "Mantis Claw"),
            ("hasSuperDash", "Crystal Heart"),
            ("hasDoubleJump", "Monarch Wings"),
            ("hasAcidArmour", "Isma's Tear"),
            ("hasKingsBrand", "King's Brand"),
            ("hasShadowDash", "Shade Cloak"),
        ])
    }

    fn spells(&self) -> Vec<Item> {
        let spell = |key: &str, level: i64, name: &str| {
            Item::check(format!("{key}{level}"), name, self.int(key) >= level).link(wiki(name))
        };
        vec![
            spell("fireballLevel", 1, "Vengeful Spirit"),
            spell("fireballLevel", 2, "Shade Soul"),
            spell("quakeLevel", 1, "Desolate Dive"),
            spell("quakeLevel", 2, "Descending Dark"),
            spell("screamLevel", 1, "Howling Wraiths"),
            spell("screamLevel", 2, "Abyss Shriek"),
        ]
    }

    fn nail(&self) -> Vec<Item> {
        let mut items = vec![Item::count(
            "nailSmithUpgrades",
            self.t("Améliorations du forgeron", "Nailsmith upgrades"),
            self.int("nailSmithUpgrades") as u32,
            4,
        )
        .hint(self.t("Forgeron d'aiguillon, Cité des larmes", "Nailsmith, City of Tears"))
        .link(wiki("Nailsmith"))];
        // The game's names for these two are swapped: hasDashSlash is Great Slash.
        items.extend(self.flags(&[
            ("hasCyclone", "Cyclone Slash"),
            ("hasDashSlash", "Great Slash"),
            ("hasUpwardSlash", "Dash Slash"),
        ]));
        items
    }

    fn health(&self) -> Vec<Item> {
        let shards = (self.int("maxHealthBase") - 5).max(0) * 4 + self.int("heartPieces");
        let fragments = self.int("MPReserveMax") / 33 * 3 + self.int("vesselFragments");
        vec![
            Item::count("maskShards", self.t("Fragments de masque", "Mask shards"), shards as u32, 16).link(wiki("Mask_Shard")),
            Item::count("vesselFragments", self.t("Fragments de réceptacle", "Vessel fragments"), fragments as u32, 9)
                .link(wiki("Vessel_Fragment")),
        ]
    }

    fn dream_nail(&self) -> Vec<Item> {
        let mut items = self.flags(&[("hasDreamNail", "Dream Nail"), ("dreamNailUpgraded", "Awoken Dream Nail")]);
        if let Some(done) = self.bool("mothDeparted") {
            let essence = self.int("dreamOrbs");
            let mut seer = Item::check("mothDeparted", self.t("Dernières paroles de la Voyante", "The Seer's final words"), done)
                .link(wiki("Seer"));
            if !done {
                seer = if essence >= SEER_ESSENCE {
                    seer.status_hint(
                        Status::InProgress,
                        format!("{} ({essence} / {SEER_ESSENCE})", self.t("Assez d'essence : retourne parler à la Voyante", "Enough Essence: go back and talk to the Seer")),
                    )
                } else {
                    seer.hint(format!("{} {essence} / {SEER_ESSENCE}", self.t("Essence :", "Essence:")))
                };
            }
            items.push(seer);
        }
        items
    }

    fn dreamers(&self) -> Vec<Item> {
        self.flags(&[
            ("lurienDefeated", "Lurien the Watcher"),
            ("monomonDefeated", "Monomon the Teacher"),
            ("hegemolDefeated", "Herrah the Beast"),
        ])
        .into_iter()
        .map(|i| i.spoiler(Spoiler::Minor))
        .collect()
    }

    /// The bosses the game counts toward completion.
    fn bosses(&self) -> Vec<Item> {
        self.flags(&[
            ("killedBigFly", "Gruz Mother"),
            ("falseKnightDefeated", "False Knight"),
            ("hornet1Defeated", "Hornet Protector"),
            ("killedMawlek", "Brooding Mawlek"),
            ("defeatedDungDefender", "Dung Defender"),
            ("mageLordDefeated", "Soul Master"),
            ("defeatedMantisLords", "Mantis Lords"),
            ("killedMimicSpider", "Nosk"),
            ("killedInfectedKnight", "Broken Vessel"),
            ("collectorDefeated", "The Collector"),
            ("defeatedMegaJelly", "Uumuu"),
            ("hornetOutskirtsDefeated", "Hornet Sentinel"),
            ("killedTraitorLord", "Traitor Lord"),
            ("killedBlackKnight", "Watcher Knight"),
        ])
    }

    fn warrior_dreams(&self) -> Vec<Item> {
        // 0 = never met, 1 = met, 2 = defeated.
        [
            ("xeroDefeated", "Xero"),
            ("aladarSlugDefeated", "Gorb"),
            ("elderHuDefeated", "Elder Hu"),
            ("mumCaterpillarDefeated", "Marmu"),
            ("noEyesDefeated", "No Eyes"),
            ("galienDefeated", "Galien"),
            ("markothDefeated", "Markoth"),
        ]
        .iter()
        .filter(|(key, _)| self.pd.contains_key(*key))
        .map(|(key, name)| {
            let item = Item::new(*key, *name, Status::Todo).link(wiki(name));
            match self.int(key) {
                2.. => item.status_hint(Status::Done, ""),
                1 => item.status_hint(Status::InProgress, self.t("Rencontré, pas encore vaincu", "Met, not defeated yet")),
                _ => item,
            }
        })
        .collect()
    }

    fn colosseum(&self) -> Vec<Item> {
        self.flags(&[
            ("colosseumBronzeCompleted", "Trial of the Warrior"),
            ("colosseumSilverCompleted", "Trial of the Conqueror"),
            ("colosseumGoldCompleted", "Trial of the Fool"),
        ])
    }

    fn grimm(&self) -> Vec<Item> {
        let mut items = self.charms(37..=40);
        items.extend(self.flags(&[("killedGrimm", "Troupe Master Grimm")]));
        // Either ending of the troupe's story counts: the ritual or the banishment.
        let level = self.int("grimmChildLevel");
        let banished = self.bool("destroyedNightmareLantern").unwrap_or(false);
        items.push(
            Item::check("grimmRitual", self.t("Accomplir le rituel ou bannir la troupe", "Complete the ritual or banish the troupe"), level >= 4 || banished)
                .link(wiki("Grimm_Troupe")),
        );
        items
    }

    fn pantheon(&self, tier: usize, name: &str) -> Option<Item> {
        let door = self.pd.get(&format!("bossDoorStateTier{tier}"))?;
        let done = door.get("completed").and_then(Value::as_bool).unwrap_or(false);
        let unlocked = door.get("unlocked").and_then(Value::as_bool).unwrap_or(false);
        let status = match (done, unlocked) {
            (true, _) => Status::Done,
            (false, true) => Status::Todo,
            _ => Status::Locked,
        };
        Some(Item::new(format!("pantheon{tier}"), name, status).link(wiki(name)).spoiler(Spoiler::Minor))
    }

    fn godmaster(&self) -> Vec<Item> {
        let mut items = self.flags(&[("hasGodfinder", "Godtuner")]);
        let names = ["Pantheon of the Master", "Pantheon of the Artist", "Pantheon of the Sage", "Pantheon of the Knight"];
        items.extend(names.iter().enumerate().filter_map(|(i, n)| self.pantheon(i + 1, n)));
        items
    }

    fn endings(&self) -> Vec<Item> {
        let mut items: Vec<Item> = self
            .flags(&[("killedHollowKnight", "The Hollow Knight"), ("killedFinalBoss", "The Radiance")])
            .into_iter()
            .map(|i| i.spoiler(Spoiler::Major))
            .collect();
        items.extend(self.pantheon(5, "Pantheon of Hallownest").map(|i| i.dlc("godmaster")));
        items
    }

    fn optional_bosses(&self) -> Vec<Item> {
        let mut items: Vec<Item> = self
            .flags(&[("greyPrinceDefeated", "Grey Prince Zote"), ("killedWhiteDefender", "White Defender")])
            .into_iter()
            .map(|i| i.dlc("hidden-dreams"))
            .collect();
        if self.bool("destroyedNightmareLantern").unwrap_or(false) {
            items.push(
                Item::new("nightmareKing", "Nightmare King Grimm", Status::Missed)
                    .hint(self.t("Troupe bannie : plus possible sur cette partie", "Troupe banished: no longer possible on this save"))
                    .dlc("grimm-troupe"),
            );
        } else {
            items.extend(self.flags(&[("killedNightmareGrimm", "Nightmare King Grimm")]).into_iter().map(|i| i.dlc("grimm-troupe")));
        }
        items
    }

    fn collection(&self, grub_counter: bool) -> Vec<Item> {
        let mut items = Vec::new();
        if grub_counter {
            items.push(Item::count("grubs", self.t("Larves libérées", "Grubs rescued"), self.int("grubsCollected") as u32, 46).link(wiki("Grub")));
        }
        items.extend([
            Item::count("charmNotches", self.t("Encoches de charme", "Charm notches"), self.int("charmSlots") as u32, 11)
                .link(wiki("Charm_Notch")),
        ]);
        items.extend(self.flags(&[("hasLantern", "Lumafly Lantern")]));
        items
    }
}

trait StatusHint {
    fn status_hint(self, status: Status, hint: impl Into<String>) -> Self;
}

impl StatusHint for Item {
    fn status_hint(mut self, status: Status, hint: impl Into<String>) -> Self {
        self.status = status;
        let hint = hint.into();
        self.hint = (!hint.is_empty()).then_some(hint);
        self
    }
}

/// Hallownest's areas as polygons on a 2000 × 1200 frame. The shapes are
/// drawn for Hundo, placed roughly as on the in-game map, not traced from it.
const AREAS: [(&str, &[(f32, f32)]); 17] = [
    ("Howling Cliffs", &[(40.0, 40.0), (420.0, 40.0), (400.0, 220.0), (60.0, 260.0)]),
    ("Dirtmouth", &[(430.0, 40.0), (900.0, 40.0), (880.0, 170.0), (440.0, 180.0)]),
    ("Crystal Peak", &[(910.0, 40.0), (1480.0, 40.0), (1460.0, 230.0), (1180.0, 260.0), (920.0, 200.0)]),
    ("Resting Grounds", &[(1490.0, 60.0), (1900.0, 80.0), (1880.0, 360.0), (1560.0, 380.0), (1470.0, 240.0)]),
    ("Forgotten Crossroads", &[(440.0, 190.0), (880.0, 175.0), (1170.0, 270.0), (1150.0, 420.0), (780.0, 450.0), (460.0, 400.0)]),
    ("Greenpath", &[(60.0, 270.0), (400.0, 230.0), (450.0, 410.0), (420.0, 560.0), (80.0, 580.0)]),
    ("Fog Canyon", &[(430.0, 420.0), (700.0, 440.0), (700.0, 620.0), (440.0, 640.0)]),
    ("Fungal Wastes", &[(710.0, 450.0), (1000.0, 430.0), (1010.0, 760.0), (720.0, 800.0), (700.0, 630.0)]),
    (
        "City of Tears",
        &[(1010.0, 430.0), (1160.0, 425.0), (1170.0, 280.0), (1460.0, 250.0), (1560.0, 390.0), (1540.0, 650.0), (1020.0, 660.0)],
    ),
    ("Kingdom's Edge", &[(1570.0, 390.0), (1900.0, 370.0), (1960.0, 820.0), (1720.0, 760.0), (1560.0, 660.0)]),
    ("Queen's Gardens", &[(60.0, 600.0), (430.0, 580.0), (450.0, 860.0), (80.0, 900.0)]),
    (
        "Royal Waterways",
        &[(1020.0, 670.0), (1550.0, 665.0), (1700.0, 770.0), (1560.0, 880.0), (1200.0, 860.0), (1030.0, 780.0)],
    ),
    ("Godhome", &[(1580.0, 890.0), (1720.0, 780.0), (1960.0, 840.0), (1940.0, 1000.0), (1620.0, 1000.0)]),
    ("Deepnest", &[(90.0, 920.0), (450.0, 870.0), (720.0, 820.0), (760.0, 1040.0), (500.0, 1160.0), (120.0, 1140.0)]),
    (
        "Ancient Basin",
        &[(770.0, 820.0), (1020.0, 790.0), (1190.0, 870.0), (1480.0, 900.0), (1420.0, 1040.0), (1180.0, 1040.0), (960.0, 1030.0), (780.0, 1030.0)],
    ),
    ("The Abyss", &[(960.0, 1050.0), (1400.0, 1060.0), (1380.0, 1170.0), (980.0, 1170.0)]),
    ("White Palace", &[(1460.0, 1060.0), (1640.0, 1040.0), (1660.0, 1170.0), (1470.0, 1170.0)]),
];

fn hallownest(sections: &[Section]) -> GameMap {
    let mut map = GameMap::new("hallownest", "Hallownest", 2000.0, 1200.0);
    for (name, points) in AREAS {
        map = map.region(name, name, points);
    }
    map.place_in_regions(sections, |item| item.location.clone(), |section, _| section.title.clone())
}

/// The game's names for its map areas, and ours.
const GAME_AREAS: [(&str, &str); 14] = [
    ("Town_Tutorial", "Dirtmouth"),
    ("Cliffs", "Howling Cliffs"),
    ("Crossroads", "Forgotten Crossroads"),
    ("Green_Path", "Greenpath"),
    ("Fog_Canyon", "Fog Canyon"),
    ("Fungal Wastes", "Fungal Wastes"),
    ("City of Tears", "City of Tears"),
    ("Kingdoms_Edge", "Kingdom's Edge"),
    ("Queens_Gardens", "Queen's Gardens"),
    ("Waterways", "Royal Waterways"),
    ("Deepnest", "Deepnest"),
    ("Ancient Basin", "Ancient Basin"),
    ("Resting_Grounds", "Resting Grounds"),
    ("Crystal Peak", "Crystal Peak"),
];

/// The inventory's own picture of each item that has one, by id.
const ITEM_IMAGES: &[(&str, &str)] = &[
    ("hasDash", "Inv_0015_dash-cloak"),
    ("hasWalljump", "items__0003_mantis-claw"),
    ("hasSuperDash", "items__0000_crystal-heart"),
    ("hasDoubleJump", "items__0002_emperor-wings"),
    ("hasAcidArmour", "items__0001_acid-armour"),
    ("hasKingsBrand", "inv_white_brand"),
    ("hasShadowDash", "items__0004_shade_cloak"),
    ("fireballLevel1", "Inv_0025_spell_fireball_01"),
    ("fireballLevel2", "Inv_0025_spell_fireball_01_level2"),
    ("quakeLevel1", "Inv_0026_spell_quake_01"),
    ("quakeLevel2", "Inv_0026_spell_quake_01_level2"),
    ("screamLevel1", "Inv_0024_spell_scream_01"),
    ("screamLevel2", "Inv_0024_spell_scream_01_level2"),
    ("nailSmithUpgrades", "nail_upgrade_0000_pure-nail"),
    ("hasCyclone", "Inv_0023_inv_whirlwind_skill"),
    ("hasDashSlash", "Inv_0022_inv_uppercut_skill"),
    ("hasUpwardSlash", "Inv_0021_inv_dash_strike_skill"),
    ("maskShards", "select_game_HUD_0001_health"),
    ("vesselFragments", "Inventory_soul_vessel_full"),
    ("hasDreamNail", "dream_nail_0003_1"),
    ("dreamNailUpgraded", "dream_nail_0000_4"),
    ("mothDeparted", "pin_shop_dream_moth"),
    ("lurienDefeated", "bestiary_dreamer_01_f"),
    ("monomonDefeated", "bestiary_dreamer_02_f"),
    ("hegemolDefeated", "bestiary_dreamer_03_f"),
    ("colosseumBronzeCompleted", "pin_colosseum"),
    ("colosseumSilverCompleted", "pin_colosseum"),
    ("colosseumGoldCompleted", "pin_colosseum"),
    ("grimmRitual", "Grimmkin_lantern"),
    ("hasGodfinder", "pin_god_seeker"),
    ("charmNotches", "Charm_Notch"),
    ("hasLantern", "Lumafly_Lantern0000"),
    ("slySimpleKey", "inv_Waterways_Key"),
    ("salubraBlessing", "pin_charm_slug"),
];

/// The language sheet's key for each area, by the English name items use.
const ZONE_KEYS: [(&str, &str); 17] = [
    ("Dirtmouth", "TOWN"),
    ("Howling Cliffs", "CLIFFS"),
    ("Forgotten Crossroads", "CROSSROADS"),
    ("Greenpath", "GREEN_PATH"),
    ("Fog Canyon", "FOG_CANYON"),
    ("Fungal Wastes", "WASTES"),
    ("City of Tears", "CITY"),
    ("Kingdom's Edge", "OUTSKIRTS"),
    ("Queen's Gardens", "ROYAL_GARDENS"),
    ("Royal Waterways", "WATERWAYS"),
    ("Deepnest", "DEEPNEST"),
    ("Ancient Basin", "ABYSS"),
    ("The Abyss", "ABYSS_DEEP"),
    ("Resting Grounds", "RESTING_GROUNDS"),
    ("Crystal Peak", "MINES"),
    ("White Palace", "WHITE_PALACE"),
    ("Godhome", "GODS_GLORY"),
];

/// The inventory's charm sprites, by charm number (the game's own list; 0 is unused).
const CHARM_SPRITES: [&str; 41] = [
    "charm_sprite_01", "charm_sprite_02", "charm_sprite_03", "charm_grub_mid", "_0006_charm_stalwart_shell",
    "charm_blocker", "_0005_charm_fury", "_0005_charm_fast_focus", "_0010_charm_bluehealth", "_0007_charm_blue_health_large",
    "charm_dung_def", "charm_fluke", "_0000_charm_thorn_counter", "char_mantis", "_0006_charm_no_recoil",
    "_0008_charm_nail_damage_up", "charm_shade_impact", "charm_fungus", "_0007_charm_greed", "_0002_charm_spell_damage_up",
    "_0001_charm_more_soul", "charm_soul_up_large", "_0009_charm_Hatchling", "_0002_charm_glass_heal", "_0003_charm_glass_geo",
    "_0002_charm_glass_attack_up", "_0004_charm_charge_time_up", "charm_blue_health_convert", "charm_slug", "charm_hive",
    "inv_dream_charm", "_0011_charm_generic_03", "_0003_charm_nail_slash_speed_up", "charm_magic_cost_down", "charm_crystal",
    "charm_grub_blade", "charm_sprite_36", "charm_grimm_sprint_master", "charm_grimm_markoth_shield", "charm_grimm_silkweaver",
    "charm_grimmkin_01",
];

fn game_area(game: &str) -> Option<&'static str> {
    GAME_AREAS.iter().find(|(g, _)| *g == game).map(|(_, a)| *a)
}

/// The room an item is in, where it is known for sure.
fn scene_of(id: &str) -> Option<&'static str> {
    Some(match id {
        "falseKnightDefeated" => "Crossroads_10",
        "killedBigFly" => "Crossroads_04",
        "killedMawlek" => "Crossroads_09",
        "hornet1Defeated" => "Fungus1_04",
        "defeatedMantisLords" => "Fungus2_15",
        "mageLordDefeated" => "Ruins1_24",
        "defeatedDungDefender" => "Waterways_05",
        "killedInfectedKnight" => "Abyss_19",
        "killedMimicSpider" => "Deepnest_32",
        "collectorDefeated" => "Ruins2_11",
        "defeatedMegaJelly" => "Fungus3_archive_02",
        "killedBlackKnight" => "Ruins2_03",
        "killedTraitorLord" => "Fungus3_23",
        "hornetOutskirtsDefeated" => "Deepnest_East_Hornet",
        "killedHiveKnight" => "Hive_05",
        "killedWhiteDefender" => "Waterways_15",
        "lurienDefeated" => "Ruins2_Watcher_Room",
        "monomonDefeated" => "Fungus3_archive_02",
        "hegemolDefeated" => "Deepnest_Spider_Town",
        "xeroDefeated" => "RestingGrounds_02",
        "aladarSlugDefeated" => "Cliffs_02",
        "elderHuDefeated" => "Fungus2_32",
        "mumCaterpillarDefeated" => "Fungus3_40",
        "noEyesDefeated" => "Fungus1_35",
        "galienDefeated" => "Deepnest_40",
        "markothDefeated" => "Deepnest_East_10",
        "hasSuperDash" => "Mines_31",
        "hasAcidArmour" => "Waterways_13",
        "hasShadowDash" => "Abyss_10",
        "quakeLevel1" => "Ruins1_24",
        "quakeLevel2" => "Mines_35",
        "screamLevel2" => "Abyss_12",
        "fireballLevel1" => "Crossroads_06",
        "screamLevel1" => "Fungus3_44",
        "hasDreamNail" => "RestingGrounds_04",
        "nailSmithUpgrades" => "Ruins1_04",
        "colosseumBronzeCompleted" | "colosseumSilverCompleted" | "colosseumGoldCompleted" => "Deepnest_East_09",
        _ => return None,
    })
}

/// The map from the game's files: its areas named, the rooms that hold
/// something coloured, and every item in its room, or in its area when only
/// the area is known.
fn game_map(w: &World, sections: &[Section], fr: bool) -> GameMap {
    let mut map = GameMap::new("hallownest", "Hallownest", w.width, w.height);
    // Area names, each over the box of its rooms: regions without items draw as labels.
    for (game, name) in GAME_AREAS {
        let rooms: Vec<&Room> = w.rooms.iter().filter(|r| r.area == game).collect();
        if rooms.is_empty() {
            continue;
        }
        let x0 = rooms.iter().map(|r| r.x).fold(f32::MAX, f32::min);
        let y0 = rooms.iter().map(|r| r.y).fold(f32::MAX, f32::min);
        let x1 = rooms.iter().map(|r| r.x + r.w).fold(f32::MIN, f32::max);
        let y1 = rooms.iter().map(|r| r.y + r.h).fold(f32::MIN, f32::max);
        map = map.region(&format!("area:{game}"), name, &[(x0, y0), (x1, y0), (x1, y1), (x0, y1)]);
    }
    let room_of = |scene: &str| w.rooms.iter().find(|r| r.scene == scene);
    // A scene's whole box: some rooms are drawn in several pieces.
    let scene_box = |scene: &str| -> Option<(f32, f32, f32, f32)> {
        // A variant of a room (`_Festival`, `_boss`) is drawn as the room itself.
        let mut scene = scene;
        let rooms: Vec<&Room> = loop {
            let rooms: Vec<&Room> = w.rooms.iter().filter(|r| r.scene == scene).collect();
            if !rooms.is_empty() {
                break rooms;
            }
            scene = scene.rsplit_once('_')?.0;
        };
        let x0 = rooms.iter().map(|r| r.x).fold(f32::MAX, f32::min);
        let y0 = rooms.iter().map(|r| r.y).fold(f32::MAX, f32::min);
        let x1 = rooms.iter().map(|r| r.x + r.w).fold(f32::MIN, f32::max);
        let y1 = rooms.iter().map(|r| r.y + r.h).fold(f32::MIN, f32::max);
        Some((x0, y0, x1 - x0, y1 - y0))
    };
    // A place in a scene, as fractions from its left and its bottom.
    let at = |scene: &str, fx: f32, fy: f32| scene_box(scene).map(|(x, y, bw, bh)| (x + fx * bw, y + (1.0 - fy) * bh));
    let minor: Vec<&str> = ["chests", "rocks", "shinies", "journal"].to_vec();
    let mut per_area: Vec<(&str, usize)> = Vec::new();
    let grub_pins: Vec<&Pin> = w.pins.iter().filter(|p| p.sprite == "pin_grub_location").collect();
    for section in sections {
        for item in &section.items {
            let category = section.title.clone();
            let marker = |x: f32, y: f32, approx: bool| {
                let mut m = MapMarker::at(format!("m-{}", item.id), item.title.clone(), x, y).item(item.id.clone()).category(category.clone());
                m.approximate = approx;
                m
            };
            // A grub: its own pin.
            if let Some(rest) = item.id.strip_prefix("grub-") {
                let (scene, nth) = rest.rsplit_once('-').unwrap_or((rest, "0"));
                let nth: usize = nth.parse().unwrap_or(0);
                if let Some(p) = grub_pins.iter().filter(|p| p.scene == scene).nth(nth) {
                    map = map.marker(marker(p.x, p.y, false));
                }
                continue;
            }
            // A whispering root: its pin.
            if let Some(room) = item.id.strip_prefix("root-") {
                if let Some(p) = w.pins.iter().find(|p| p.sprite.starts_with("pin_dream_tree") && p.room == room) {
                    map = map.marker(marker(p.x, p.y, false));
                }
                continue;
            }
            // Bought: at the merchant's pin.
            if let Some(p) = shops::ware(&item.id).and_then(|ware| w.pins.iter().find(|p| p.sprite == ware.pin)) {
                map = map.marker(marker(p.x, p.y, false));
                continue;
            }
            // Something lying in a room, or a charm on the ground: its very spot.
            let spot = pickups::PICKUPS
                .iter()
                .find(|p| pickups::item_id(p) == item.id)
                .map(|p| (p.scene, p.x, p.y))
                .or_else(|| pickups::item_spot(&item.id));
            if let Some((x, y)) = spot.and_then(|(scene, fx, fy)| at(scene, fx, fy)) {
                map = map.marker(marker(x, y, false));
                continue;
            }
            // A room the map does not draw: at the door that leads to it.
            if let Some((x, y, bw, bh)) = spot
                .and_then(|(scene, _, _)| pickups::DOORS.iter().find(|d| d.0 == scene))
                .and_then(|(_, door)| scene_box(door))
            {
                map = map.marker(marker(x + bw / 2.0, y + bh / 2.0, true));
                continue;
            }
            // A creature: the area it is most found in.
            if section.id == "journal" {
                let Some(area) = item.areas.first() else { continue };
                let Some((game, _)) = GAME_AREAS.iter().find(|(_, a)| a == area) else { continue };
                let rooms: Vec<&Room> = w.rooms.iter().filter(|r| r.area == *game).collect();
                // Spread over the area's rooms, the same room for the same creature.
                let n = item.id.bytes().fold(0usize, |h, b| h.wrapping_mul(31).wrapping_add(b as usize));
                if let Some(room) = rooms.get(n % rooms.len().max(1)) {
                    map = map.marker(marker(room.x + room.w / 2.0, room.y + room.h / 2.0, true));
                }
                continue;
            }
            // A known room: its centre.
            if let Some(room) = scene_of(&item.id).and_then(room_of) {
                map = map.marker(marker(room.x + room.w / 2.0, room.y + room.h / 2.0, false));
                continue;
            }
            // Godhome and the White Palace are not on the game's map: their
            // things go where one enters them.
            if let Some(room) = item.location.as_deref().and_then(|a| match a {
                "Godhome" => Some("GG_Waterways"),
                "White Palace" => Some("Abyss_05"),
                _ => None,
            }).and_then(room_of) {
                map = map.marker(marker(room.x + room.w / 2.0, room.y + room.h / 2.0, true));
                continue;
            }
            // Only the area: one of its rooms in turn, so they spread over it.
            let Some(area) = item.location.as_deref() else { continue };
            let Some((game, _)) = GAME_AREAS.iter().find(|(_, a)| *a == area) else { continue };
            let rooms: Vec<&Room> = w.rooms.iter().filter(|r| r.area == *game && !r.scene.contains('(')).collect();
            if rooms.is_empty() {
                continue;
            }
            let n = match per_area.iter_mut().find(|(a, _)| *a == *game) {
                Some((_, n)) => {
                    *n += 1;
                    *n
                }
                None => {
                    per_area.push((game, 0));
                    0
                }
            };
            // A prime stride walks the rooms out of order, so neighbours differ.
            let room = rooms[(n * 7) % rooms.len()];
            map = map.marker(marker(room.x + room.w / 2.0, room.y + room.h / 2.0, true));
        }
    }
    // On the map for completeness, shown on demand.
    for section in sections.iter().filter(|s| minor.contains(&s.id.as_str())) {
        map = map.hide_layer(section.title.clone());
    }
    // The game's own places: benches, stations, merchants. Not things to do,
    // so they carry no item and never count.
    for (i, p) in w.pins.iter().enumerate() {
        if let Some((label, layer)) = point_of_interest(&p.sprite, fr) {
            // The game's own pin for it.
            map = map.marker(MapMarker::at(format!("poi-{i}"), label, p.x, p.y).category(layer).icon(format!("game:{}", p.sprite)));
        }
    }
    map
}

/// Where each tracked item is found.
fn area_of(id: &str) -> Option<&'static str> {
    if let Some(n) = id.strip_prefix("gotCharm_") {
        let area = match n.parse::<u8>().ok()? {
            1 | 2 | 4 | 6 | 15 | 26 | 37 | 40 => "Dirtmouth",
            3 | 7 | 8 | 14 | 18 | 19 | 20 | 22 | 35 => "Forgotten Crossroads",
            5 | 27 => "Howling Cliffs",
            9 => "The Abyss",
            10 | 11 => "Royal Waterways",
            12 | 28 => "Greenpath",
            13 | 17 | 23 | 24 | 25 | 31 => "Fungal Wastes",
            16 | 39 => "Deepnest",
            21 | 30 | 38 => "Resting Grounds",
            29 | 32 => "Kingdom's Edge",
            33 => "City of Tears",
            34 => "Crystal Peak",
            36 => "White Palace",
            _ => return None,
        };
        return Some(area);
    }
    Some(match id {
        "hasDash" | "hornet1Defeated" | "hasDashSlash" | "fireballLevel1" | "noEyesDefeated" => "Greenpath",
        "hasWalljump" | "defeatedMantisLords" | "elderHuDefeated" => "Fungal Wastes",
        "hasSuperDash" | "quakeLevel2" => "Crystal Peak",
        "hasDoubleJump" | "killedInfectedKnight" => "Ancient Basin",
        "hasAcidArmour" | "defeatedDungDefender" | "killedWhiteDefender" => "Royal Waterways",
        "hasShadowDash" | "screamLevel2" => "The Abyss",
        "hasKingsBrand" | "hasUpwardSlash" | "hornetOutskirtsDefeated" | "markothDefeated" | "killedHiveKnight"
        | "colosseumBronzeCompleted" | "colosseumSilverCompleted" | "colosseumGoldCompleted" => "Kingdom's Edge",
        "hasDreamNail" | "dreamNailUpgraded" | "mothDeparted" | "xeroDefeated" => "Resting Grounds",
        "hasLantern" | "killedGrimm" | "grimmRitual" | "killedNightmareGrimm" | "nightmareKing" | "greyPrinceDefeated" => "Dirtmouth",
        "hasCyclone" | "aladarSlugDefeated" => "Howling Cliffs",
        "fireballLevel2" | "quakeLevel1" | "mageLordDefeated" | "killedBlackKnight" | "collectorDefeated" | "lurienDefeated"
        | "nailSmithUpgrades" => "City of Tears",
        "screamLevel1" | "monomonDefeated" | "defeatedMegaJelly" => "Fog Canyon",
        "killedBigFly" | "falseKnightDefeated" | "killedMawlek" | "killedHollowKnight" | "killedFinalBoss" => "Forgotten Crossroads",
        "killedTraitorLord" | "mumCaterpillarDefeated" => "Queen's Gardens",
        "hegemolDefeated" | "galienDefeated" | "killedMimicSpider" => "Deepnest",
        "hasGodfinder" => "Godhome",
        id if id.starts_with("pantheon") => "Godhome",
        _ => return None,
    })
}

fn wiki(name: &str) -> String {
    format!("{WIKI}{}", name.replace(' ', "_"))
}

/// Charm names by their number in the save, `gotCharm_1` to `gotCharm_40`.
const CHARMS: [&str; 40] = [
    "Gathering Swarm", "Wayward Compass", "Grubsong", "Stalwart Shell", "Baldur Shell",
    "Fury of the Fallen", "Quick Focus", "Lifeblood Heart", "Lifeblood Core", "Defender's Crest",
    "Flukenest", "Thorns of Agony", "Mark of Pride", "Steady Body", "Heavy Blow",
    "Sharp Shadow", "Spore Shroom", "Longnail", "Shaman Stone", "Soul Catcher",
    "Soul Eater", "Glowing Womb", "Fragile Heart", "Fragile Greed", "Fragile Strength",
    "Nailmaster's Glory", "Joni's Blessing", "Shape of Unn", "Hiveblood", "Dream Wielder",
    "Dashmaster", "Quick Slash", "Spell Twister", "Deep Focus", "Grubberfly's Elegy",
    "Kingsoul", "Sprintmaster", "Dreamshield", "Weaversong", "Grimmchild",
];
