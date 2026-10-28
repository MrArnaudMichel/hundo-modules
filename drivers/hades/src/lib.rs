//! Hades: reads `ProfileN.sav` and reports the arms, the guardians, the
//! Fated List and the house's relationships.

mod save;

use hundo_sdk::{DriverError, Input, Item, GameMap, Report, Section, Spoiler, Stat, Status};
use save::Lua;

struct Hades;

impl hundo_sdk::Guest for Hades {
    fn analyze(input: Input) -> Result<Report, DriverError> {
        analyze(&input)
    }
}

hundo_sdk::export!(Hades);

const WEAPONS: [(&str, &str); 6] = [
    ("SwordWeapon", "Stygius"),
    ("SpearWeapon", "Varatha"),
    ("ShieldWeapon", "Aegis"),
    ("BowWeapon", "Coronacht"),
    ("FistWeapon", "Malphon"),
    ("GunWeapon", "Exagryph"),
];

/// Guardians by the name the game counts their kills under, and their realm.
const GUARDIANS: [(&str, &str, &str); 7] = [
    ("Harpy", "Megaera", "Tartarus"),
    ("Harpy2", "Alecto", "Tartarus"),
    ("Harpy3", "Tisiphone", "Tartarus"),
    ("HydraHeadImmortal", "Lernie", "Asphodel"),
    ("Theseus", "Theseus", "Elysium"),
    ("Minotaur", "Asterius", "Elysium"),
    ("Hades", "Hades", "Styx"),
];

/// A realm: its id, its French and English names, its outline on the map.
type Realm = (&'static str, &'static str, &'static str, [(f32, f32); 4]);

/// The realms, from the House up to the Surface, as bands of a 1000 × 1400
/// map: Zagreus climbs from the bottom to the top.
const REALMS: [Realm; 6] = [
    ("House", "Maison d'Hadès", "House of Hades", [(260.0, 1230.0), (740.0, 1230.0), (800.0, 1380.0), (200.0, 1380.0)]),
    ("Tartarus", "Tartare", "Tartarus", [(120.0, 960.0), (880.0, 960.0), (840.0, 1210.0), (160.0, 1210.0)]),
    ("Asphodel", "Asphodèle", "Asphodel", [(160.0, 700.0), (840.0, 700.0), (880.0, 940.0), (120.0, 940.0)]),
    ("Elysium", "Élysée", "Elysium", [(140.0, 440.0), (860.0, 440.0), (840.0, 680.0), (160.0, 680.0)]),
    ("Styx", "Temple du Styx", "Temple of Styx", [(200.0, 200.0), (800.0, 200.0), (860.0, 420.0), (140.0, 420.0)]),
    ("Surface", "Surface", "Surface", [(300.0, 30.0), (700.0, 30.0), (780.0, 180.0), (220.0, 180.0)]),
];

const NPCS: [(&str, &str); 18] = [
    ("NPC_Nyx_01", "Nyx"),
    ("NPC_Thanatos_01", "Thanatos"),
    ("NPC_FurySister_01", "Megaera"),
    ("NPC_Dusa_01", "Dusa"),
    ("NPC_Achilles_01", "Achilles"),
    ("NPC_Hypnos_01", "Hypnos"),
    ("NPC_Cerberus_01", "Cerberus"),
    ("NPC_Orpheus_01", "Orpheus"),
    ("NPC_Skelly_01", "Skelly"),
    ("NPC_Charon_01", "Charon"),
    ("NPC_Sisyphus_01", "Sisyphus"),
    ("NPC_Eurydice_01", "Eurydice"),
    ("NPC_Patroclus_01", "Patroclus"),
    ("NPC_Bouldy_01", "Bouldy"),
    ("NPC_Persephone_Home_01", "Persephone"),
    ("NPC_Hades_01", "Hades"),
    ("NPC_Thanatos_Field_01", "Thanatos"),
    ("NPC_Dusa_Field_01", "Dusa"),
];

const GODS: [(&str, &str); 10] = [
    ("ZeusUpgrade", "Zeus"),
    ("PoseidonUpgrade", "Poseidon"),
    ("AthenaUpgrade", "Athena"),
    ("AresUpgrade", "Ares"),
    ("AphroditeUpgrade", "Aphrodite"),
    ("ArtemisUpgrade", "Artemis"),
    ("DionysusUpgrade", "Dionysus"),
    ("HermesUpgrade", "Hermes"),
    ("DemeterUpgrade", "Demeter"),
    ("TrialUpgrade", "Chaos"),
];

pub fn analyze(input: &Input) -> Result<Report, DriverError> {
    let file = input.files.first().ok_or_else(|| DriverError::MissingFile("ProfileN.sav".into()))?;
    let save = save::decode(&file.bytes)?;
    let fr = input.is_french();
    let t = |f: &'static str, e: &'static str| if fr { f } else { e };
    let state = &save.state;
    let gs = state.get("GameState");

    let mut report = Report::empty();
    if let Lua::Num(rev) = state.get("Revision") {
        report.game_version = Some(format!("{rev}"));
    }
    if save.god_mode {
        report.warnings.push(t("Mode Dieu activé sur ce profil.", "God Mode is on for this profile.").into());
    }

    let gained = gs.get("LifetimeResourcesGained");
    let codex = state.get("CodexStatus").entries().flat_map(|(_, cat)| cat.entries()).filter(|(_, e)| e.at(1).get("Unlocked").truthy()).count();
    let escapes = gs.get("RunHistory").entries().filter(|(_, r)| r.get("Cleared").truthy()).count()
        + usize::from(state.get("CurrentRun").get("Cleared").truthy());
    report.stats = vec![
        Stat::new("runs", t("Tentatives", "Runs"), save.runs.to_string()),
        Stat::new("escapes", t("Évasions", "Escapes"), escapes.to_string()),
        Stat::new("darkness", t("Ténèbres gagnées", "Darkness earned"), (gained.get("MetaPoints").num() as u64).to_string()),
        Stat::new("gems", t("Gemmes gagnées", "Gems earned"), (gained.get("Gems").num() as u64).to_string()),
        Stat::new("nectar", t("Nectar reçu", "Nectar received"), (gained.get("GiftPoints").num() as u64).to_string()),
        Stat::new("codex", t("Entrées du Codex", "Codex entries"), codex.to_string()),
        Stat::new("mirror", t("Talents du Miroir", "Mirror talents"),
            gs.get("MetaUpgradesUnlocked").entries().filter(|(k, _)| !k.contains("Shrine")).count().to_string()),
        Stat::new("contractor", t("Ouvrages du Constructeur", "House Contractor works"), gs.get("CosmeticsAdded").len().to_string()),
    ];

    let kills = gs.get("EnemyKills");
    let realm_name = |id: &'static str| -> &'static str { REALMS.iter().find(|r| r.0 == id).map(|r| if fr { r.1 } else { r.2 }).unwrap_or(id) };
    let weapons = WEAPONS
        .iter()
        .map(|(key, name)| Item::check(*key, *name, gs.get("WeaponsUnlocked").get(key).truthy()).location(realm_name("House")))
        .collect();
    let guardians = GUARDIANS
        .iter()
        .map(|(key, name, realm)| {
            let n = kills.get(key).num() as u32;
            let mut item = Item::check(*key, *name, n > 0).location(realm_name(realm));
            if n > 0 {
                item = item.hint(format!("{n} {}", t("fois vaincu", "times defeated")));
            }
            if *key == "Hades" { item.spoiler(Spoiler::Minor) } else { item }
        })
        .collect();

    let mut reached: Vec<&str> = Vec::new();
    for run in gs.get("RunHistory").entries().map(|(_, r)| r).chain([state.get("CurrentRun")]) {
        for (biome, v) in run.get("BiomesReached").entries() {
            if v.truthy() && !reached.contains(&biome) {
                reached.push(biome);
            }
        }
    }
    let realms = REALMS[1..]
        .iter()
        .map(|(id, ..)| Item::check(format!("realm-{id}"), realm_name(id), reached.contains(id) || (*id == "Surface" && escapes > 0)).location(realm_name(id)))
        .collect();

    let prophecies = gs
        .get("QuestStatus")
        .entries()
        .map(|(id, status)| {
            let (status, hint) = match status.str() {
                Some("CashedOut") => (Status::Done, None),
                Some("Complete") => (Status::InProgress, Some(t("Accomplie : récompense à récupérer dans la Liste du destin", "Fulfilled: collect the reward in the Fated List"))),
                _ => (Status::Todo, None),
            };
            let item = Item::new(id, prophecy_title(id, fr), status);
            match hint {
                Some(h) => item.hint(h),
                None => item,
            }
        })
        .collect();

    let gift = gs.get("Gift");
    let hearts = |list: &[(&str, &str)]| -> Vec<Item> {
        let mut seen = Vec::new();
        list.iter()
            .filter(|(_, name)| {
                let fresh = !seen.contains(name);
                seen.push(*name);
                fresh
            })
            .filter_map(|(key, name)| {
                let v = gift.get(key);
                if matches!(v, Lua::Nil) {
                    return None;
                }
                let n = v.get("Value").num() as u32;
                let status = if n > 0 { Status::InProgress } else { Status::Todo };
                Some(Item::new(*key, *name, status).hint(format!("{n} {}", t("cœur(s)", "heart(s)"))))
            })
            .collect()
    };

    let progress = t("Progression", "Progression");
    let bonds = t("Relations", "Relationships");
    report.sections = vec![
        Section::new("weapons", t("Armes infernales", "Infernal Arms"), weapons).group(progress),
        Section::new("guardians", t("Gardiens", "Guardians"), guardians).group(progress),
        Section::new("realms", t("Royaumes atteints", "Realms reached"), realms).group(progress),
        Section::new("prophecies", t("Liste du destin", "Fated List"), prophecies).group(progress),
        Section::new("denizens", t("Habitants des Enfers", "Underworld denizens"), hearts(&NPCS)).group(bonds),
        Section::new("gods", t("Dieux de l'Olympe", "Olympians"), hearts(&GODS)).group(bonds),
    ];
    if !report.sections[3].items.is_empty() {
        report.warnings.push(t(
            "La Liste du destin ne montre que les prophéties déjà révélées par le jeu.",
            "The Fated List only shows the prophecies the game has revealed so far.",
        ).into());
    }

    let mut map = GameMap::new("underworld", t("Les Enfers", "The Underworld"), 1000.0, 1400.0);
    for (id, f, e, points) in REALMS {
        map = map.region(id, if fr { f } else { e }, &points);
    }
    let map = map.place_in_regions(
        &report.sections,
        |item| REALMS.iter().find(|r| Some(r.1) == item.location.as_deref() || Some(r.2) == item.location.as_deref()).map(|r| r.0.to_string()),
        |section, _| section.title.clone(),
    );
    report.maps.push(map);
    Ok(report)
}

/// The Fated List entries, described: the game's own titles live in its text
/// files, not in the save.
fn prophecy_title(id: &str, fr: bool) -> String {
    let god = |g: &str| if fr { format!("Tous les bienfaits : {g}") } else { format!("Every boon: {g}") };
    let hammer = |w: &str| if fr { format!("Améliorations du marteau : {w}") } else { format!("Daedalus Hammer upgrades: {w}") };
    if let Some(g) = id.strip_suffix("Upgrades") {
        if let Some((_, name)) = GODS.iter().find(|(k, _)| k.trim_end_matches("Upgrade") == g) {
            return god(name);
        }
        if let Some(w) = g.strip_suffix("Hammer")
            && let Some((_, name)) = WEAPONS.iter().find(|(k, _)| k.trim_end_matches("Weapon") == w)
        {
            return hammer(name);
        }
    }
    let known: Option<(&str, &str)> = match id {
        "FirstClear" => Some(("S'échapper des Enfers", "Escape the Underworld")),
        "MeetOlympians" => Some(("Rencontrer les dieux de l'Olympe", "Meet the Olympians")),
        "MeetChthonicGods" => Some(("Rencontrer les dieux chthoniens", "Meet the Chthonic gods")),
        "WeaponUnlocks" => Some(("Débloquer toutes les armes infernales", "Unlock every Infernal Arm")),
        "CodexSmall" => Some(("Remplir le Codex", "Fill the Codex")),
        "CosmeticsSmall" => Some(("Commander des ouvrages au Constructeur", "Order House Contractor works")),
        "SynergyUpgrades" => Some(("Obtenir des bienfaits duo", "Obtain Duo boons")),
        "LegendaryUpgrades" => Some(("Obtenir des bienfaits légendaires", "Obtain Legendary boons")),
        "ChaosCurses" => Some(("Les bénédictions du Chaos", "Chaos blessings")),
        "MiniBossKills" => Some(("Vaincre les champions des royaumes", "Defeat the realms' champions")),
        "OrpheusRelease" => Some(("Libérer Orphée", "Free Orpheus")),
        _ => None,
    };
    match known {
        Some((f, e)) => (if fr { f } else { e }).into(),
        None => split_camel(id),
    }
}

/// `AresEarnKills` → `Ares Earn Kills`.
fn split_camel(id: &str) -> String {
    let mut out = String::new();
    for (i, c) in id.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

