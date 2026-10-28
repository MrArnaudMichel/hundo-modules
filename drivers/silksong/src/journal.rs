//! The hunting journal: every creature of Pharloom, with the kills it takes
//! to complete its entry.
//!
//! The save lists the creatures met (`EnemyJournalKillData`) with their kill
//! counts. Names, descriptions and notes come from the game's own language
//! sheet when the host could read it from the installed game, pictures from
//! its sprites.

use std::collections::HashMap;

use hundo_sdk::{Item, Spoiler, Status};
use serde_json::{Map, Value};

/// One entry, identifiers and numbers only, taken from the game's journal
/// records: the save's name, the sheet's key, the picture, the kills that
/// complete it, whether it counts towards completing the journal, and the
/// map areas of the scenes whose enemies point to it, the busiest first.
struct E {
    key: &'static str,
    text: &'static str,
    image: &'static str,
    kills: u32,
    required: bool,
    areas: &'static [&'static str],
}

/// In the game's order (its master journal list).
const ENTRIES: &[E] = &[
    E { key: "MossBone Crawler", text: "MOSSBONE_CRAWLER", image: "MossBone Crawler", kills: 25, required: true, areas: &["Arborium", "Tut", "Weavehome"] },
    E { key: "MossBone Crawler Fat", text: "MOSSBONE_CRAWLER_FAT", image: "MossBone Crawler Fat", kills: 3, required: true, areas: &["Arborium"] },
    E { key: "MossBone Fly", text: "MOSSBONE_FLY", image: "MossBone Fly", kills: 20, required: true, areas: &["Tut", "Arborium", "Weavehome"] },
    E { key: "Mossbone Mother", text: "MOSSBONE_MOTHER", image: "Mossbone Mother", kills: 3, required: true, areas: &["Tut"] },
    E { key: "Aspid Collector", text: "ASPID_COLLECTOR", image: "Aspid Collector", kills: 15, required: true, areas: &["Bonetown", "Crawl", "Shellwood"] },
    E { key: "Bone Goomba", text: "BONE_GOOMBA", image: "Bone Goomba", kills: 35, required: true, areas: &["Bone", "Abyss", "Arborium", "Dock"] },
    E { key: "Bone Goomba Bounce Fly", text: "BONE_GOOMBA_BOUNCE_FLY", image: "Bone Goomba Bounce Fly", kills: 15, required: true, areas: &["Abyss", "Weavehome"] },
    E { key: "Bone Goomba Large", text: "BONE_GOOMBA_LARGE", image: "Bone Goomba Large", kills: 15, required: true, areas: &["Bone", "Dock", "Arborium"] },
    E { key: "Skull King", text: "SKULL_KING", image: "Skull King", kills: 1, required: true, areas: &["Bone", "Bonetown"] },
    E { key: "Bone Crawler", text: "BONE_CRAWLER", image: "Bone Crawler", kills: 25, required: true, areas: &["Bone", "Ant", "Arborium"] },
    E { key: "Bone Flyer", text: "BONE_FLYER", image: "Bone Flyer", kills: 30, required: true, areas: &["Bone", "Dock", "Ant", "Arborium", "Wilds"] },
    E { key: "Bone Flyer Giant", text: "BONE_FLYER_GIANT", image: "Bone Flyer Giant", kills: 2, required: true, areas: &["Ant", "Wilds"] },
    E { key: "Bone Circler", text: "BONE_CIRCLER", image: "Bone Circler", kills: 25, required: true, areas: &["Bone", "Dock", "Wilds", "Ant", "Arborium", "Bonetown"] },
    E { key: "Bone Circler Vicious", text: "BONE_CIRCLER_VICIOUS", image: "Bone Circler Vicious", kills: 15, required: true, areas: &["Wilds", "Dock", "Ant", "Arborium"] },
    E { key: "Bone Hopper", text: "BONE_HOPPER", image: "Bone Hopper", kills: 20, required: true, areas: &["Wilds", "Dock"] },
    E { key: "Bone Hopper Giant", text: "BONE_HOPPER_GIANT", image: "Bone Hopper Giant", kills: 10, required: true, areas: &["Wilds"] },
    E { key: "Bone Spitter", text: "BONE_SPITTER", image: "Bone Spitter", kills: 10, required: true, areas: &["Wilds"] },
    E { key: "Bone Roller", text: "BONE_ROLLER", image: "Bone Roller", kills: 30, required: true, areas: &["Arborium", "Tut", "Weavehome"] },
    E { key: "Bone Thumper", text: "BONE_THUMPER", image: "Bone Thumper", kills: 8, required: true, areas: &["Tut", "Weavehome", "Arborium"] },
    E { key: "Spine Floater", text: "SPINE_FLOATER", image: "Spine Floater", kills: 15, required: true, areas: &["Wilds"] },
    E { key: "Rock Roller", text: "ROCK_ROLLER", image: "Rock Roller", kills: 3, required: false, areas: &["Bone"] },
    E { key: "Rhino", text: "RHINO", image: "Rhino", kills: 2, required: true, areas: &["Wilds", "Arborium"] },
    E { key: "Crypt Worm", text: "CRYPT_WORM", image: "Crypt Worm", kills: 25, required: true, areas: &["Crawl"] },
    E { key: "Bone Worm", text: "BONE_WORM", image: "Bone Worm", kills: 10, required: true, areas: &["Crawl"] },
    E { key: "Bone Beast", text: "BONE_BEAST", image: "Bone Beast", kills: 1, required: true, areas: &["Bone"] },
    E { key: "Pilgrim 03", text: "PILGRIM_03", image: "Pilgrim 03", kills: 35, required: true, areas: &["Bonetown", "Bone", "Wilds", "Blasted_Steps", "Greymoor", "Shellwood"] },
    E { key: "Pilgrim 01", text: "PILGRIM_01", image: "Pilgrim 01", kills: 35, required: true, areas: &["Bone", "Bonetown", "Wilds", "Blasted_Steps", "Greymoor", "Shellwood"] },
    E { key: "Pilgrim 04", text: "PILGRIM_04", image: "Pilgrim 04", kills: 15, required: true, areas: &["Shellwood", "Wilds", "Bonetown", "Crawl", "Greymoor"] },
    E { key: "Pilgrim 02", text: "PILGRIM_02", image: "Pilgrim 02", kills: 10, required: true, areas: &["Wilds", "Dock", "Shellwood"] },
    E { key: "Pilgrim Bell Thrower", text: "PILGRIM_BELL_THROWER", image: "Pilgrim Bell Thrower", kills: 10, required: true, areas: &["Bonetown", "Blasted_Steps", "Greymoor", "Shellwood"] },
    E { key: "Pilgrim Fly", text: "PILGRIM_FLY", image: "Pilgrim Fly", kills: 25, required: true, areas: &["Bone", "Blasted_Steps", "Wilds", "Bonetown", "Crawl", "Greymoor", "Shellwood"] },
    E { key: "Pilgrim 05", text: "PILGRIM_05", image: "Pilgrim 05", kills: 5, required: true, areas: &["Bone", "Bonetown", "Blasted_Steps", "Greymoor", "Shellwood"] },
    E { key: "Pilgrim Bellthrower Fly", text: "PILGRIM_BELLTHROWER_FLY", image: "Pilgrim Bellthrower Fly", kills: 10, required: true, areas: &["Blasted_Steps", "Crawl", "Greymoor", "Shellwood"] },
    E { key: "Pilgrim Hiker", text: "PILGRIM_HIKER", image: "Pilgrim Hiker", kills: 10, required: true, areas: &["Blasted_Steps"] },
    E { key: "Pilgrim StaffWielder", text: "PILGRIM_STAFFWIELDER", image: "Pilgrim StaffWeilder", kills: 15, required: true, areas: &["Greymoor", "Bone", "Bonetown", "Shellwood"] },
    E { key: "Pilgrim Moss Spitter", text: "PILGRIM_MOSS_SPITTER", image: "Pilgrim Moss Spitter", kills: 20, required: true, areas: &["Bonetown", "Crawl"] },
    E { key: "Rosary Pilgrim", text: "ROSARY_PILGRIM", image: "Rosary Pilgrim", kills: 2, required: true, areas: &["Bonetown", "Swamp", "Shellwood"] },
    E { key: "Rosary Thief", text: "ROSARY_THIEF", image: "Rosary Thief", kills: 4, required: true, areas: &["Wilds", "Bonetown", "Blasted_Steps", "Hang", "Swamp", "Slab", "Song"] },
    E { key: "Tar Slug", text: "TAR_SLUG", image: "Tar Slug", kills: 12, required: true, areas: &["Dock"] },
    E { key: "Tar Slug Huge", text: "TAR_SLUG_HUGE", image: "Tar Slug Huge", kills: 8, required: true, areas: &["Dock"] },
    E { key: "Dock Worker", text: "DOCK_WORKER", image: "Dock Worker", kills: 25, required: true, areas: &["Dock", "Bone"] },
    E { key: "Dock Flyer", text: "DOCK_FLYER", image: "Dock Flyer", kills: 25, required: true, areas: &["Dock", "Bone"] },
    E { key: "Dock Bomber", text: "DOCK_BOMBER", image: "Dock Bomber", kills: 10, required: true, areas: &["Dock"] },
    E { key: "Shield Dock Worker", text: "SHIELD_DOCK_WORKER", image: "Shield Dock Worker", kills: 10, required: true, areas: &["Dock"] },
    E { key: "Dock Charger", text: "DOCK_CHARGER", image: "Dock Charger", kills: 5, required: true, areas: &["Dock"] },
    E { key: "Dock Guard Thrower", text: "DOCK_GUARD_THROWER", image: "Dock Guard Thrower", kills: 1, required: true, areas: &[] },
    E { key: "Small Crab", text: "SMALL_CRAB", image: "Small Crab", kills: 3, required: true, areas: &["Crawl"] },
    E { key: "Roof Crab", text: "ROOF_CRAB", image: "Roof Crab", kills: 1, required: true, areas: &["Crawl"] },
    E { key: "Fields Flock Flyers", text: "FIELDS_FLOCK_FLYERS", image: "Fields Flock Flyers", kills: 15, required: true, areas: &["Wilds"] },
    E { key: "Fields Goomba", text: "FIELDS_GOOMBA", image: "Fields Goomba", kills: 25, required: true, areas: &["Wilds", "Dock"] },
    E { key: "Fields Flyer", text: "FIELDS_FLYER", image: "Fields Flyer", kills: 20, required: true, areas: &["Wilds", "Dock"] },
    E { key: "Song Golem", text: "SONG_GOLEM", image: "Song Golem", kills: 1, required: true, areas: &["Wilds"] },
    E { key: "Bone Hunter Tiny", text: "BONE_HUNTER_TINY", image: "Bone Hunter Tiny", kills: 25, required: true, areas: &["Ant", "Dock"] },
    E { key: "Bone Hunter Buzzer", text: "BONE_HUNTER_BUZZER", image: "Bone Hunter Buzzer", kills: 30, required: true, areas: &["Ant", "Dock"] },
    E { key: "Bone Hunter Child", text: "BONE_HUNTER_CHILD", image: "Bone Hunter Child", kills: 20, required: true, areas: &["Wilds", "Ant", "Dock"] },
    E { key: "Bone Hunter", text: "BONE_HUNTER", image: "Bone Hunter", kills: 15, required: true, areas: &["Wilds", "Ant", "Dock"] },
    E { key: "Bone Hunter Fly", text: "BONE_HUNTER_FLY", image: "Bone Hunter Fly", kills: 10, required: true, areas: &["Ant", "Wilds", "Dock", "Bone"] },
    E { key: "Bone Hunter Throw", text: "BONE_HUNTER_THROW", image: "Bone Hunter Throw", kills: 2, required: true, areas: &["Ant", "Wilds"] },
    E { key: "Bone Hunter Trapper", text: "BONE_HUNTER_TRAPPER", image: "Bone Hunter Trapper", kills: 1, required: true, areas: &["Wilds"] },
    E { key: "Bone Hunter Chief", text: "BONE_HUNTER_CHIEF", image: "Bone Hunter Chief", kills: 6, required: true, areas: &[] },
    E { key: "Hunter Queen", text: "HUNTER_QUEEN", image: "Hunter Queen", kills: 1, required: true, areas: &[] },
    E { key: "Mite", text: "MITE", image: "Mite", kills: 30, required: true, areas: &["Greymoor", "Dust", "Ant"] },
    E { key: "Mitefly", text: "MITEFLY", image: "Mitefly", kills: 25, required: true, areas: &["Greymoor", "Ant"] },
    E { key: "Gnat Giant", text: "GNAT_GIANT", image: "Gnat Giant", kills: 10, required: true, areas: &["Greymoor"] },
    E { key: "Farmer Catcher", text: "FARMER_CATCHER", image: "Farmer Catcher", kills: 20, required: true, areas: &["Greymoor"] },
    E { key: "Farmer Scissors", text: "FARMER_SCISSORS", image: "Farmer Scissors", kills: 25, required: true, areas: &["Greymoor"] },
    E { key: "Farmer Centipede", text: "FARMER_CENTIPEDE", image: "Farmer Centipede", kills: 15, required: true, areas: &["Greymoor"] },
    E { key: "Vampire Gnat", text: "VAMPIRE_GNAT", image: "Vampire Gnat", kills: 1, required: true, areas: &["Greymoor"] },
    E { key: "Wisp", text: "WISP", image: "Wisp", kills: 15, required: true, areas: &[] },
    E { key: "Farmer Wisp", text: "FARMER_WISP", image: "Farmer Wisp", kills: 8, required: true, areas: &["Wisp"] },
    E { key: "Wisp Pyre Effigy", text: "WISP_PYRE_EFFIGY", image: "Wisp Pyre Effigy", kills: 1, required: true, areas: &[] },
    E { key: "Crow", text: "CROW", image: "Crow", kills: 25, required: true, areas: &["Greymoor"] },
    E { key: "Crowman", text: "CROWMAN", image: "Crowman", kills: 20, required: true, areas: &["Greymoor"] },
    E { key: "Crowman Dagger", text: "CROWMAN_DAGGER", image: "Crowman Dagger", kills: 15, required: true, areas: &["Greymoor"] },
    E { key: "Crowman Juror Tiny", text: "CROWMAN_JUROR_TINY", image: "Crowman Juror Tiny", kills: 15, required: true, areas: &["Greymoor"] },
    E { key: "Crowman Juror", text: "CROWMAN_JUROR", image: "Crowman Juror", kills: 10, required: true, areas: &[] },
    E { key: "Crowman Dagger Juror", text: "CROWMAN_DAGGER_JUROR", image: "Crowman Dagger Juror", kills: 8, required: true, areas: &[] },
    E { key: "Crawfather", text: "CRAWFATHER", image: "Crawfather", kills: 1, required: true, areas: &[] },
    E { key: "Maggots", text: "MAGGOTS", image: "Maggots", kills: 80, required: true, areas: &[] },
    E { key: "Dustroach Pollywog", text: "DUSTROACH_POLLYWOG", image: "Dustroach Pollywog", kills: 12, required: true, areas: &["Swamp"] },
    E { key: "Dustroach", text: "DUSTROACH", image: "Dustroach", kills: 20, required: true, areas: &["Dust"] },
    E { key: "Bloat Roach", text: "BLOAT_ROACH", image: "Bloat Roach", kills: 12, required: true, areas: &["Swamp"] },
    E { key: "Roachfeeder Short", text: "ROACHFEEDER_SHORT", image: "Roachfeeder Short", kills: 20, required: true, areas: &["Greymoor", "Dust"] },
    E { key: "Roachfeeder Tall", text: "ROACHFEEDER_TALL", image: "Roachfeeder Tall", kills: 20, required: true, areas: &["Dust"] },
    E { key: "Roachkeeper", text: "ROACHKEEPER", image: "Roachkeeper", kills: 10, required: true, areas: &["Dust"] },
    E { key: "Roachkeeper Chef Tiny", text: "ROACHKEEPER_CHEF_TINY", image: "Roachkeeper Chef Tiny", kills: 10, required: true, areas: &["Dust"] },
    E { key: "Roachkeeper Chef", text: "ROACHKEEPER_CHEF", image: "Roachkeeper Chef", kills: 1, required: true, areas: &["Dust"] },
    E { key: "Wraith", text: "WRAITH", image: "Wraith", kills: 10, required: true, areas: &["Dust"] },
    E { key: "Swamp Drifter", text: "SWAMP_DRIFTER", image: "Swamp Drifter", kills: 25, required: true, areas: &["Swamp"] },
    E { key: "Swamp Goomba", text: "SWAMP_GOOMBA", image: "Swamp Goomba", kills: 20, required: true, areas: &["Swamp"] },
    E { key: "Swamp Mosquito", text: "SWAMP_MOSQUITO", image: "Swamp Mosquito", kills: 30, required: true, areas: &["Swamp"] },
    E { key: "Swamp Mosquito Skinny", text: "SWAMP_MOSQUITO_SKINNY", image: "Swamp Mosquito Skinny", kills: 20, required: true, areas: &["Aqueduct"] },
    E { key: "Swamp Muckman", text: "SWAMP_MUCKMAN", image: "Swamp Muckman", kills: 15, required: true, areas: &[] },
    E { key: "Swamp Muckman Tall", text: "SWAMP_MUCKMAN_TALL", image: "Swamp Muckman Tall", kills: 12, required: true, areas: &["Swamp"] },
    E { key: "Swamp Shaman", text: "SWAMP_SHAMAN", image: "Swamp Shaman", kills: 1, required: true, areas: &[] },
    E { key: "Swamp Barnacle", text: "SWAMP_BARNACLE", image: "Swamp Barnacle", kills: 8, required: true, areas: &["Aqueduct"] },
    E { key: "Swamp Ductsucker", text: "SWAMP_DUCTSUCKER", image: "Swamp Ductsucker", kills: 6, required: true, areas: &["Aqueduct"] },
    E { key: "Pond Skater", text: "POND_SKATER", image: "Pond Skater", kills: 15, required: true, areas: &["Shellwood", "Arborium"] },
    E { key: "Pilgrim Fisher", text: "PILGRIM_FISHER", image: "Pilgrim Fisher", kills: 20, required: true, areas: &["Shellwood"] },
    E { key: "Shellwood Gnat", text: "SHELLWOOD_GNAT", image: "Shellwood Gnat", kills: 15, required: true, areas: &[] },
    E { key: "Shellwood Wasp", text: "SHELLWOOD_WASP", image: "Shellwood Wasp", kills: 12, required: true, areas: &["Shellwood"] },
    E { key: "Stick Insect", text: "STICK_INSECT", image: "Stick Insect", kills: 12, required: true, areas: &["Shellwood"] },
    E { key: "Stick Insect Charger", text: "STICK_INSECT_CHARGER", image: "Stick Insect Charger", kills: 10, required: true, areas: &["Shellwood"] },
    E { key: "Stick Insect Flyer", text: "STICK_INSECT_FLYER", image: "Stick Insect Flyer", kills: 10, required: true, areas: &["Shellwood"] },
    E { key: "Splinter Queen", text: "SPLINTER_QUEEN", image: "Splinter Queen", kills: 1, required: true, areas: &["Shellwood"] },
    E { key: "Flower Drifter", text: "FLOWER_DRIFTER", image: "Flower Drifter", kills: 25, required: true, areas: &["Shellwood", "Arborium", "Song_Gate"] },
    E { key: "Bloom Shooter", text: "BLOOM_SHOOTER", image: "Bloom Shooter", kills: 10, required: true, areas: &["Arborium", "Shellwood", "Song_Gate"] },
    E { key: "Bloom Puncher", text: "BLOOM_PUNCHER", image: "Bloom Puncher", kills: 20, required: true, areas: &["Shellwood", "Arborium"] },
    E { key: "Seth", text: "SETH", image: "Seth", kills: 1, required: true, areas: &["Song_Gate"] },
    E { key: "Flower Queen", text: "FLOWER_QUEEN", image: "Flower Queen", kills: 1, required: true, areas: &["Shellwood"] },
    E { key: "Bell Goomba", text: "BELL_GOOMBA", image: "Bell Goomba", kills: 25, required: true, areas: &["Belltown", "Greymoor"] },
    E { key: "Bell Fly", text: "BELL_FLY", image: "Bell Fly", kills: 12, required: true, areas: &["Belltown"] },
    E { key: "Blade Spider", text: "BLADE_SPIDER", image: "Blade Spider", kills: 20, required: true, areas: &["Blasted_Steps", "Clover", "Coral_Caves", "Cradle", "Library", "Under", "Wisp"] },
    E { key: "Blade Spider Hang", text: "BLADE_SPIDER_HANG", image: "Blade Spider Hang", kills: 10, required: true, areas: &["Clover", "Cradle", "Library", "Under", "Weavehome", "Wisp"] },
    E { key: "Shell Fossil Mimic", text: "SHELL_FOSSIL_MIMIC", image: "Shell Fossil Mimic", kills: 3, required: true, areas: &["Bone", "Dock", "Wilds", "Coral_Caves", "Crawl", "Greymoor"] },
    E { key: "Sand Centipede", text: "SAND_CENTIPEDE", image: "Sand Centipede", kills: 40, required: true, areas: &["Coral_Caves", "Blasted_Steps"] },
    E { key: "Coral Judge Child", text: "CORAL_JUDGE_CHILD", image: "Coral Judge Child", kills: 4, required: true, areas: &["Blasted_Steps"] },
    E { key: "Coral Judge", text: "CORAL_JUDGE", image: "Coral Judge", kills: 15, required: true, areas: &["Blasted_Steps"] },
    E { key: "Last Judge", text: "LAST_JUDGE", image: "Last Judge", kills: 1, required: true, areas: &[] },
    E { key: "Coral Spike Goomba", text: "CORAL_SPIKE_GOOMBA", image: "Coral Spike Goomba", kills: 15, required: true, areas: &["Coral_Caves"] },
    E { key: "Coral Conch Shooter", text: "CORAL_CONCH_SHOOTER", image: "Coral Conch Shooter", kills: 25, required: true, areas: &["Blasted_Steps", "Arborium", "Coral_Caves"] },
    E { key: "Coral Conch Shooter Heavy", text: "CORAL_CONCH_SHOOTER_HEAVY", image: "Coral Conch Shooter Heavy", kills: 14, required: true, areas: &["Coral_Caves", "Arborium", "Blasted_Steps"] },
    E { key: "Coral Conch Stabber", text: "CORAL_CONCH_STABBER", image: "Coral Conch Stabber", kills: 16, required: true, areas: &["Coral_Caves"] },
    E { key: "Coral Conch Driller", text: "CORAL_CONCH_DRILLER", image: "Coral Conch Driller", kills: 8, required: true, areas: &["Coral_Caves"] },
    E { key: "Coral Conch Driller Giant", text: "CORAL_CONCH_DRILLER_GIANT", image: "Coral Conch Driller Giant", kills: 2, required: true, areas: &["Blasted_Steps", "Coral_Caves"] },
    E { key: "Coral Goombas", text: "CORAL_GOOMBAS", image: "Coral Goombas", kills: 10, required: true, areas: &["Coral_Caves"] },
    E { key: "Coral Goomba Large", text: "CORAL_GOOMBA_LARGE", image: "Coral Goomba Large", kills: 6, required: true, areas: &["Arborium"] },
    E { key: "Coral Swimmer Fat", text: "CORAL_SWIMMER_FAT", image: "Coral Swimmer Fat", kills: 10, required: true, areas: &["Coral_Caves"] },
    E { key: "Poke Swimmer", text: "POKE_SWIMMER", image: "Poke Swimmer", kills: 10, required: true, areas: &["Coral_Caves"] },
    E { key: "Spike Swimmer", text: "SPIKE_SWIMMER", image: "Spike Swimmer", kills: 15, required: true, areas: &["Coral_Caves"] },
    E { key: "Coral Swimmer Small", text: "CORAL_SWIMMER_SMALL", image: "Coral Swimmer Small", kills: 16, required: true, areas: &["Coral_Caves"] },
    E { key: "Coral Big Jellyfish", text: "CORAL_BIG_JELLYFISH", image: "Coral Big Jellyfish", kills: 3, required: true, areas: &["Coral_Caves"] },
    E { key: "Coral Warrior", text: "CORAL_WARRIOR", image: "Coral Warrior", kills: 10, required: true, areas: &["Coral_Caves"] },
    E { key: "Coral Flyer", text: "CORAL_FLYER", image: "Coral Flyer", kills: 10, required: true, areas: &["Coral_Caves"] },
    E { key: "Coral Flyer Throw", text: "CORAL_FLYER_THROW", image: "Coral Flyer Throw", kills: 12, required: true, areas: &["Coral_Caves"] },
    E { key: "Coral Brawler", text: "CORAL_BRAWLER", image: "Coral Brawler", kills: 8, required: true, areas: &["Coral_Caves"] },
    E { key: "Coral Hunter", text: "CORAL_HUNTER", image: "Coral Hunter", kills: 6, required: true, areas: &["Coral_Caves"] },
    E { key: "Coral Bubble Brute", text: "CORAL_BUBBLE_BRUTE", image: "Coral Bubble Brute", kills: 6, required: true, areas: &["Coral_Caves"] },
    E { key: "Coral King", text: "CORAL_KING", image: "Coral King", kills: 1, required: true, areas: &["Coral_Caves"] },
    E { key: "Coral Warrior Grey", text: "CORAL_WARRIOR_GREY", image: "Coral Warrior Grey", kills: 1, required: true, areas: &["Coral_Caves"] },
    E { key: "Zap Core Enemy", text: "ZAP_CORE_ENEMY", image: "Zap Core Enemy", kills: 1, required: true, areas: &[] },
    E { key: "Citadel Bat", text: "CITADEL_BAT", image: "Citadel Bat", kills: 30, required: true, areas: &["Library", "Hang", "Song", "Slab", "Under", "Arborium", "Song_Gate", "Ward"] },
    E { key: "Citadel Bat Large", text: "CITADEL_BAT_LARGE", image: "Citadel Bat Large", kills: 4, required: true, areas: &["Arborium", "Library"] },
    E { key: "Mite Heavy", text: "MITE_HEAVY", image: "Mite Heavy", kills: 20, required: true, areas: &["Library", "Song", "Slab", "Under", "Cog", "Hang", "Song_Gate"] },
    E { key: "Understore Mite Giant", text: "UNDERSTORE_MITE_GIANT", image: "Understore Mite Giant", kills: 6, required: true, areas: &["Hang", "Library", "Ward"] },
    E { key: "Understore Small", text: "UNDERSTORE_SMALL", image: "Understore Small", kills: 15, required: true, areas: &["Under", "Cog"] },
    E { key: "Pilgrim 03 Understore", text: "PILGRIM_03_UNDERSTORE", image: "Pilgrim 03 Understore", kills: 20, required: true, areas: &["Under"] },
    E { key: "Pilgrim Staff Understore", text: "PILGRIM_STAFF_UNDERSTORE", image: "Pilgrim Staff Understore", kills: 10, required: true, areas: &["Under"] },
    E { key: "Understore Poker", text: "UNDERSTORE_POKER", image: "Understore Poker", kills: 25, required: true, areas: &["Under", "Cog"] },
    E { key: "Understore Thrower", text: "UNDERSTORE_THROWER", image: "Understore Thrower", kills: 25, required: true, areas: &["Under", "Cog"] },
    E { key: "Understore Heavy", text: "UNDERSTORE_HEAVY", image: "Understore Heavy", kills: 4, required: true, areas: &["Under", "Cog"] },
    E { key: "Song Pilgrim 01", text: "SONG_PILGRIM_01", image: "Song Pilgrim 01", kills: 20, required: true, areas: &["Song", "Hang"] },
    E { key: "Pilgrim 01 Song", text: "PILGRIM_01_SONG", image: "Pilgrim 01 Song", kills: 30, required: true, areas: &["Song", "Library"] },
    E { key: "Pilgrim 02 Song", text: "PILGRIM_02_SONG", image: "Pilgrim 02 Song", kills: 15, required: true, areas: &["Song", "Library"] },
    E { key: "Pilgrim 03 Song", text: "PILGRIM_03_SONG", image: "Pilgrim 03 Song", kills: 25, required: true, areas: &["Song", "Library", "Hang"] },
    E { key: "Pilgrim 04 Song", text: "PILGRIM_04_SONG", image: "Pilgrim 04 Song", kills: 20, required: true, areas: &["Song", "Library"] },
    E { key: "Pilgrim Stomper Song", text: "PILGRIM_STOMPER_SONG", image: "Pilgrim Stomper Song", kills: 6, required: true, areas: &["Song", "Library"] },
    E { key: "Song Pilgrim 03", text: "SONG_PILGRIM_03", image: "Song Pilgrim 03", kills: 25, required: true, areas: &["Song", "Library", "Hang", "Slab"] },
    E { key: "Song Reed", text: "SONG_REED", image: "Song Reed", kills: 30, required: true, areas: &["Song", "Library", "Hang", "Slab"] },
    E { key: "Song Reed Grand", text: "SONG_REED_GRAND", image: "Song Reed Grand", kills: 12, required: true, areas: &["Song"] },
    E { key: "Song Heavy Sentry", text: "SONG_HEAVY_SENTRY", image: "Song Heavy Sentry", kills: 3, required: true, areas: &["Hang"] },
    E { key: "Song Handmaiden", text: "SONG_HANDMAIDEN", image: "Song Handmaiden", kills: 10, required: true, areas: &["Hang", "Song"] },
    E { key: "Arborium Keeper", text: "ARBORIUM_KEEPER", image: "Arborium Keeper", kills: 2, required: true, areas: &["Arborium"] },
    E { key: "Song Administrator", text: "SONG_ADMINISTRATOR", image: "Song Administrator", kills: 15, required: true, areas: &["Hang"] },
    E { key: "Song Pilgrim Maestro", text: "SONG_PILGRIM_MAESTRO", image: "Song Pilgrim Maestro", kills: 6, required: true, areas: &["Hang"] },
    E { key: "Song Knight", text: "SONG_KNIGHT", image: "Song Knight", kills: 1, required: true, areas: &["Song", "Hang"] },
    E { key: "Song Threaded Husk", text: "SONG_THREADED_HUSK", image: "Song Threaded Husk", kills: 8, required: true, areas: &["Ward"] },
    E { key: "Song Threaded Husk Spin", text: "SONG_THREADED_HUSK_SPIN", image: "Song Threaded Husk Spin", kills: 8, required: true, areas: &["Ward"] },
    E { key: "Song Pilgrim 02", text: "SONG_PILGRIM_02", image: "Song Pilgrim 02", kills: 3, required: true, areas: &["Ward"] },
    E { key: "Song Creeper", text: "SONG_CREEPER", image: "Song Creeper", kills: 10, required: true, areas: &["Ward"] },
    E { key: "Conductor Boss", text: "CONDUCTOR_BOSS", image: "Conductor Boss", kills: 1, required: true, areas: &[] },
    E { key: "Understore Automaton", text: "UNDERSTORE_AUTOMATON", image: "Understore Automaton", kills: 25, required: true, areas: &["Under"] },
    E { key: "Understore Automaton EX", text: "UNDERSTORE_AUTOMATON_EX", image: "Understore Automaton EX", kills: 12, required: true, areas: &["Under"] },
    E { key: "Song Automaton Goomba", text: "SONG_AUTOMATON_GOOMBA", image: "Song Automaton Goomba", kills: 15, required: true, areas: &["Cog"] },
    E { key: "Song Automaton Fly", text: "SONG_AUTOMATON_FLY", image: "Song Automaton Fly", kills: 15, required: true, areas: &["Cog", "Song"] },
    E { key: "Song Automaton Fly Spike", text: "SONG_AUTOMATON_FLY_SPIKE", image: "Song Automaton Fly Spike", kills: 10, required: true, areas: &["Cog"] },
    E { key: "Song Automaton 01", text: "SONG_AUTOMATON_01", image: "Song Automaton 01", kills: 20, required: true, areas: &["Cog"] },
    E { key: "Song Automaton 02", text: "SONG_AUTOMATON_02", image: "Song Automaton 02", kills: 12, required: true, areas: &["Cog"] },
    E { key: "Song Automaton Shield", text: "SONG_AUTOMATON_SHIELD", image: "Song Automaton Shield", kills: 8, required: true, areas: &["Cog"] },
    E { key: "Song Automaton Ball", text: "SONG_AUTOMATON_BALL", image: "Song Automaton Ball", kills: 1, required: true, areas: &["Cog"] },
    E { key: "Clockwork Dancer", text: "CLOCKWORK_DANCER", image: "Clockwork Dancer", kills: 1, required: true, areas: &[] },
    E { key: "Song Scholar Acolyte", text: "SONG_SCHOLAR_ACOLYTE", image: "Song Scholar Acolyte", kills: 20, required: true, areas: &["Library"] },
    E { key: "Lightbearer", text: "LIGHTBEARER", image: "Lightbearer", kills: 20, required: true, areas: &["Library"] },
    E { key: "Scrollkeeper", text: "SCROLLKEEPER", image: "Scrollkeeper", kills: 15, required: true, areas: &["Library"] },
    E { key: "Scholar", text: "SCHOLAR", image: "Scholar", kills: 10, required: true, areas: &["Library"] },
    E { key: "Trobbio", text: "TROBBIO", image: "Trobbio", kills: 1, required: true, areas: &[] },
    E { key: "Tormented Trobbio", text: "TORMENTED_TROBBIO", image: "Tormented Trobbio", kills: 1, required: true, areas: &[] },
    E { key: "Slab Prisoner Leaper New", text: "SLAB_PRISONER_LEAPER_NEW", image: "Slab Prisoner Leaper New", kills: 8, required: true, areas: &["Slab"] },
    E { key: "Slab Prisoner Fly New", text: "SLAB_PRISONER_FLY_NEW", image: "Slab Prisoner Fly New", kills: 6, required: true, areas: &["Slab"] },
    E { key: "Slab Fly Small Fresh", text: "SLAB_FLY_SMALL_FRESH", image: "Slab Fly Small Fresh", kills: 20, required: true, areas: &["Slab"] },
    E { key: "Slab Fly Small", text: "SLAB_FLY_SMALL", image: "Slab Fly Small", kills: 15, required: true, areas: &["Slab"] },
    E { key: "Slab Fly Mid", text: "SLAB_FLY_MID", image: "Slab Fly Mid", kills: 10, required: true, areas: &["Slab"] },
    E { key: "Slab Fly Large", text: "SLAB_FLY_LARGE", image: "Slab Fly Large", kills: 8, required: true, areas: &["Slab", "Dock", "Greymoor", "Swamp"] },
    E { key: "Slab Fly Broodmother", text: "SLAB_FLY_BROODMOTHER", image: "Slab Fly Broodmother", kills: 1, required: true, areas: &["Slab"] },
    E { key: "Peaks Drifter", text: "PEAKS_DRIFTER", image: "Peaks Drifter", kills: 20, required: true, areas: &["Peak"] },
    E { key: "Crystal Drifter", text: "CRYSTAL_DRIFTER", image: "Crystal Drifter", kills: 10, required: true, areas: &["Peak"] },
    E { key: "Crystal Drifter Giant", text: "CRYSTAL_DRIFTER_GIANT", image: "Crystal Drifter Giant", kills: 3, required: true, areas: &["Peak"] },
    E { key: "Weaver Servitor", text: "WEAVER_SERVITOR", image: "Weaver Servitor", kills: 8, required: true, areas: &["Weavehome", "Wilds", "Swamp"] },
    E { key: "Weaver Servitor Large", text: "WEAVER_SERVITOR_LARGE", image: "Weaver Servitor Large", kills: 5, required: true, areas: &["Peak"] },
    E { key: "Lifeblood Fly", text: "LIFEBLOOD_FLY", image: "Lifeblood_fly", kills: 8, required: true, areas: &[] },
    E { key: "Bone Worm BlueBlood", text: "BONE_WORM_BLUEBLOOD", image: "Bone Worm BlueBlood", kills: 15, required: true, areas: &["Crawl"] },
    E { key: "Bone Worm BlueTurret", text: "BONE_WORM_BLUETURRET", image: "Bone Worm BlueTurret", kills: 6, required: true, areas: &["Crawl"] },
    E { key: "Blue Assistant", text: "BLUE_ASSISTANT", image: "Blue Assistant", kills: 1, required: true, areas: &[] },
    E { key: "Lilypad Fly", text: "LILYPAD_FLY", image: "Lilypad Fly", kills: 12, required: true, areas: &["Clover"] },
    E { key: "Grass Goomba", text: "GRASS_GOOMBA", image: "Grass Goomba", kills: 10, required: true, areas: &["Clover"] },
    E { key: "Hornet Dragonfly", text: "HORNET_DRAGONFLY", image: "Hornet Dragonfly", kills: 10, required: true, areas: &["Clover"] },
    E { key: "Dragonfly Large", text: "DRAGONFLY_LARGE", image: "Dragonfly Large", kills: 10, required: true, areas: &["Clover"] },
    E { key: "Lilypad Trap", text: "LILYPAD_TRAP", image: "Lilypad Trap", kills: 6, required: true, areas: &["Clover", "Arborium"] },
    E { key: "Cloverstag", text: "CLOVERSTAG", image: "Cloverstag", kills: 10, required: true, areas: &["Clover"] },
    E { key: "Cloverstag White", text: "CLOVERSTAG_WHITE", image: "Cloverstag White", kills: 1, required: false, areas: &["Clover"] },
    E { key: "Grasshopper Child", text: "GRASSHOPPER_CHILD", image: "Grasshopper Child", kills: 10, required: true, areas: &["Clover"] },
    E { key: "Grasshopper Slasher", text: "GRASSHOPPER_SLASHER", image: "Grasshopper Slasher", kills: 10, required: true, areas: &["Clover"] },
    E { key: "Grasshopper Fly", text: "GRASSHOPPER_FLY", image: "Grasshopper Fly", kills: 10, required: true, areas: &["Clover"] },
    E { key: "Clover Dancer", text: "CLOVER_DANCER", image: "Clover Dancer", kills: 1, required: true, areas: &["Clover"] },
    E { key: "Abyss Crawler", text: "ABYSS_CRAWLER", image: "Abyss Crawler", kills: 16, required: true, areas: &["Abyss"] },
    E { key: "Abyss Crawler Large", text: "ABYSS_CRAWLER_LARGE", image: "Abyss Crawler Large", kills: 6, required: true, areas: &["Abyss"] },
    E { key: "Gloomfly", text: "GLOOMFLY", image: "Gloomfly", kills: 15, required: true, areas: &[] },
    E { key: "Gloom Beast", text: "GLOOM_BEAST", image: "Gloom Beast", kills: 4, required: true, areas: &["Abyss"] },
    E { key: "Void Tendrils", text: "ABYSS_TENDRIL", image: "Abyss_Tendrils", kills: 1, required: true, areas: &[] },
    E { key: "Black Thread Core", text: "BLACK_THREAD_CORE", image: "Black Thread Core", kills: 8, required: true, areas: &[] },
    E { key: "Abyss Mass", text: "ABYSS_MASS", image: "Abyss Mass", kills: 1, required: false, areas: &["Bonetown"] },
    E { key: "White Palace Fly", text: "WHITE_PALACE_FLY", image: "White Palace Fly", kills: 10, required: true, areas: &[] },
    E { key: "Centipede Trap", text: "CENTIPEDE_TRAP", image: "Centipede_trap", kills: 30, required: true, areas: &[] },
    E { key: "Spike Lazy Flyer", text: "SPIKE_LAZY_FLYER", image: "Spike Lazy Flyer", kills: 4, required: true, areas: &["Cradle"] },
    E { key: "Surface Scuttler", text: "SURFACE_SCUTTLER", image: "Surface Scuttler", kills: 10, required: true, areas: &[] },
    E { key: "Giant Centipede", text: "GIANT_CENTIPEDE", image: "Giant Centipede", kills: 1, required: true, areas: &[] },
    E { key: "Giant Flea", text: "GIANT_FLEA", image: "Giant Flea", kills: 1, required: true, areas: &[] },
    E { key: "Shakra", text: "SHAKRA", image: "Shakra", kills: 1, required: false, areas: &[] },
    E { key: "Garmond_Zaza", text: "GARMOND_ZAZA", image: "Garmond_Zaza", kills: 1, required: false, areas: &[] },
    E { key: "Garmond", text: "GARMOND", image: "Garmond", kills: 1, required: false, areas: &["Blasted_Steps"] },
    E { key: "Pinstress Boss", text: "PINSTRESS_BOSS", image: "Pinstress Boss", kills: 1, required: true, areas: &["Peak"] },
    E { key: "Spinner Boss", text: "SPINNER_BOSS", image: "Spinner Boss", kills: 1, required: true, areas: &[] },
    E { key: "First Weaver", text: "FIRST_WEAVER", image: "First Weaver", kills: 1, required: true, areas: &[] },
    E { key: "Phantom", text: "PHANTOM", image: "Phantom", kills: 1, required: true, areas: &[] },
    E { key: "Lace", text: "LACE", image: "Lace", kills: 2, required: true, areas: &["Dock", "Cradle"] },
    E { key: "Silk Boss", text: "SILK_BOSS", image: "Silk Boss", kills: 1, required: true, areas: &[] },
    E { key: "Lost Lace", text: "LOST_LACE", image: "Lost_Lace", kills: 1, required: false, areas: &[] },
];

/// The game's words for an entry, as `crate::Words` reads them.
pub fn items(pd: &Map<String, Value>, words: &crate::Words, fr: bool) -> Vec<Item> {
    let kills: HashMap<&str, u32> = pd["EnemyJournalKillData"]["list"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|e| Some((e["Name"].as_str()?, e["Record"]["Kills"].as_u64().unwrap_or(0) as u32)))
        .collect();
    ENTRIES
        .iter()
        .map(|e| {
            let name = words.get(&format!("NAME_{}", e.text)).unwrap_or_else(|| e.key.to_string());
            let met = kills.get(e.key).copied();
            let mut item = Item::count(format!("journal-{}", e.key), name, met.unwrap_or(0), e.kills)
                .image(format!("game:{}", e.image))
                .areas(e.areas.iter().map(|a| words.zone(a)));
            // Met but not yet killed is still a start.
            if met.is_some() && item.status == Status::Todo {
                item.status = Status::InProgress;
            }
            // An entry not yet met is a creature the reader has not seen: the
            // game itself keeps it hidden.
            if met.is_none() {
                item = item.spoiler(Spoiler::Major);
            }
            if !e.required {
                item = item.hint(if fr { "Ne compte pas pour compléter le journal" } else { "Not needed to complete the journal" });
            }
            // What the journal shows: the description, and the notes once complete.
            let mut details: Vec<String> = Vec::new();
            if let Some(d) = words.get(&format!("DESC_{}", e.text)) {
                details.push(d);
            }
            if item.status == Status::Done
                && let Some(n) = words.get(&format!("NOTE_{}", e.text))
            {
                details.push(n);
            }
            if !details.is_empty() {
                item = item.details(details.join("\n\n"));
            }
            item
        })
        .collect()
}
