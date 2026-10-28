//! Hollow Knight: Silksong: same save format as Hollow Knight, checked
//! against the 100 % list in `data/checks.json`.

mod gear;
mod hints;
mod icons;
mod journal;
mod pickups;
mod save;
mod shops;

use hundo_sdk::{Completion, DriverError, GameMap, Input, Item, MapMarker, Report, Section, Stat};
use serde::Deserialize;
use std::collections::HashMap;

use serde_json::{Map, Value};

struct Silksong;

impl hundo_sdk::Guest for Silksong {
    fn analyze(input: Input) -> Result<Report, DriverError> {
        analyze(&input)
    }
}

hundo_sdk::export!(Silksong);

#[derive(Deserialize)]
struct Data {
    sections: Vec<SectionDef>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SectionDef {
    id: String,
    title: String,
    group: String,
    /// How many of these make one percent (4 mask shards, 2 spool fragments).
    per_percent: u32,
    items: Vec<ItemDef>,
}

#[derive(Deserialize)]
struct ItemDef {
    id: String,
    name: String,
    location: String,
    wiki: Option<String>,
    check: Check,
}

/// How the save records one entry.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum Check {
    /// Any of these playerData booleans is true.
    Flag { keys: Vec<String> },
    /// Any of these tools is unlocked.
    Tool { names: Vec<String> },
    Needle { level: i64 },
    /// A scene the player has been to.
    Scene { scene: String },
    /// A playerData string that is set.
    Text { key: String },
    #[serde(rename_all = "camelCase")]
    IntAtLeast { key: String, value: i64 },
    /// A wish (quest) completed.
    Quest { name: String },
    /// A persistent object of a scene, picked up or opened.
    #[serde(rename_all = "camelCase")]
    World { scene: String, id: String, or_int_at_least: Option<(String, i64)> },
}

const CHECKS: &str = include_str!("../data/checks.json");

pub fn analyze(input: &Input) -> Result<Report, DriverError> {
    let file = input.files.first().ok_or_else(|| DriverError::MissingFile("userN.dat".into()))?;
    let save = save::decode(&file.bytes)?;
    let pd = save.get("playerData").and_then(Value::as_object).ok_or_else(|| DriverError::Corrupt("playerData absent".into()))?;
    let world = save["sceneData"]["persistentBools"]["serializedList"].as_array().map(Vec::as_slice).unwrap_or_default();
    let data: Data = serde_json::from_str(CHECKS).expect("data/checks.json is checked by the tests");
    let scene_data = save.get("sceneData");
    // The map Hundo drew from the installed game, when there is one.
    let map_world: Option<World> = input.file("@maps/pharloom.json").ok().and_then(|f| serde_json::from_slice::<World>(&f.bytes).ok());
    // Where each item lies, when the game's scenes say: item id → pickup.
    // Where each item lies or is sold: item id → (scene, fraction from the
    // left, from the bottom; negative when unknown).
    let mut spots: Vec<(String, &'static str, f32, f32)> = Vec::new();
    // The wares already listed by the 100 % list.
    let mut sold: Vec<(&'static str, &'static str, &'static str)> = Vec::new();
    let fr = input.is_french();
    let t = |f: &'static str, e: &'static str| if fr { f } else { e };
    let s = State { pd, world };
    // The game's own words, in the reader's language, English filling the gaps.
    let mut text = HashMap::new();
    for l in ["EN", if fr { "FR" } else { "EN" }] {
        for sheet in ["Journal", "Map Zones", "Tools", "UI", "Quests", "Belltown"] {
            if let Ok(f) = input.file(&format!("@texts/{l}_{sheet}")) {
                text.extend(save::sheet(&f.bytes));
            }
        }
    }
    let words = Words { text };

    let mut report = Report::empty();
    report.game_version = pd.get("version").and_then(Value::as_str).map(str::to_string);
    report.completion = Some(Completion {
        label: t("Complétion du jeu", "Game completion").into(),
        value: s.num("completionPercentage"),
        max: 100.0,
    });
    report.stats = vec![
        Stat::new("playTime", t("Temps de jeu", "Play time"), hundo_sdk::format_duration(s.num("playTime"))),
        Stat::new("rosaries", t("Chapelets", "Rosaries"), s.int("geo").to_string()),
        Stat::new("shards", t("Éclats de coquille", "Shell Shards"), s.int("ShellShards").to_string()),
        Stat::new("masks", t("Masques", "Masks"), s.int("maxHealthBase").to_string()),
        Stat::new("silk", t("Soie", "Silk"), s.int("silkMax").to_string()),
    ];
    if s.int("permadeathMode") > 0 {
        report.warnings.push(t("Partie en mode Acier (Steel Soul).", "Steel Soul save.").into());
    }

    let complete = t("Complétion 100 %", "100 % completion");
    let beyond = t("Au-delà du 100 %", "Beyond 100 %");
    for def in data.sections {
        let mut seen: Vec<(String, u32)> = Vec::new();
        let items = def
            .items
            .iter()
            .enumerate()
            .map(|(index, it)| {
                // Entries sharing a name ("Mask Shards") get a number; a
                // numbered collection counts all its pieces as one.
                let stem = |n: &str| n.split(" #").next().unwrap_or(n).to_string();
                let shared = def.items.iter().filter(|o| stem(&o.name) == stem(&it.name)).count() > 1;
                // Loose names of the list ("Upgrades"): the game's own, and its picture.
                let look = match &it.check {
                    Check::Needle { level } => Some(icons::needle(*level)),
                    _ => icons::entry(&def.id, &it.name).map(|(k, s)| (k.to_string(), s.to_string())),
                };
                let name = look.as_ref().and_then(|(key, _)| words.get(key)).unwrap_or_else(|| it.name.clone());
                let title = if let Some(name) = collection_name(&def.id, fr) {
                    format!("{name} {}", index + 1)
                } else if shared {
                    let n = match seen.iter_mut().find(|(seen, _)| *seen == stem(&it.name)) {
                        Some((_, n)) => {
                            *n += 1;
                            *n
                        }
                        None => {
                            seen.push((stem(&it.name), 1));
                            1
                        }
                    };
                    format!("{name} {n}")
                } else {
                    name
                };
                let mut item = Item::check(it.id.clone(), title, s.passes(&it.check)).hint(it.location.clone());
                let spot = match &it.check {
                    Check::World { scene, id, .. } => pickups::spot(scene, id),
                    Check::Tool { names } => pickups::tool_spot(names),
                    _ => None,
                };
                if let Some(p) = spot {
                    spots.push((it.id.clone(), p.scene, p.x, p.y));
                }
                // Bought: from whom, for how much, and where they stand.
                let (flags, tools) = match &it.check {
                    Check::Flag { keys } => (keys.clone(), Vec::new()),
                    Check::Tool { names } => (Vec::new(), names.clone()),
                    _ => (Vec::new(), Vec::new()),
                };
                if let Some(w) = shops::ware_for(&flags, &tools) {
                    item.hint = Some(shops::price(w, fr));
                    if spot.is_none() {
                        spots.push((it.id.clone(), w.scene, w.x, w.y));
                    }
                    sold.push((w.merchant, w.flag, w.gives));
                }
                if let Some((_, sprite)) = &look {
                    item = item.image(format!("game:{sprite}"));
                }
                if let Some(w) = &it.wiki {
                    item = item.link(w.clone());
                }
                // Fleas at their pin, spool fragments: the game's own pictures.
                match def.id.as_str() {
                    "flea" => item = item.image("game:pin_flea"),
                    "spoolFragments" => item = item.image("game:Inv_Hornet_Spool_0000_1_half"),
                    "maskShards" => item = item.image("game:Inv_0030_inv_health_backboard_mask_1"),
                    "silkHeart" => item = item.image("game:silk_heart_inv_icon"),
                    _ => {}
                }
                // A tool or a crest: the game's own name, words and picture.
                let gear = match &it.check {
                    Check::Tool { names } => names.first().and_then(|n| gear::find(n)),
                    _ if def.id == "crests" => {
                        gear::CRESTS.iter().find(|(name, _)| *name == it.name).and_then(|(_, c)| gear::find(&format!("crest:{c}")))
                    }
                    _ => None,
                };
                if let Some((_, name, desc, sprite)) = gear {
                    if let Some(n) = words.get(name) {
                        item.title = n;
                    }
                    if let Some(d) = words.get(desc) {
                        item = item.details(d);
                    }
                    if !sprite.is_empty() {
                        item = item.image(format!("game:{sprite}"));
                    }
                }
                item
            })
            .collect();
        let base = if fr { french_title(&def.id).unwrap_or(&def.title).to_string() } else { def.title.clone() };
        let title = match def.per_percent {
            1 => base,
            n => format!("{base} ({n} = 1 %)"),
        };
        let group = if def.group == "complete" { complete } else { beyond };
        let section = Section::new(def.id.clone(), title, items).group(group);
        // Collections whose pictures say more than their names.
        let gallery = matches!(def.id.as_str(), "crests" | "redTools" | "blueTools" | "yellowTools");
        report.sections.push(if gallery { section.gallery() } else { section });
    }
    report.sections.push(
        Section::new("journal", t("Journal de chasse", "Hunter's Journal"), journal::items(pd, &words, fr)).group(t("Bestiaire", "Bestiary")).gallery(),
    );
    // Each item's area, by the game's name, from where the 100 % list says it is.
    for item in report.sections.iter_mut().flat_map(|s| s.items.iter_mut()) {
        if item.areas.is_empty()
            && let Some(hint) = &item.hint
            && let Some((_, area)) = PLACES.iter().find(|(place, _)| hint.starts_with(place))
        {
            item.areas.push(words.zone(area));
        }
    }
    // The map Hundo drew from the installed game, when there is one.
    // What the merchants sell that the 100 % list does not name, when the
    // save can tell whether it was bought.
    let wares: Vec<Item> = shops::WARES
        .iter()
        .filter(|w| (!w.flag.is_empty() || w.tool) && !sold.contains(&(w.merchant, w.flag, w.gives)))
        .enumerate()
        .map(|(n, w)| {
            let bought = if w.flag.is_empty() { s.passes(&Check::Tool { names: vec![w.gives.to_string()] }) } else { s.passes(&Check::Flag { keys: vec![w.flag.to_string()] }) };
            let id = format!("shop-{}-{n}", w.merchant);
            spots.push((id.clone(), w.scene, w.x, w.y));
            let title = words.get(w.text).unwrap_or_else(|| w.gives.to_string());
            let mut item = Item::check(id, title, bought).hint(shops::price(w, fr));
            // The picture of what it gives: the tool's own, or a shard's.
            let picture = match w.gives {
                "Heart Piece" => Some("Inv_0030_inv_health_backboard_mask_1"),
                "Silk Spool" => Some("Inv_Hornet_Spool_0000_1_half"),
                "" => icons::of(w.text),
                g => gear::find(g).map(|g| g.3).filter(|s| !s.is_empty()).or_else(|| icons::of(g)),
            };
            if let Some(p) = picture {
                item = item.image(format!("game:{p}"));
            }
            item
        })
        .collect();
    if !wares.is_empty() {
        report.sections.push(Section::new("shops", t("Boutiques", "Shops"), wares).group(beyond));
    }
    if let Some(w) = &map_world {
        // Everything else lying in the rooms, on the map for completeness.
        let area_of_scene = |scene: &str| w.rooms.iter().find(|r| r.scene.eq_ignore_ascii_case(scene)).map(|r| words.zone(&r.area));
        let finds = t("Trouvailles", "Finds");
        for (id, kind, title) in [
            ("items", pickups::Kind::Item, t("Objets au sol", "Items on the ground")),
            ("rosaries", pickups::Kind::Rosary, t("Caches de chapelets", "Rosary caches")),
            ("rocks", pickups::Kind::Rock, t("Rochers de chapelets", "Rosary rocks")),
            ("fossils", pickups::Kind::Fossil, t("Fossiles d'éclats", "Shell shard fossils")),
        ] {
            let items = pickups::items(kind, scene_data, &words, fr, &area_of_scene);
            spots.extend(pickups::PICKUPS.iter().filter(|p| p.kind == kind).map(|p| (pickups::item_id(p), p.scene, p.x, p.y)));
            // Rosaries and shell shards: needed for nothing, shown on demand.
            let section = Section::new(id, title, items).group(finds);
            report.sections.push(if kind == pickups::Kind::Item { section } else { section.optional() });
        }
        let scenes: Vec<(String, String)> = serde_json::from_str::<Data>(CHECKS)
            .expect("checked by the tests")
            .sections
            .into_iter()
            .flat_map(|s| s.items)
            .filter_map(|i| match i.check {
                Check::World { scene, .. } => Some((i.id, scene)),
                _ => None,
            })
            .collect();
        report.maps.push(pharloom(w, &report.sections, &scenes, &spots, input.is_french(), &words));
    }
    // The list's hints, in French: last, once the English ones placed each item.
    if fr {
        let table = hints::table();
        for item in report.sections.iter_mut().flat_map(|s| s.items.iter_mut()) {
            if let Some(french) = item.hint.as_deref().and_then(|h| hints::french(h, &words, &table)) {
                item.hint = Some(french);
            }
        }
    }
    Ok(report)
}

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
    x: f32,
    y: f32,
}

/// The game's map pins that are places: what each is called and its layer.
fn point_of_interest(sprite: &str, fr: bool) -> Option<(&'static str, &'static str)> {
    let t = |f: &'static str, e: &'static str| if fr { f } else { e };
    let kind = sprite.trim_end_matches(|c: char| c.is_ascii_digit() || c == ' ' || c == '(' || c == ')' || c == '_');
    Some(match kind {
        "pin_bench" => (t("Banc", "Bench"), t("Bancs", "Benches")),
        "pin_stag_station" => ("Bellway", "Bellways"),
        "pin_tube_station" => ("Ventrica", "Ventrica"),
        "pin_shop" => (t("Marchand", "Merchant"), t("Marchands", "Merchants")),
        "pin_flea" => (t("Caravane des puces", "Flea caravan"), t("Lieux", "Places")),
        _ => return None,
    })
}

/// The game's own text by key (`NAME_MOSSBONE_CRAWLER`), when the host could
/// read it from the installed game.
pub struct Words {
    text: HashMap<String, String>,
}

impl Words {
    /// For tests: a few lines of the game's text.
    #[cfg(test)]
    pub fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        Words { text: pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect() }
    }

    /// A line of the game's text, on one line.
    pub fn get(&self, key: &str) -> Option<String> {
        let t = self.text.get(key)?.replace("<br>", " ");
        let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
        (!t.is_empty()).then_some(t)
    }

    /// A map area's name as the game words it, from its id (`Bone`).
    pub fn zone(&self, area: &str) -> String {
        ZONE_KEYS.iter().find(|(a, _)| *a == area).and_then(|(_, k)| self.get(k)).unwrap_or_else(|| area.replace('_', " "))
    }
}

/// The language sheet's key for each map area of the game.
const ZONE_KEYS: [(&str, &str); 30] = [
    ("Abyss", "ABYSS"),
    ("Ant", "HUNTERS_NEST"),
    ("Aqueduct", "AQUEDUCT"),
    ("Arborium", "ARBORIUM"),
    ("Belltown", "BELLTOWN"),
    ("Blasted_Steps", "JUDGE_STEPS"),
    ("Bone", "PATH_OF_BONE"),
    ("Bonetown", "BONETOWN"),
    ("Clover", "CLOVER"),
    ("Cog", "COG_CORE"),
    ("Coral_Caves", "RED_CORAL_GORGE"),
    ("Cradle", "CRADLE"),
    ("Crawl", "CRAWLSPACE"),
    ("Dock", "DOCKS"),
    ("Dust", "DUSTPENS"),
    ("Greymoor", "GREYMOOR"),
    ("Hang", "HANG"),
    ("Library", "LIBRARY"),
    ("Peak", "PEAK"),
    ("Shellwood", "SHELLWOOD_THICKET"),
    ("Slab", "THE_SLAB"),
    ("Song", "CITY_OF_SONG"),
    ("Song_Gate", "FRONT_GATE"),
    ("Swamp", "SWAMP"),
    ("Tut", "MOSS_CAVE"),
    ("Under", "UNDERSTORE"),
    ("Ward", "WARD"),
    ("Weavehome", "WEAVER_SHRINE"),
    ("Wilds", "WILDS"),
    ("Wisp", "WISP"),
];

/// The game's map areas for the region names the 100 % list uses, where the
/// match is sure; other places are left off the map rather than guessed.
const PLACES: [(&str, &str); 19] = [
    ("The Marrow", "Bone"),
    ("Marrow", "Bone"),
    ("Bone Bottom", "Bonetown"),
    ("Mosshome", "Bonetown"),
    ("Deep Docks", "Dock"),
    ("Greymoor", "Greymoor"),
    ("Shellwood", "Shellwood"),
    ("The Slab", "Slab"),
    ("Mount Fay", "Peak"),
    ("Whiteward", "Ward"),
    ("Underworks", "Under"),
    ("Wormways", "Crawl"),
    ("Cogwork Core", "Cog"),
    ("The Cradle", "Cradle"),
    ("Bellhart", "Belltown"),
    ("Bilewater", "Swamp"),
    ("Sinner's Road", "Dust"),
    ("Hunter's March", "Ant"),
    ("Blasted Steps", "Blasted_Steps"),
];

/// Pharloom from the game's files: areas named, each item at its pin or
/// room when the save says where, else spread over its area.
fn pharloom(w: &World, sections: &[Section], scenes: &[(String, String)], spots: &[(String, &str, f32, f32)], fr: bool, words: &Words) -> GameMap {
    let mut map = GameMap::new("pharloom", "Pharloom", w.width, w.height);
    let mut areas: Vec<&str> = w.rooms.iter().map(|r| r.area.as_str()).collect();
    areas.sort();
    areas.dedup();
    for area in &areas {
        let rooms: Vec<&Room> = w.rooms.iter().filter(|r| r.area == *area).collect();
        let x0 = rooms.iter().map(|r| r.x).fold(f32::MAX, f32::min);
        let y0 = rooms.iter().map(|r| r.y).fold(f32::MAX, f32::min);
        let x1 = rooms.iter().map(|r| r.x + r.w).fold(f32::MIN, f32::max);
        let y1 = rooms.iter().map(|r| r.y + r.h).fold(f32::MIN, f32::max);
        // The game's own name for the area, as it spells it.
        map = map.region(&format!("area:{area}"), words.zone(area), &[(x0, y0), (x1, y0), (x1, y1), (x0, y1)]);
    }
    // A scene's whole box: some rooms are drawn in several pieces.
    let scene_box = |scene: &str| -> Option<(f32, f32, f32, f32)> {
        // A variant of a room (`_Festival`, `_boss`) is drawn as the room itself.
        let mut scene = scene;
        let rooms: Vec<&Room> = loop {
            let rooms: Vec<&Room> = w.rooms.iter().filter(|r| r.scene.eq_ignore_ascii_case(scene)).collect();
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
    let mut spread: Vec<(&str, usize)> = Vec::new();
    for section in sections {
        for item in &section.items {
            let marker = |x: f32, y: f32, approx: bool| {
                let mut m = MapMarker::at(format!("m-{}", item.id), item.title.clone(), x, y).item(item.id.clone()).category(section.title.clone());
                m.approximate = approx;
                m
            };
            if let Some(p) = w.pins.iter().find(|p| p.scene == item.id) {
                map = map.marker(marker(p.x, p.y, false));
                continue;
            }

            // Its very spot in its room, from the game's scenes.
            if let Some((_, scene, px, py)) = spots.iter().find(|(id, ..)| *id == item.id)
                && let Some((x, y, bw, bh)) = scene_box(scene)
            {
                // Its room is sure even when its spot in it is not.
                let (fx, fy) = if *px >= 0.0 { (*px, *py) } else { (0.5, 0.5) };
                map = map.marker(marker(x + fx * bw, y + (1.0 - fy) * bh, false));
                continue;
            }
            let scene = scenes.iter().find(|(id, _)| *id == item.id).map(|(_, s)| s.as_str());
            if let Some(room) = scene.and_then(|s| w.rooms.iter().find(|r| r.scene == s)) {
                map = map.marker(marker(room.x + room.w / 2.0, room.y + room.h / 2.0, false));
                continue;
            }
            // The 100 % list names the place: one of its area's rooms in turn.
            let place = item.hint.as_deref().and_then(|hint| PLACES.iter().find(|(place, _)| hint.starts_with(place)));
            let Some((_, area)) = place else {
                // Else the area it belongs to, the same room for the same item.
                let Some(id) = item.areas.first().and_then(|a| areas.iter().find(|id| words.zone(id) == *a)) else { continue };
                let rooms: Vec<&Room> = w.rooms.iter().filter(|r| r.area == *id).collect();
                let n = item.id.bytes().fold(0usize, |h, b| h.wrapping_mul(31).wrapping_add(b as usize));
                if let Some(room) = rooms.get(n % rooms.len().max(1)) {
                    map = map.marker(marker(room.x + room.w / 2.0, room.y + room.h / 2.0, true));
                }
                continue;
            };
            let rooms: Vec<&Room> = w.rooms.iter().filter(|r| r.area == *area).collect();
            if rooms.is_empty() {
                continue;
            }
            let n = match spread.iter_mut().find(|(a, _)| a == area) {
                Some((_, n)) => {
                    *n += 1;
                    *n
                }
                None => {
                    spread.push((area, 0));
                    0
                }
            };
            let room = rooms[(n * 7) % rooms.len()];
            map = map.marker(marker(room.x + room.w / 2.0, room.y + room.h / 2.0, true));
        }
    }
    // On the map for completeness, shown on demand.
    for section in sections.iter().filter(|s| ["items", "rosaries", "rocks", "fossils", "journal"].contains(&s.id.as_str())) {
        map = map.hide_layer(section.title.clone());
    }
    // The game's own places: benches, Bellways, merchants. They never count.
    for (i, p) in w.pins.iter().enumerate() {
        if let Some((label, layer)) = point_of_interest(&p.sprite, fr) {
            // The game's own pin for it.
            map = map.marker(MapMarker::at(format!("poi-{i}"), label, p.x, p.y).category(layer).icon(format!("game:{}", p.sprite)));
        }
    }
    map
}

/// The name of one piece of a numbered collection, in the reader's language.
fn collection_name(section: &str, fr: bool) -> Option<&'static str> {
    Some(match (section, fr) {
        ("maskShards", true) => "Fragment de masque",
        ("maskShards", false) => "Mask shard",
        ("spoolFragments", true) => "Fragment de bobine",
        ("spoolFragments", false) => "Spool fragment",
        ("silkHeart", true) => "Cœur de soie",
        ("silkHeart", false) => "Silk heart",
        _ => return None,
    })
}

fn french_title(section: &str) -> Option<&'static str> {
    Some(match section {
        "crests" => "Emblèmes",
        "NeedleUpgrades" => "Améliorations de l'aiguille",
        "silkHeart" => "Cœurs de soie",
        "redTools" => "Outils rouges",
        "blueTools" => "Outils bleus",
        "yellowTools" => "Outils jaunes",
        "equipment" => "Compétences de soie",
        "skills" => "Capacités",
        "miscellaneous" => "Divers",
        "maskShards" => "Fragments de masque",
        "spoolFragments" => "Fragments de bobine",
        "craftingKitUpgrades" => "Améliorations du kit d'artisan",
        "toolPouchUpgrades" => "Améliorations de la sacoche",
        "essentialsMemoryLocket" => "Médaillons de mémoire",
        "essentialsCraftmetal" => "Métal d'artisan",
        "essentialsKeys" => "Clés",
        "flea" => "Puces perdues",
        _ => return None,
    })
}

struct State<'a> {
    pd: &'a Map<String, Value>,
    world: &'a [Value],
}

impl State<'_> {
    fn num(&self, key: &str) -> f64 {
        self.pd.get(key).and_then(Value::as_f64).unwrap_or(0.0)
    }

    fn int(&self, key: &str) -> i64 {
        self.pd.get(key).and_then(Value::as_i64).unwrap_or(0)
    }

    fn saved<'b>(&'b self, list: &str) -> impl Iterator<Item = &'b Value> {
        self.pd.get(list).and_then(|l| l["savedData"].as_array()).into_iter().flatten()
    }

    fn passes(&self, check: &Check) -> bool {
        match check {
            Check::Flag { keys } => keys.iter().any(|k| self.pd.get(k) == Some(&Value::Bool(true))),
            Check::Tool { names } => self.saved("Tools").any(|t| names.iter().any(|n| t["Name"] == **n) && t["Data"]["IsUnlocked"] == true),
            Check::Needle { level } => self.int("nailUpgrades") >= *level,
            Check::Scene { scene } => self.pd.get("scenesVisited").and_then(Value::as_array).is_some_and(|v| v.iter().any(|s| s == scene)),
            Check::Text { key } => self.pd.get(key).and_then(Value::as_str).is_some_and(|s| !s.is_empty()),
            Check::IntAtLeast { key, value } => self.int(key) >= *value,
            Check::Quest { name } => self.saved("QuestCompletionData").any(|q| q["Name"] == **name && q["Data"]["IsCompleted"] == true),
            Check::World { scene, id, or_int_at_least } => {
                self.world.iter().any(|w| w["SceneName"] == **scene && w["ID"] == **id && w["Value"] == true)
                    || or_int_at_least.as_ref().is_some_and(|(k, v)| self.int(k) >= *v)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn checks_add_up_to_one_hundred() {
        let data: super::Data = serde_json::from_str(super::CHECKS).unwrap();
        let percent: usize =
            data.sections.iter().filter(|s| s.group == "complete").map(|s| s.items.len() / s.per_percent as usize).sum();
        assert_eq!(percent, 100);
    }
}
