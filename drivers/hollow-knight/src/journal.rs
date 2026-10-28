//! The Hunter's Journal: every creature the Hunter asks the Knight to study.
//!
//! The save keeps, for each entry, `killedX` (met and killed once) and
//! `killsX` (kills still needed before the Hunter's notes unlock; 0 when
//! complete). Names, descriptions and notes come from the game's own language
//! sheet when the host could read it from the installed game, portraits from
//! its sprites.

use hundo_sdk::{Item, Spoiler, Status};

use crate::Analysis;

/// One entry, identifiers only, taken from the game's files (1.5): its
/// journal entry (save key, text key, portrait sprite) and the map areas of
/// the scenes whose enemies count towards it (their death effects name the
/// entry), the busiest first. Bosses the game spawns by script have their
/// area set by hand.
struct E {
    key: &'static str,
    text: &'static str,
    image: &'static str,
    dlc: Option<&'static str>,
    areas: &'static [&'static str],
}

/// In the game's order.
const ENTRIES: &[E] = &[
    E { key: "Crawler", text: "CRAWLER", image: "bestiary_crawler_f", dlc: None, areas: &["White Palace", "Crossroads", "Cliffs", "Waterways", "Resting_Grounds", "Green_Path", "Town_Tutorial"] },
    E { key: "Buzzer", text: "BUZZER", image: "bestiary_buzzer_f", dlc: None, areas: &["City of Tears", "Crossroads", "Cliffs", "Resting_Grounds", "Green_Path", "Town_Tutorial"] },
    E { key: "Bouncer", text: "BOUNCER", image: "bestiary_bouncer_f", dlc: None, areas: &["Crossroads"] },
    E { key: "Climber", text: "CLIMBER", image: "bestiary_climber_f", dlc: None, areas: &["Crossroads", "Cliffs", "Resting_Grounds", "Green_Path"] },
    E { key: "Hopper", text: "HOPPER", image: "bestiary_hopper_f", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "Worm", text: "WORM", image: "bestiary_armoured-worm_f", dlc: None, areas: &["Crossroads", "Deepnest"] },
    E { key: "Spitter", text: "SPITTER", image: "bestiary__0009_spitter", dlc: None, areas: &["Crossroads", "Ancient Basin"] },
    E { key: "Hatcher", text: "HATCHER", image: "bestiary_hatcher_f", dlc: None, areas: &["Crossroads"] },
    E { key: "Hatchling", text: "HATCHLING", image: "bestiary_hatcher_baby_f", dlc: None, areas: &["Crossroads"] },
    E { key: "ZombieRunner", text: "ZOM_RUNNER", image: "bestiary_bug_f", dlc: None, areas: &["Crossroads", "Deepnest", "Green_Path", "City of Tears"] },
    E { key: "ZombieHornhead", text: "ZOM_HORNHEAD", image: "bestiary_hornhead_f", dlc: None, areas: &["Crossroads", "City of Tears", "Deepnest", "Cliffs", "Green_Path", "Waterways", "Fungal Wastes"] },
    E { key: "ZombieLeaper", text: "ZOM_LEAPER", image: "bestiary_leaper_f", dlc: None, areas: &["Crossroads", "City of Tears", "Cliffs", "Green_Path"] },
    E { key: "ZombieBarger", text: "ZOM_BARGER", image: "bestiary_barger_f", dlc: None, areas: &["Crossroads", "City of Tears", "Cliffs", "Waterways", "Green_Path"] },
    E { key: "ZombieShield", text: "ZOM_SHIELD", image: "bestiary_shell-shielder_f", dlc: None, areas: &["Crossroads"] },
    E { key: "ZombieGuard", text: "ZOM_GUARD", image: "bestiary_crossroads-guard_f", dlc: None, areas: &["Crossroads"] },
    E { key: "BigBuzzer", text: "BIGBUZZER", image: "bestiary_big-buzzer_f", dlc: None, areas: &["Green_Path", "Kingdoms_Edge"] },
    E { key: "BigFly", text: "BIGFLY", image: "bestiary_big-bouncer_f", dlc: None, areas: &["Crossroads"] },
    E { key: "Mawlek", text: "MAWLEK", image: "bestiary_mawlek_f", dlc: None, areas: &["Crossroads"] },
    E { key: "FalseKnight", text: "FALSEKNIGHT", image: "bestiary_false-knight_f", dlc: None, areas: &["Crossroads"] },
    E { key: "Roller", text: "ROLLER", image: "bestiary_roller_f", dlc: None, areas: &[] },
    E { key: "Blocker", text: "BLOCKER", image: "bestiary_blocker_f", dlc: None, areas: &["Crossroads", "Cliffs"] },
    E { key: "PrayerSlug", text: "PRAYER_SLUG", image: "bestiary_prayer_slug_f", dlc: None, areas: &["Crossroads"] },
    E { key: "MenderBug", text: "MENDERBUG", image: "bestiary_menderbug_f", dlc: None, areas: &["Crossroads"] },
    E { key: "MossmanRunner", text: "MOSSMAN_RUNNER", image: "bestiary_mossman_runner_f", dlc: None, areas: &["Green_Path"] },
    E { key: "MossmanShaker", text: "MOSSMAN_SHAKER", image: "bestiary_mossman_shaker_f", dlc: None, areas: &["Green_Path"] },
    E { key: "Mosquito", text: "MOSQUITO", image: "bestiary_mosquito_f", dlc: None, areas: &["Green_Path"] },
    E { key: "BlobFlyer", text: "BLOB_FLYER", image: "bestiary_blob_fly_f", dlc: None, areas: &["Green_Path"] },
    E { key: "FungifiedZombie", text: "FUNG_ZOMBIE", image: "bestiary_fungus_zombie_02_f", dlc: None, areas: &["Fungal Wastes"] },
    E { key: "PlantShooter", text: "PLANT_SHOOTER", image: "bestiary_spit_turret_f", dlc: None, areas: &["Green_Path"] },
    E { key: "MossCharger", text: "MOSS_CHARGER", image: "bestiary_moss_charger_f", dlc: None, areas: &["Green_Path"] },
    E { key: "MegaMossCharger", text: "MEGA_MOSS_CHARGER", image: "bestiary_mega_moss_charger_f", dlc: None, areas: &["Green_Path"] },
    E { key: "SnapperTrap", text: "SNAPPER_TRAP", image: "bestiary_snap_turret_f", dlc: None, areas: &["Green_Path", "Queens_Gardens"] },
    E { key: "MossKnight", text: "MOSS_KNIGHT", image: "bestiary_moss_knight_f", dlc: None, areas: &["Green_Path"] },
    E { key: "GrassHopper", text: "GRASS_HOPPER", image: "bestiary__0029_crazy-hopper", dlc: None, areas: &["Queens_Gardens"] },
    E { key: "AcidFlyer", text: "ACID_FLYER", image: "bestiary__0039_acid-flyer", dlc: None, areas: &["Green_Path"] },
    E { key: "AcidWalker", text: "ACID_WALKER", image: "bestiary__0037_acid-walker", dlc: None, areas: &["Green_Path", "Queens_Gardens"] },
    E { key: "MossFlyer", text: "MOSS_FLYER", image: "bestiary__0017_green-flyer", dlc: None, areas: &["Queens_Gardens", "Fog_Canyon"] },
    E { key: "MossKnightFat", text: "MOSS_KNIGHT_FAT", image: "bestiary_moss_knight_fat_f", dlc: None, areas: &["Queens_Gardens"] },
    E { key: "MossWalker", text: "MOSS_WALKER", image: "bestiary__0041_moss-crawler", dlc: None, areas: &["Green_Path", "Queens_Gardens"] },
    E { key: "InfectedKnight", text: "INFECTED_KNIGHT", image: "bestiary_infected_knight_f", dlc: None, areas: &["Ancient Basin"] },
    E { key: "LazyFlyer", text: "LAZY_FLYER", image: "bestiary_fung_lazy_flyer_f", dlc: None, areas: &["Green_Path", "Queens_Gardens"] },
    E { key: "ZapBug", text: "ZAP_BUG", image: "bestiary_zap_bugs_f", dlc: None, areas: &["Fog_Canyon"] },
    E { key: "Jellyfish", text: "JELLYFISH", image: "bestiary_jellyfish_f", dlc: None, areas: &["Fog_Canyon"] },
    E { key: "JellyCrawler", text: "JELLY_CRAWLER", image: "bestiary__0025_jelly-mini", dlc: None, areas: &["Fog_Canyon"] },
    E { key: "MegaJellyfish", text: "MEGA_JELLYFISH", image: "bestiary__0073_giant-jellyfish", dlc: None, areas: &["Godhome"] },
    E { key: "FungoonBaby", text: "FUNGOON_BABY", image: "bestiary__0033_fungoon-mini", dlc: None, areas: &["Fungal Wastes"] },
    E { key: "MushroomTurret", text: "MUSH_TURRET", image: "bestiary_mushroom_turret_f", dlc: None, areas: &["Fungal Wastes"] },
    E { key: "Mantis", text: "MANTIS", image: "bestiary_mantis_f", dlc: None, areas: &["Fungal Wastes"] },
    E { key: "MushroomRoller", text: "MUSH_ROLLER", image: "bestiary_mushroom_roller_f", dlc: None, areas: &["Fungal Wastes"] },
    E { key: "MushroomBrawler", text: "MUSH_BRAWLER", image: "bestiary_mushroom_brawler_f", dlc: None, areas: &["Fungal Wastes"] },
    E { key: "MushroomBaby", text: "MUSH_BABY", image: "bestiary_mushroom_mini_f", dlc: None, areas: &["Fungal Wastes"] },
    E { key: "MantisFlyerChild", text: "MANTIS_FLYER_CHILD", image: "bestiary_mantis_flyer_f", dlc: None, areas: &["Fungal Wastes"] },
    E { key: "FungusFlyer", text: "FUNGOON", image: "bestiary__0031_fungoon", dlc: None, areas: &["Fungal Wastes"] },
    E { key: "FungCrawler", text: "FUNG_CRAWLER", image: "bestiary__0015_fung-crawler", dlc: None, areas: &["Fungal Wastes"] },
    E { key: "MantisLord", text: "MANTIS_LORD", image: "bestiary_mantis_lords_f", dlc: None, areas: &["Fungal Wastes"] },
    E { key: "BlackKnight", text: "BLACK_KNIGHT", image: "bestiary_black_knight_f", dlc: None, areas: &["City of Tears"] },
    E { key: "ElectricMage", text: "ELEC_MAGE", image: "bestiary_electric_mage_f", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "Mage", text: "MAGE", image: "bestiary_mage_f", dlc: None, areas: &["City of Tears", "Kingdoms_Edge"] },
    E { key: "MageKnight", text: "MAGE_KNIGHT", image: "bestiary_mage_knight_f", dlc: None, areas: &["City of Tears", "Kingdoms_Edge"] },
    E { key: "RoyalDandy", text: "ZOM_ROYAL_1", image: "bestiary_royal_zombie_point_f", dlc: None, areas: &["City of Tears"] },
    E { key: "RoyalCoward", text: "ZOM_ROYAL_2", image: "bestiary_royal_zombie_flat_f", dlc: None, areas: &["City of Tears"] },
    E { key: "RoyalPlumper", text: "ZOM_ROYAL_FAT", image: "bestiary_royal_zombie_fat_f", dlc: None, areas: &["City of Tears"] },
    E { key: "FlyingSentrySword", text: "SENTRY_FLYING", image: "bestiary_ruins_zombie_flier_f", dlc: None, areas: &["City of Tears", "Waterways", "Resting_Grounds"] },
    E { key: "FlyingSentryJavelin", text: "SENTRY_FLYING_JAVELIN", image: "bestiary_ruins_zombie_lance_f", dlc: None, areas: &["City of Tears", "Waterways"] },
    E { key: "Sentry", text: "SENTRY_1", image: "bestiary_ruins_zombie_f", dlc: None, areas: &["City of Tears", "Waterways", "Resting_Grounds"] },
    E { key: "SentryFat", text: "SENTRY_FAT", image: "bestiary_ruins_zombie_big_f", dlc: None, areas: &["City of Tears", "Waterways"] },
    E { key: "MageBlob", text: "MAGEBLOB", image: "bestiary__0023_mage-blob", dlc: None, areas: &["City of Tears", "Kingdoms_Edge"] },
    E { key: "GreatShieldZombie", text: "GREAT_SHIELD_ZOMBIE", image: "bestiary__ruin_shield_guard_f", dlc: None, areas: &["City of Tears", "Resting_Grounds"] },
    E { key: "JarCollector", text: "JAR_COLLECTOR", image: "bestiary__0069_collector", dlc: None, areas: &["City of Tears"] },
    E { key: "MageBalloon", text: "MAGE_BALLOON", image: "bestiary__0021_mage-balloon", dlc: None, areas: &["City of Tears", "Kingdoms_Edge"] },
    E { key: "MageLord", text: "MAGE_LORD", image: "bestiary_mage_lord_f", dlc: None, areas: &["City of Tears"] },
    E { key: "GorgeousHusk", text: "GORGEOUS_HUSK", image: "bestiary_gold_royal_f", dlc: None, areas: &["City of Tears"] },
    E { key: "FlipHopper", text: "FLIP_HOPPER", image: "bestiary_flipper_f", dlc: None, areas: &["Waterways"] },
    E { key: "Flukeman", text: "FLUKEMAN", image: "bestiary_flukeman_f", dlc: None, areas: &["Waterways"] },
    E { key: "Inflater", text: "INFLATOR", image: "bestiary_inflater_f", dlc: None, areas: &["Waterways"] },
    E { key: "Flukefly", text: "FLUKEFLY", image: "bestiary__0055_fluke-fairy", dlc: None, areas: &["Waterways"] },
    E { key: "FlukeMother", text: "FLUKE_MOTHER", image: "bestiary__0057_fluke-mother", dlc: None, areas: &["Waterways"] },
    E { key: "DungDefender", text: "DUNG_DEFENDER", image: "bestiary__0005_dung-defender", dlc: None, areas: &["Waterways"] },
    E { key: "CrystalCrawler", text: "CRYS_CRAWLER", image: "bestiary_mines_goomba_f", dlc: None, areas: &["Crystal Peak", "Crossroads"] },
    E { key: "CrystalFlyer", text: "CRYS_FLYER", image: "bestiary_mines_flier_f", dlc: None, areas: &["Crystal Peak"] },
    E { key: "LaserBug", text: "CRYS_LAZER_BUG", image: "bestiary_mines_climber_f", dlc: None, areas: &["Crystal Peak"] },
    E { key: "BeamMiner", text: "ZOM_BEAM_MINER", image: "bestiary_zombie_miner_crystal_f", dlc: None, areas: &["Crystal Peak"] },
    E { key: "ZombieMiner", text: "ZOM_MINER", image: "bestiary_zombie_miner_f", dlc: None, areas: &["Crystal Peak", "Crossroads"] },
    E { key: "MegaBeamMiner", text: "MEGA_BEAM_MINER", image: "bestiary__0065_mega-beam-miner", dlc: None, areas: &["Crystal Peak"] },
    E { key: "MinesCrawler", text: "MINES_CRAWLER", image: "bestiary__0019_mines-crawler", dlc: None, areas: &["Crystal Peak"] },
    E { key: "AngryBuzzer", text: "ANGRY_BUZZER", image: "bestiary_infected_buzzer_f", dlc: None, areas: &["Crossroads"] },
    E { key: "BurstingBouncer", text: "BURSTING_BOUNCER", image: "bestiary_infected_fly_f", dlc: None, areas: &["Crossroads"] },
    E { key: "BurstingZombie", text: "ZOM_BURSTING", image: "bestiary_infected_husk1_f", dlc: None, areas: &["Crossroads"] },
    E { key: "SpittingZombie", text: "ZOM_SPITTING", image: "bestiary_infected_husk2_f", dlc: None, areas: &["Crossroads"] },
    E { key: "BabyCentipede", text: "BABY_CENTIPEDE", image: "bestiary_centipede_f", dlc: None, areas: &["Deepnest"] },
    E { key: "BigCentipede", text: "BIG_CENTIPEDE", image: "bestiary_big_centipede_f", dlc: None, areas: &["Deepnest"] },
    E { key: "CentipedeHatcher", text: "CENTIPEDE_HATCHER", image: "bestiary_centipede_hatcher_f", dlc: None, areas: &["Deepnest"] },
    E { key: "LesserMawlek", text: "LESSER_MAWLEK", image: "bestiary_mini_mawlek_f", dlc: None, areas: &["Ancient Basin"] },
    E { key: "SlashSpider", text: "SLASH_SPIDER", image: "bestiary_spider_soldier_f", dlc: None, areas: &["Deepnest"] },
    E { key: "SpiderCorpse", text: "ZOM_SPIDER", image: "bestiary_zombie_spider_01_f", dlc: None, areas: &["Deepnest"] },
    E { key: "ShootSpider", text: "SPITTING_SPIDER", image: "bestiary__0051_spit-spider", dlc: None, areas: &["Deepnest"] },
    E { key: "MiniSpider", text: "MINI_SPIDER", image: "bestiary_tiny_spider_f", dlc: None, areas: &["Deepnest"] },
    E { key: "SpiderFlyer", text: "SPIDER_FLYER", image: "bestiary__0011_spider-flyer", dlc: None, areas: &["Deepnest"] },
    E { key: "MimicSpider", text: "MIMIC_SPIDER", image: "bestiary__0059_mimic-spider", dlc: None, areas: &["Deepnest"] },
    E { key: "BeeHatchling", text: "BEE_HATCHLING", image: "bestiary_bee_tiny", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "BeeStinger", text: "BEE", image: "bestiary_bee_f", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "BigBee", text: "BIG_BEE", image: "bestiary_bee_big_f", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "HiveKnight", text: "HIVE_KNIGHT", image: "bestiary_hive_knight_f", dlc: Some("lifeblood"), areas: &["Kingdoms_Edge"] },
    E { key: "BlowFly", text: "BIG_FAT_FLYER", image: "bestiary__0053_blow-fly", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "CeilingDropper", text: "CEILING_DROPPER", image: "bestiary_bat_dropper_f", dlc: None, areas: &["Waterways", "City of Tears", "Kingdoms_Edge", "Ancient Basin", "Resting_Grounds"] },
    E { key: "GiantHopper", text: "GIANT_HOPPER", image: "bestiary__0071_giant-hopper", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "GrubMimic", text: "GRUB_MIMIC", image: "bestiary_mimic_grub_f", dlc: None, areas: &["Deepnest", "Crystal Peak"] },
    E { key: "MawlekTurret", text: "MAWLEK_TURRET", image: "bestiary_mawlek_turret_f", dlc: None, areas: &["Ancient Basin"] },
    E { key: "OrangeScuttler", text: "PARASITE_BLOB", image: "bestiary__0013_orange-blob", dlc: None, areas: &[] },
    E { key: "HealthScuttler", text: "HEALTH_SCUTTLER", image: "bestiary_blue_blob_f", dlc: None, areas: &[] },
    E { key: "Pigeon", text: "PIGEON", image: "bestiary_fung_pigeon_flyer_f", dlc: None, areas: &["Green_Path", "Queens_Gardens"] },
    E { key: "ZombieHive", text: "ZOM_HIVE", image: "bestiary_zombie_hive_f", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "Hornet", text: "HORNET", image: "bestiary_hornet", dlc: None, areas: &["Town_Tutorial", "Kingdoms_Edge", "Green_Path"] },
    E { key: "AbyssCrawler", text: "ABYSS_CRAWLER", image: "bestiary__0047_abyss-crawler", dlc: None, areas: &["Ancient Basin"] },
    E { key: "SuperSpitter", text: "SUPER_SPITTER", image: "bestiary_super_spitter_f", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "Sibling", text: "SIBLING", image: "bestiary__0045_shade-sibling", dlc: None, areas: &["Ancient Basin"] },
    E { key: "PalaceFly", text: "PALACE_FLY", image: "bestiary__0043_white-palace-fly", dlc: None, areas: &["White Palace"] },
    E { key: "EggSac", text: "EGG_SAC", image: "bestiary_egg_sac", dlc: None, areas: &["Waterways", "Crystal Peak", "Resting_Grounds", "Deepnest", "Kingdoms_Edge", "Queens_Gardens"] },
    E { key: "Mummy", text: "MUMMY", image: "bestiary__0007_grave-zombie", dlc: None, areas: &["Resting_Grounds"] },
    E { key: "OrangeBalloon", text: "ORANGE_BALLOON", image: "bestiary_parasite_flyer_f", dlc: None, areas: &["Ancient Basin"] },
    E { key: "AbyssTendril", text: "ABYSS_TENDRIL", image: "bestiary_abyss_tendrils_f", dlc: None, areas: &["Ancient Basin"] },
    E { key: "HeavyMantis", text: "MANTIS_HEAVY", image: "bestiary_heavy_mantis_f", dlc: None, areas: &["Queens_Gardens"] },
    E { key: "TraitorLord", text: "TRAITOR_LORD", image: "bestiary__0061_traitor-lord", dlc: None, areas: &["Queens_Gardens"] },
    E { key: "MantisHeavyFlyer", text: "MANTIS_FLYER_ADULT", image: "bestiary_heavy_mantis_flyer_f", dlc: None, areas: &["Queens_Gardens"] },
    E { key: "GardenZombie", text: "ZOM_SPINE", image: "bestiary_fungus_zombie_03_f", dlc: None, areas: &["Queens_Gardens"] },
    E { key: "RoyalGuard", text: "ROYAL_GUARD", image: "bestiary_kingsguard_f", dlc: None, areas: &["White Palace"] },
    E { key: "WhiteRoyal", text: "WHITE_ROYAL", image: "bestiary_white_servant_f", dlc: None, areas: &["White Palace"] },
    E { key: "Oblobble", text: "OBLOBBLE", image: "bestiary__0063_giant-fat-bee", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "Zote", text: "ZOTE", image: "bestiary__0049_zote", dlc: None, areas: &["Godhome"] },
    E { key: "Blobble", text: "BLOBBLE", image: "bestiary__0035_blobble", dlc: None, areas: &[] },
    E { key: "ColMosquito", text: "COL_MOSQUITO", image: "bestiary_colosseum_armoured_mosquito_f", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "ColRoller", text: "COL_ROLLER", image: "bestiary_colosseum_armoured_roller_f", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "ColFlyingSentry", text: "COL_FLYINGSENTRY", image: "bestiary_colosseum_flying_gladiator_f", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "ColMiner", text: "COL_MINER", image: "bestiary_colosseum_miner", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "ColShield", text: "COL_SHIELD", image: "bestiary_colosseum_shield_gladiator_f", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "ColWorm", text: "COL_WORM", image: "bestiary_colosseum_worm_f", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "ColHopper", text: "COL_HOPPER", image: "bestiary_colosseum_hopper_f", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "LobsterLancer", text: "LOBSTER_LANCER", image: "bestiary__0067_lobster-lancer", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "GhostAladar", text: "GHOST_ALADAR", image: "bestiary_ghosts__0000_brain-slug", dlc: None, areas: &["Cliffs"] },
    E { key: "GhostXero", text: "GHOST_XERO", image: "bestiary_ghosts__0001_xero", dlc: None, areas: &["Resting_Grounds"] },
    E { key: "GhostHu", text: "GHOST_HU", image: "bestiary_ghosts__0002_elderhu", dlc: None, areas: &["Fungal Wastes"] },
    E { key: "GhostMarmu", text: "GHOST_MARMU", image: "bestiary_ghosts__0005_marmu", dlc: None, areas: &["Queens_Gardens"] },
    E { key: "GhostNoEyes", text: "GHOST_NOEYES", image: "bestiary_ghosts__0006_no-eyes", dlc: None, areas: &["Godhome"] },
    E { key: "GhostMarkoth", text: "GHOST_MARKOTH", image: "bestiary_ghosts__0004_markoth", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "GhostGalien", text: "GHOST_GALIEN", image: "bestiary_ghosts__0003_galien", dlc: None, areas: &["Deepnest"] },
    E { key: "WhiteDefender", text: "WHITE_DEFENDER", image: "bestiary_White_Defender_full", dlc: Some("hidden-dreams"), areas: &["Godhome"] },
    E { key: "GreyPrince", text: "GREY_PRINCE", image: "bestiary_mighty_zote_full", dlc: Some("hidden-dreams"), areas: &["Godhome"] },
    E { key: "ZotelingBalloon", text: "ZOTE_BALLOON", image: "bestiary_mighty_zote_balloon_full", dlc: Some("hidden-dreams"), areas: &[] },
    E { key: "ZotelingHopper", text: "ZOTE_HOPPER", image: "bestiary_mighty_zote_hopper_full", dlc: Some("hidden-dreams"), areas: &[] },
    E { key: "ZotelingBuzzer", text: "ZOTE_FLYER", image: "bestiary_mighty_zoteling_full", dlc: Some("hidden-dreams"), areas: &[] },
    E { key: "HollowKnight", text: "HOLLOW_KNIGHT", image: "bestiary__0003_hollow-knight", dlc: None, areas: &["Crossroads"] },
    E { key: "FinalBoss", text: "FINAL_BOSS", image: "bestiary__0001_radiant_moth", dlc: None, areas: &["Crossroads"] },
    E { key: "HunterMark", text: "HUNTERMARK", image: "bestiary_hunter_mark_f", dlc: None, areas: &[] },
    E { key: "FlameBearerSmall", text: "FLAMEBEARER_SMALL", image: "Bestiary_Grimmkin_little", dlc: Some("grimm-troupe"), areas: &[] },
    E { key: "FlameBearerMed", text: "FLAMEBEARER_MED", image: "Bestiary_Grimmkin_fat", dlc: Some("grimm-troupe"), areas: &[] },
    E { key: "FlameBearerLarge", text: "FLAMEBEARER_LARGE", image: "Bestiary_Grimmkin_tall", dlc: Some("grimm-troupe"), areas: &[] },
    E { key: "Grimm", text: "GRIMM", image: "Bestiary_Grimm_boss", dlc: Some("grimm-troupe"), areas: &["Town_Tutorial"] },
    E { key: "NightmareGrimm", text: "NIGHTMARE_GRIMM", image: "Bestiary_Grimm_Nightmare_boss", dlc: Some("grimm-troupe"), areas: &["Town_Tutorial"] },
    E { key: "BindingSeal", text: "KIN_SEAL", image: "vessel_seal", dlc: None, areas: &["Godhome"] },
    E { key: "FatFluke", text: "FAT_FLUKE", image: "bestiary_fat_fluke_full", dlc: None, areas: &["Waterways"] },
    E { key: "PaleLurker", text: "PALE_LURKER", image: "bestiary_gg__0009_pale_runaway", dlc: None, areas: &["Kingdoms_Edge"] },
    E { key: "NailBros", text: "NAILMASTERS", image: "bestiary_gg__0006_nailmasters", dlc: Some("godmaster"), areas: &["Godhome"] },
    E { key: "Paintmaster", text: "PAINTMASTER", image: "bestiary_gg__0007_paintmaster", dlc: Some("godmaster"), areas: &["Godhome"] },
    E { key: "Nailsage", text: "SAGE_SLY", image: "bestiary_gg__0005_sage", dlc: Some("godmaster"), areas: &["Godhome"] },
    E { key: "HollowKnightPrime", text: "PURE_VESSEL", image: "bestiary_gg__0008_HK_prime", dlc: Some("godmaster"), areas: &["Godhome"] },
    E { key: "GodseekerMask", text: "GODSEEKER_MASK", image: "bestiary_gg_mask", dlc: Some("godmaster"), areas: &["Godhome"] },
    E { key: "VoidIdol_1", text: "VOID_IDOL_1", image: "bestiary_gg_void_idol", dlc: Some("godmaster"), areas: &["Godhome"] },
    E { key: "VoidIdol_2", text: "VOID_IDOL_2", image: "bestiary_gg_void_idol", dlc: Some("godmaster"), areas: &["Godhome"] },
    E { key: "VoidIdol_3", text: "VOID_IDOL_3", image: "bestiary_gg_void_idol", dlc: Some("godmaster"), areas: &["Godhome"] },
];

impl Analysis<'_> {
    pub(crate) fn journal(&self) -> Vec<Item> {
        ENTRIES
            .iter()
            .filter_map(|e| {
                let killed = self.bool(&format!("killed{}", e.key))?;
                let left = self.int(&format!("kills{}", e.key));
                let name = self.text(&format!("NAME_{}", e.text)).unwrap_or_else(|| spaced(e.key));
                let status = match (killed, left) {
                    (true, n) if n <= 0 || !self.full_journal => Status::Done,
                    (true, _) => Status::InProgress,
                    _ => Status::Todo,
                };
                let mut item = Item::new(format!("journal-{}", e.key), name, status)
                    .image(format!("game:{}", e.image))
                    .areas(e.areas.iter().map(|a| crate::game_area(a).unwrap_or(a)));
                if killed && left > 0 {
                    item = item.hint(if self.fr {
                        format!("Encore {left} à vaincre pour les notes du Chasseur")
                    } else {
                        format!("{left} more to defeat for the Hunter's notes")
                    });
                }
                // What the journal shows: the description once met, the
                // Hunter's notes once the entry is complete.
                let mut details: Vec<String> = Vec::new();
                if let Some(d) = self.text(&format!("DESC_{}", e.text)) {
                    details.push(d);
                }
                if status == Status::Done
                    && let Some(n) = self.text(&format!("NOTE_{}", e.text))
                {
                    details.push(format!("{} {n}", self.t("Notes du Chasseur :", "Hunter's notes:")));
                }
                if !details.is_empty() {
                    item = item.details(details.join("\n\n"));
                }
                // An entry not yet met is a creature the reader has not seen: the
                // game itself keeps it hidden.
                if !killed {
                    item = item.spoiler(Spoiler::Major);
                }
                Some(match e.dlc {
                    Some(d) => item.dlc(d),
                    None => item,
                })
            })
            .collect()
    }
}

/// A Journal entry's text key and portrait, by its save key.
pub fn entry(key: &str) -> Option<(&'static str, &'static str)> {
    ENTRIES.iter().find(|e| e.key == key).map(|e| (e.text, e.image))
}

/// The Journal entry of each boss item: its portrait and its official name.
pub const BOSSES: &[(&str, &str)] = &[
    ("killedBigFly", "BigFly"),
    ("falseKnightDefeated", "FalseKnight"),
    ("hornet1Defeated", "Hornet"),
    ("killedMawlek", "Mawlek"),
    ("defeatedDungDefender", "DungDefender"),
    ("mageLordDefeated", "MageLord"),
    ("defeatedMantisLords", "MantisLord"),
    ("killedMimicSpider", "MimicSpider"),
    ("killedInfectedKnight", "InfectedKnight"),
    ("collectorDefeated", "JarCollector"),
    ("defeatedMegaJelly", "MegaJellyfish"),
    ("hornetOutskirtsDefeated", "Hornet"),
    ("killedTraitorLord", "TraitorLord"),
    ("killedBlackKnight", "BlackKnight"),
    ("xeroDefeated", "GhostXero"),
    ("aladarSlugDefeated", "GhostAladar"),
    ("elderHuDefeated", "GhostHu"),
    ("mumCaterpillarDefeated", "GhostMarmu"),
    ("noEyesDefeated", "GhostNoEyes"),
    ("galienDefeated", "GhostGalien"),
    ("markothDefeated", "GhostMarkoth"),
    ("killedHiveKnight", "HiveKnight"),
    ("killedWhiteDefender", "WhiteDefender"),
    ("greyPrinceDefeated", "GreyPrince"),
    ("killedGrimm", "Grimm"),
    ("killedNightmareGrimm", "NightmareGrimm"),
    ("nightmareKing", "NightmareGrimm"),
    ("killedHollowKnight", "HollowKnight"),
    ("killedFinalBoss", "FinalBoss"),
];

/// `MossKnightFat` → `Moss Knight Fat`, when the game's names are not at hand.
fn spaced(key: &str) -> String {
    let mut out = String::new();
    for (i, c) in key.chars().enumerate() {
        if i > 0 && (c.is_ascii_uppercase() || c == '_') {
            out.push(' ');
        }
        if c != '_' {
            out.push(c);
        }
    }
    out
}
