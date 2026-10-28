//! The game's own picture of each thing Hornet can pick up or buy, by its
//! name in the game's data: the sprite its inventory shows.

/// Identifiers only: an item's name, and its sprite's.
pub const ICONS: &[(&str, &str)] = &[
    ("Ant Trapper Item", "Gilly_Doll"),
    ("Architect Key", "I_key_architect"),
    ("Barbed Wire", "T_barbed_wire"),
    ("Beastfly Remains", "M_crest"),
    ("Bell Bind", "Hornet_icon_0001_T_bell_shield"),
    ("Bone Necklace", "Hornet_Bone_Necklace"),
    ("Broken SilkShot", "_0000_T_web_shot_broken"),
    ("Brolly Spike", "T_brolly_spike"),
    ("Broodmother Remains", "Brood_Mother_Eye_Icon"),
    ("Clover Heart", "clover_prince_heart0000"),
    ("Clover Map", "Shop_map_icon__0003_clover"),
    ("Cog Heart Pieces", "cog_heart_pieces"),
    ("Cog Map", "Shop_map_icon__0012_cog"),
    ("Cogwork Saw", "T_cogwork_saw"),
    ("Conch Drill", "T_Conch_Drill_Shot"),
    ("Coral Heart", "coral_king_heart_icon0000"),
    ("Coral Ingredient", "_0007_quest_coral_nut"),
    ("Cradle Map", "Shop_map_icon__0012_cradle"),
    ("Crawbell", "crawbell"),
    ("Crest Socket Unlocker", "Charm_Notch"),
    ("Crowman Memento", "craw_court_memento"),
    ("Curve Claws", "T_curve_claw"),
    ("Curve Claws Upgraded", "T_curve_claw_upgraded"),
    ("Dazzle Bind", "T_dazzle_bind"),
    ("Dazzle Bind Upgraded", "T_dazzle_bind_upg"),
    ("Dead Mans Purse", "T_dead_purse"),
    ("Dock Demo Key", "dock_key"),
    ("Farsight", "farsight"),
    ("Flea Charm", "Flea_Egg"),
    ("Flintstone", "Hornet_icon_0003_T_flintstone"),
    ("Flower Heart", "flower_queen_heart_icon0000"),
    ("Fractured Mask", "Hornet_T_fractured_mask"),
    ("Great Shard", "Icon_Beast_Shard"),
    ("Grey Memento", "grey_warrior_memento"),
    ("Harpoon", "T_claw_javelin"),
    ("Heart Piece", "Hornet_Spool_Upgrade_Shop_Icon_Heart"),
    ("Hunter Heart", "ant_queen_heart_icon0002"),
    ("Lava Charm", "Hornet_T_lava_charm"),
    ("Lightning Rod", "_0004_T_lightning__0000_1"),
    ("Maggot Charm", "poultice_pouch_icon"),
    ("Magnetite Dice", "_0006_I_magnetite_dice"),
    ("Materium", "materium"),
    ("Memento Surface", "surface_memento"),
    ("Multibind", "T_multi_bind"),
    ("Musician Charm", "T_attunement_charm"),
    ("Pale_Oil", "oil_phial"),
    ("Pinstress Tool", "T_pinstress_tool"),
    ("Pristine Core", "Icon_Pristine_Core"),
    ("Quick Sling", "T_quick_sling"),
    ("Quickbind", "T_quick_bind"),
    ("Reserve Bind", "T_focus_spool"),
    ("Revenge Crystal", "T_revenge_crystal"),
    ("Rosary Cannon", "_0004_T_rosary_cannon_loaded"),
    ("Rosary Magnet", "T_rosary_magnet"),
    ("Rosary_Set_Frayed", "I_rosary_necklace_0002_1_frayed"),
    ("Rosary_Set_Huge_White", "I_rosary_necklace_0001_2"),
    ("Rosary_Set_Large", "I_rosary_necklace_0000_3_old"),
    ("Rosary_Set_Medium", "I_rosary_necklace_0000_3"),
    ("Rosary_Set_Small", "I_rosary_necklace_0002_1"),
    ("Screw Attack", "T_Spine_head"),
    ("Scuttlebrace", "T_steel_spine"),
    ("Shard Pouch", "Hornet_Tool_Metal_Pouch"),
    ("Shell Satchel", "T_shell_satchel"),
    ("Silk Boss Needle", "S_finger_blade"),
    ("Silk Snare", "_0004_T_snare_setter"),
    ("Silk Spool", "Hornet_Spool_Upgrade_Shop_Icon"),
    ("Simple Key", "I_token_of_faith"),
    ("Skull King Fragment", "M_tusk"),
    ("Slab Key A", "I_slab_key"),
    ("Slab Key B", "I_slab_key_gold"),
    ("Slab Key C", "I_slab_key_brass"),
    ("Snare Soul Swamp Bug", "snail_icon__0001_bell_swamp_soul"),
    ("Spool Extender", "T_spool_bar_extender"),
    ("Sprintmaster", "T_icon_sprintmaster"),
    ("Sting Shard", "T_sting_shard"),
    ("Straight Pin", "T_straight_pin"),
    ("Thief Charm", "Thief_Brooch"),
    ("Thief Claw", "Thief_Claw"),
    ("Tool Kit Pickup", "icon_tool_kit_upgrade"),
    ("Tool Metal", "Hornet_Tool_Metal"),
    ("Tool Pouch Pickup", "Inv_tool_pouch_upgrade_02"),
    ("Tri Pin", "T_tri_pin"),
    ("Trobbio Quill Purple", "I_quill_purple"),
    ("Trobbio Quill Red", "I_quill_red"),
    ("Vintage Nectar", "vintage_nectar_item"),
    ("Wallcling", "T_longneedle_old1"),
    ("Ward Boss Key", "I_chute_key_whiteward"),
    ("Ward Key", "I_key_whiteward"),
    ("WebShot Architect", "_0003_T_web_shot_architect"),
    ("WebShot Forge", "_0002_T_web_shot_forge"),
    ("Weighted Anklet", "T_weighted_anklet"),
    ("White Ring", "T_icon_white_ring"),
    ("Wood Witch Item", "mandrake_icon0000"),
    ("Zap Imbuement", "T_zap_imbuement"),
];

/// Bellhome's furnishings, sold by Bellhart: by the ware's name key.
pub const FURNISHINGS: &[(&str, &str)] = &[
    ("SHOP_ITEM_FURNISHING_DESK_NAME", "desk"),
    ("SHOP_ITEM_FURNISHING_LIGHTS_NAME", "gleamflies"),
    ("SHOP_ITEM_FURNISHING_PAINT_NAME", "paint"),
    ("SHOP_ITEM_FURNISHING_SPA_NAME", "spa"),
    ("SHOP_ITEM_FURNISHING_GRAMAPHONE_NAME", "gramaphone"),
];

/// Relics, one picture per family (the type's icon), by the start of their name.
const RELICS: &[(&str, &str)] = &[
    ("Ancient Egg", "Hornet_icon_0004_R_ancient_egg"),
    ("Bone Record", "Hornet_icon_0002_R_bone_record"),
    ("Weaver Record", "Hornet_icon_0002_R_weaver_record"),
    ("Psalm Cylinder", "Hornet_icon_0000_R_psalm_cylinder"),
    ("Librarian Melody", "Hornet_icon_0000_R_psalm_cylinder_white"),
    ("Seal Chit", "Hornet_icon_0000_R_seal_chit"),
    ("Weaver Totem", "Hornet_icon_0005_R_saint_locket"),
];

/// The sprite showing what `name` gives (or a furnishing, by its name
/// key), when the game has one.
pub fn of(name: &str) -> Option<&'static str> {
    ICONS
        .iter()
        .find(|(n, _)| *n == name)
        .or_else(|| FURNISHINGS.iter().find(|(key, _)| *key == name))
        .or_else(|| RELICS.iter().find(|(family, _)| name.starts_with(family)))
        .map(|(_, sprite)| *sprite)
}

/// What the 100 % list names loosely ("Upgrades", "Sample Key #2"): the
/// game's own name, by its key in the `UI` sheet, and its picture, by the
/// section and the entry's name. A name shared by several entries gets a
/// number.
pub const ENTRIES: &[(&str, &str, &str, &str)] = &[
    ("skills", "Swift Step", "INV_NAME_SKILL_SPRINT", "Inv_0029_spell_core_outer_icons_0000_1_sprint"),
    ("skills", "Cling Grip", "INV_NAME_WALLJUMP", "Inv_0029_spell_core_outer_icons_0000_1_wall_jump"),
    ("skills", "Clawline", "INV_NAME_SKILL_HARPOON", "Inv_0029_spell_core_outer_icons_0000_1_harpoon_dash"),
    ("skills", "Needolin", "INV_NAME_SKILL_NEEDOLIN", "Inv_0029_spell_core_outer_icons_0000_1"),
    // No picture of its own in the inventory: the needle's.
    ("skills", "Needle Strike", "INV_NAME_SKILL_CHARGESLASH", "Hornet_Inv_pane_icons_0003_needle_sharpened"),
    ("skills", "Silk Soar", "INV_NAME_SKILL_ASCENT", "Inv_0029_spell_core_outer_icons_0000_1_super_jump"),
    ("miscellaneous", "Sylphsong Step", "INV_NAME_SKILL_EVAHEAL", "Inv_0029_spell_core_outer_icons_eva_heal"),
    ("miscellaneous", "Everbloom", "INV_NAME_WHITE_FLOWER", "I_white_flower"),
    ("craftingKitUpgrades", "Upgrades", "INV_MSG_TOOLKIT_ALT", "icon_tool_kit_upgrade"),
    ("toolPouchUpgrades", "Upgrades", "INV_MSG_TOOLPOUCH_UPGRADE", "Inv_tool_pouch_upgrade_02"),
    ("essentialsMemoryLocket", "Memory Locket", "INV_NAME_CREST_SOCKET", "Charm_Notch"),
    ("essentialsCraftmetal", "Craftmetal", "INV_NAME_TOOL_METAL", "Hornet_Tool_Metal"),
    ("essentialsKeys", "Sample Key", "INV_NAME_TOKEN_FAITH", "I_token_of_faith"),
    ("essentialsKeys", "Key of Indolent", "INV_NAME_SLAB_KEY_A", "I_slab_key"),
    ("essentialsKeys", "Key of Heretic", "INV_NAME_SLAB_KEY_B", "I_slab_key_gold"),
    ("essentialsKeys", "Key of Apostate", "INV_NAME_SLAB_KEY_C", "I_slab_key_brass"),
    // The save's flag is the ward boss's key, which the list calls white.
    ("essentialsKeys", "White Key", "INV_NAME_WARD_BOSS_KEY", "I_chute_key_whiteward"),
    ("essentialsKeys", "Bellhome Key", "INV_NAME_BELL_HOUSE_KEY", "belltown_house_key"),
    ("essentialsKeys", "Diving Bell Key", "INV_NAME_DOCK_KEY", "dock_key"),
    ("essentialsKeys", "Craw Summons", "INV_NAME_CRAW_SUMMONS", "Inv_craw_summons"),
];

/// The game's name key and picture for an entry of a 100 % section; the
/// list's "Sample Key #2" is a "Sample Key".
pub fn entry(section: &str, name: &str) -> Option<(&'static str, &'static str)> {
    let name = name.split(" #").next().unwrap_or(name);
    ENTRIES.iter().find(|(s, n, _, _)| *s == section && *n == name).map(|(_, _, key, sprite)| (*key, *sprite))
}

/// A needle's name key and picture, by its upgrade level (1 = sharpened).
pub fn needle(level: i64) -> (String, String) {
    (format!("INV_NAME_NAIL{}", level + 1), format!("Inv_0033_inv_nail_0{}", level + 1))
}
