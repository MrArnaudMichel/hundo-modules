//! Clair Obscur: Expedition 33: reads `EXPEDITION_N.sav`, an Unreal save,
//! and reports the weapons, pictos, outfits, haircuts and expedition
//! journals held in its inventory, area by area.
//!
//! Which inventory key stands for which item, and where each is found, come
//! from data/items.json: facts gathered by the community tracker
//! MahmoudHamdy02/e33tracker, the names derived from the keys. Items only
//! reachable in New Game+ or after one ending are left out, as there.

mod gvas;

use std::collections::HashSet;

use hundo_sdk::{DriverError, Input, Item, Report, Section, Spoiler, Stat};
use serde::Deserialize;

struct Expedition33;

impl hundo_sdk::Guest for Expedition33 {
    fn analyze(input: Input) -> Result<Report, DriverError> {
        analyze(&input)
    }
}

hundo_sdk::export!(Expedition33);

const ITEMS: &str = include_str!("../data/items.json");

#[derive(Deserialize)]
struct Data {
    items: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
    kind: String,
    name: String,
    /// The inventory keys that mean the player has it: one, or a few for
    /// items the game renamed.
    keys: Vec<String>,
    areas: Vec<String>,
    #[serde(default)]
    character: Option<String>,
}

const CHARACTERS: [&str; 6] = ["Gustave", "Maelle", "Lune", "Sciel", "Verso", "Monoco"];

/// Who joins the expedition late: their outfits name them.
const LATE_CHARACTERS: [&str; 2] = ["Verso", "Monoco"];

/// Areas of the last act: what is found there says how the story goes.
const LATE_AREAS: [&str; 3] = ["Lumiere (Act III)", "Verso's Drafts", "Renoir's Drafts"];

pub fn analyze(input: &Input) -> Result<Report, DriverError> {
    let file = input.files.first().ok_or_else(|| DriverError::MissingFile("EXPEDITION_N.sav".into()))?;
    if file.bytes.len() < 4 {
        return Err(DriverError::Incomplete("sauvegarde en cours d'écriture".into()));
    }
    let inventory = gvas::name_int_map(&file.bytes, "InventoryItems_0")?
        .ok_or_else(|| DriverError::UnsupportedVersion("pas d'inventaire (InventoryItems_0) dans cette sauvegarde".into()))?;
    let held: HashSet<&str> = inventory.iter().filter(|(_, n)| *n > 0).map(|(k, _)| k.as_str()).collect();

    let fr = input.is_french();
    let t = |f: &'static str, e: &'static str| if fr { f } else { e };
    let data: Data = serde_json::from_str(ITEMS).expect("data/items.json is checked by the tests");

    let item = |e: &Entry| {
        let done = e.keys.iter().any(|k| held.contains(k.as_str()));
        let mut item = Item::check(e.id.clone(), e.name.clone(), done).areas(e.areas.iter().cloned());
        if let Some(first) = e.areas.first() {
            item = item.location(first.clone());
        }
        let late = !e.areas.is_empty() && e.areas.iter().all(|a| LATE_AREAS.contains(&a.as_str()));
        let named_journal = e.kind == "journal" && !e.name.starts_with("Expedition ");
        let late_character = e.character.as_deref().is_some_and(|c| LATE_CHARACTERS.contains(&c));
        if late {
            item = item.spoiler(Spoiler::Major);
        } else if named_journal || late_character {
            item = item.spoiler(Spoiler::Minor);
        }
        item
    };
    let of = |kind: &'static str| data.items.iter().filter(move |e| e.kind == kind);

    let mut report = Report::empty();
    report.warnings.push(
        t(
            "Module pas encore vérifié sur une vraie sauvegarde : signale tout écart.",
            "This module has not been checked against a real save yet: report anything off.",
        )
        .into(),
    );
    report.sections.push(Section::new("weapons", t("Armes", "Weapons"), of("weapon").map(item).collect()));
    report.sections.push(Section::new("pictos", "Pictos", of("picto").map(item).collect()));
    for (kind, id, group) in [("outfit", "outfits", t("Tenues", "Outfits")), ("haircut", "haircuts", t("Coiffures", "Haircuts"))] {
        for c in CHARACTERS {
            let items: Vec<Item> = of(kind).filter(|e| e.character.as_deref() == Some(c)).map(item).collect();
            if !items.is_empty() {
                report.sections.push(Section::new(format!("{id}-{}", c.to_ascii_lowercase()), c, items).group(group));
            }
        }
    }
    report.sections.push(Section::new("journals", t("Journaux d'expédition", "Expedition journals"), of("journal").map(item).collect()));

    let count = |kind: &'static str| of(kind).filter(|e| e.keys.iter().any(|k| held.contains(k.as_str()))).count();
    report.stats = vec![
        Stat::new("weapons", t("Armes", "Weapons"), format!("{} / {}", count("weapon"), of("weapon").count())),
        Stat::new("pictos", "Pictos", format!("{} / {}", count("picto"), of("picto").count())),
        Stat::new("inventory", t("Objets en inventaire", "Inventory entries"), held.len().to_string()),
    ];
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hundo_sdk::{SaveFile, Slot, Status};

    fn input(bytes: Vec<u8>) -> Input {
        Input {
            slot: Slot { id: "0".into(), label: "Expédition 0".into(), captures: Vec::new() },
            files: vec![SaveFile { path: "76561198000000000/EXPEDITION_0.sav".into(), bytes, modified: 0 }],
            locale: "fr".into(),
            options: Vec::new(),
        }
    }

    fn status(report: &Report, id: &str) -> Status {
        report.sections.iter().flat_map(|s| &s.items).find(|i| i.id == id).unwrap_or_else(|| panic!("{id}")).status
    }

    #[test]
    fn the_data_is_whole_and_ids_are_unique() {
        let data: Data = serde_json::from_str(ITEMS).unwrap();
        let mut ids = HashSet::new();
        for e in &data.items {
            assert!(ids.insert(e.id.as_str()), "{} twice", e.id);
            assert!(!e.keys.is_empty() && !e.name.is_empty(), "{}", e.id);
            if matches!(e.kind.as_str(), "outfit" | "haircut") {
                assert!(CHARACTERS.contains(&e.character.as_deref().unwrap_or("")), "{}", e.id);
            }
        }
        let n = |k: &str| data.items.iter().filter(|e| e.kind == k).count();
        assert_eq!((n("weapon"), n("picto"), n("journal")), (116, 210, 49));
    }

    #[test]
    fn held_items_are_done() {
        let save = gvas::tests::complete(&[("Verleso", 1), ("SkinGustave_Baguette", 1), ("Journal_Exp34", 1), ("Unknown", 1)]);
        let report = analyze(&input(save)).unwrap();
        assert_eq!(status(&report, "weapon-Verleso"), Status::Done);
        assert_eq!(status(&report, "outfit-OutfitGustave_Baguette"), Status::Done);
        assert_eq!(status(&report, "journal-Journal34"), Status::Done);
        assert_eq!(status(&report, "weapon-Abysseram"), Status::Todo);
        assert!(report.sections.iter().any(|s| s.group.as_deref() == Some("Tenues")));
    }

    #[test]
    fn a_save_without_inventory_is_not_read_as_empty() {
        let mut save = b"GVAS".to_vec();
        save.extend_from_slice(&[0; 32]);
        assert!(matches!(analyze(&input(save)), Err(DriverError::UnsupportedVersion(_))));
        assert!(matches!(analyze(&input(Vec::new())), Err(DriverError::Incomplete(_))));
    }
}
