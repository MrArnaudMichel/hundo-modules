//! What Pharloom's merchants sell, taken from the game's shop stocks: who
//! sells it and where they stand (a fraction of their room, from the left and
//! from the bottom), its name's key, its price in rosaries, the save flag set
//! once it is bought, and the tool or item it gives.

pub struct W {
    pub merchant: &'static str,
    pub scene: &'static str,
    pub x: f32,
    pub y: f32,
    pub text: &'static str,
    pub geo: u32,
    pub flag: &'static str,
    pub gives: &'static str,
    pub tool: bool,
}

/// Identifiers, positions and prices only.
// Fractions of a room, not π.
#[allow(clippy::approx_constant)]
pub const WARES: &[W] = &[
    W { merchant: "Grindle", scene: "Coral_42", x: 0.403, y: 0.656, text: "THIEF_CHARM_NAME", geo: 350, flag: "", gives: "Thief Charm", tool: true },
    W { merchant: "Grindle", scene: "Coral_42", x: 0.403, y: 0.656, text: "THIEF_CLAW_NAME", geo: 740, flag: "", gives: "Thief Claw", tool: true },
    W { merchant: "Grindle", scene: "Coral_42", x: 0.403, y: 0.656, text: "ROSARY_MAGNET_NAME", geo: 220, flag: "", gives: "Rosary Magnet", tool: true },
    W { merchant: "Grindle", scene: "Coral_42", x: 0.403, y: 0.656, text: "MAGNETITE_DICE_NAME", geo: 300, flag: "", gives: "Magnetite Dice", tool: true },
    W { merchant: "Grindle", scene: "Coral_42", x: 0.403, y: 0.656, text: "SHOP_SHELLFRAG_NAME", geo: 320, flag: "PurchasedBonebottomHeartPiece", gives: "Heart Piece", tool: false },
    W { merchant: "Grindle", scene: "Coral_42", x: 0.403, y: 0.656, text: "SHOP_SPOOL_SEGMENT_NAME", geo: 680, flag: "purchasedGrindleSpoolPiece", gives: "Silk Spool", tool: false },
    W { merchant: "Grindle", scene: "Coral_42", x: 0.403, y: 0.656, text: "INV_NAME_R_PSALM_CYL", geo: 240, flag: "", gives: "Psalm Cylinder Grindle", tool: false },
    W { merchant: "Grindle", scene: "Coral_42", x: 0.403, y: 0.656, text: "INV_NAME_TOOL_METAL", geo: 120, flag: "PurchasedBonebottomToolMetal", gives: "Tool Metal", tool: false },
    W { merchant: "Grindle", scene: "Coral_42", x: 0.403, y: 0.656, text: "INV_NAME_TOOLPOUCH", geo: 220, flag: "PurchasedPilgrimsRestToolPouch", gives: "Tool Pouch Pickup", tool: false },
    W { merchant: "Grindle", scene: "Coral_42", x: 0.403, y: 0.656, text: "INV_NAME_CREST_SOCKET", geo: 250, flag: "PurchasedPilgrimsRestMemoryLocket", gives: "Crest Socket Unlocker", tool: false },
    W { merchant: "Grindle", scene: "Coral_42", x: 0.403, y: 0.656, text: "INV_NAME_TOKEN_FAITH", geo: 600, flag: "PurchasedBonebottomFaithToken", gives: "Simple Key", tool: false },
    W { merchant: "Grindle", scene: "Coral_42", x: 0.403, y: 0.656, text: "INV_MSG_TOOLKIT", geo: 700, flag: "purchasedGrindleToolKit", gives: "Tool Kit Pickup", tool: false },
    W { merchant: "Grindle", scene: "Coral_42", x: 0.403, y: 0.656, text: "INV_NAME_COIN_SET_S", geo: 50, flag: "", gives: "Rosary_Set_Small", tool: false },
    W { merchant: "Bellhart", scene: "Belltown", x: 0.505, y: 0.107, text: "MULTIBIND_NAME", geo: 120, flag: "", gives: "Multibind", tool: true },
    W { merchant: "Bellhart", scene: "Belltown", x: 0.505, y: 0.107, text: "SHOP_SPOOL_SEGMENT_NAME", geo: 270, flag: "PurchasedBelltownSpoolSegment", gives: "Silk Spool", tool: false },
    W { merchant: "Bellhart", scene: "Belltown", x: 0.505, y: 0.107, text: "INV_NAME_CREST_SOCKET", geo: 330, flag: "PurchasedBelltownMemoryLocket", gives: "Crest Socket Unlocker", tool: false },
    W { merchant: "Bellhart", scene: "Belltown", x: 0.505, y: 0.107, text: "SHOP_ITEM_FURNISHING_DESK_NAME", geo: 380, flag: "BelltownFurnishingDesk", gives: "", tool: false },
    W { merchant: "Bellhart", scene: "Belltown", x: 0.505, y: 0.107, text: "SHOP_ITEM_FURNISHING_LIGHTS_NAME", geo: 320, flag: "BelltownFurnishingFairyLights", gives: "", tool: false },
    W { merchant: "Bellhart", scene: "Belltown", x: 0.505, y: 0.107, text: "SHOP_ITEM_FURNISHING_PAINT_NAME", geo: 520, flag: "", gives: "", tool: false },
    W { merchant: "Bellhart", scene: "Belltown", x: 0.505, y: 0.107, text: "SHOP_ITEM_FURNISHING_SPA_NAME", geo: 1100, flag: "BelltownFurnishingSpa", gives: "", tool: false },
    W { merchant: "Bellhart", scene: "Belltown", x: 0.505, y: 0.107, text: "SHOP_ITEM_FURNISHING_GRAMAPHONE_NAME", geo: 490, flag: "BelltownFurnishingGramaphone", gives: "", tool: false },
    W { merchant: "Bellhart", scene: "Belltown", x: 0.505, y: 0.107, text: "INV_NAME_SHARD_POUCH", geo: 100, flag: "", gives: "Shard Pouch", tool: false },
    W { merchant: "Bellhart", scene: "Belltown", x: 0.505, y: 0.107, text: "INV_NAME_COIN_SET_M", geo: 50, flag: "", gives: "Rosary_Set_Medium", tool: false },
    W { merchant: "Pilgrim's Rest", scene: "Bone_East_10_Room", x: 0.284, y: 0.28, text: "WEIGHTED_ANKLET_NAME", geo: 160, flag: "", gives: "Weighted Anklet", tool: true },
    W { merchant: "Pilgrim's Rest", scene: "Bone_East_10_Room", x: 0.284, y: 0.28, text: "INV_NAME_TOOLPOUCH", geo: 220, flag: "PurchasedPilgrimsRestToolPouch", gives: "Tool Pouch Pickup", tool: false },
    W { merchant: "Pilgrim's Rest", scene: "Bone_East_10_Room", x: 0.284, y: 0.28, text: "INV_NAME_CREST_SOCKET", geo: 150, flag: "PurchasedPilgrimsRestMemoryLocket", gives: "Crest Socket Unlocker", tool: false },
    W { merchant: "Pilgrim's Rest", scene: "Bone_East_10_Room", x: 0.284, y: 0.28, text: "INV_NAME_COIN_SET_S", geo: 50, flag: "", gives: "Rosary_Set_Small", tool: false },
    W { merchant: "Pilgrim's Rest", scene: "Bone_East_10_Room", x: 0.284, y: 0.28, text: "INV_NAME_SHARD_POUCH", geo: 100, flag: "", gives: "Shard Pouch", tool: false },
    W { merchant: "Bonebottom", scene: "Bonetown", x: 0.879, y: 0.089, text: "ROSARY_MAGNET_NAME", geo: 120, flag: "", gives: "Rosary Magnet", tool: true },
    W { merchant: "Bonebottom", scene: "Bonetown", x: 0.879, y: 0.089, text: "SHOP_SHELLFRAG_NAME", geo: 300, flag: "PurchasedBonebottomHeartPiece", gives: "Heart Piece", tool: false },
    W { merchant: "Bonebottom", scene: "Bonetown", x: 0.879, y: 0.089, text: "INV_NAME_TOOL_METAL", geo: 60, flag: "PurchasedBonebottomToolMetal", gives: "Tool Metal", tool: false },
    W { merchant: "Bonebottom", scene: "Bonetown", x: 0.879, y: 0.089, text: "INV_NAME_TOKEN_FAITH", geo: 500, flag: "PurchasedBonebottomFaithToken", gives: "Simple Key", tool: false },
    W { merchant: "Bonebottom", scene: "Bonetown", x: 0.879, y: 0.089, text: "INV_NAME_COIN_SET_S", geo: 100, flag: "", gives: "Rosary_Set_Small", tool: false },
    W { merchant: "Citadel", scene: "Song_Enclave", x: 0.842, y: 0.242, text: "WALLCLING_NAME", geo: 350, flag: "", gives: "Wallcling", tool: true },
    W { merchant: "Citadel", scene: "Song_Enclave", x: 0.842, y: 0.242, text: "MUSICIAN_CHARM_NAME", geo: 320, flag: "", gives: "Musician Charm", tool: true },
    W { merchant: "Citadel", scene: "Song_Enclave", x: 0.842, y: 0.242, text: "SHOP_SHELLFRAG_NAME", geo: 750, flag: "MerchantEnclaveShellFragment", gives: "Heart Piece", tool: false },
    W { merchant: "Citadel", scene: "Song_Enclave", x: 0.842, y: 0.242, text: "INV_NAME_WARD_KEY", geo: 220, flag: "MerchantEnclaveWardKey", gives: "Ward Key", tool: false },
    W { merchant: "Citadel", scene: "Song_Enclave", x: 0.842, y: 0.242, text: "INV_NAME_TOKEN_FAITH", geo: 650, flag: "MerchantEnclaveSimpleKey", gives: "Simple Key", tool: false },
    W { merchant: "Citadel", scene: "Song_Enclave", x: 0.842, y: 0.242, text: "SPOOL_EXTENDER_NAME", geo: 720, flag: "", gives: "Spool Extender", tool: true },
    W { merchant: "Citadel", scene: "Song_Enclave", x: 0.842, y: 0.242, text: "SHOP_SPOOL_SEGMENT_NAME", geo: 500, flag: "MerchantEnclaveSpoolPiece", gives: "Silk Spool", tool: false },
    W { merchant: "Citadel", scene: "Song_Enclave", x: 0.842, y: 0.242, text: "INV_NAME_TOOL_METAL", geo: 180, flag: "MerchantEnclaveToolMetal", gives: "Tool Metal", tool: false },
    W { merchant: "Citadel", scene: "Song_Enclave", x: 0.842, y: 0.242, text: "INV_NAME_R_SEAL_CHIT", geo: 180, flag: "", gives: "Seal Chit City Merchant", tool: false },
    W { merchant: "Citadel", scene: "Song_Enclave", x: 0.842, y: 0.242, text: "INV_NAME_COIN_SET_S", geo: 50, flag: "", gives: "Rosary_Set_Small", tool: false },
    W { merchant: "Forge", scene: "Room_Forge", x: 0.814, y: 0.507, text: "SILKSHOT_NAME", geo: 240, flag: "", gives: "WebShot Forge", tool: true },
    W { merchant: "Forge", scene: "Room_Forge", x: 0.814, y: 0.507, text: "STING_SHARD_NAME", geo: 140, flag: "", gives: "Sting Shard", tool: true },
    W { merchant: "Forge", scene: "Room_Forge", x: 0.814, y: 0.507, text: "LAVA_CHARM_NAME", geo: 110, flag: "", gives: "Lava Charm", tool: true },
    W { merchant: "Forge", scene: "Room_Forge", x: 0.814, y: 0.507, text: "INV_MSG_TOOLKIT", geo: 180, flag: "PurchasedForgeToolKit", gives: "Tool Kit Pickup", tool: false },
    W { merchant: "Forge", scene: "Room_Forge", x: 0.814, y: 0.507, text: "INV_NAME_SHARD_POUCH", geo: 100, flag: "", gives: "Shard Pouch", tool: false },
    W { merchant: "Architect", scene: "Under_17", x: 0.47, y: 0.583, text: "SILKSHOT_NAME", geo: 130, flag: "", gives: "WebShot Architect", tool: true },
    W { merchant: "Architect", scene: "Under_17", x: 0.47, y: 0.583, text: "COGWORK_SAW_NAME", geo: 360, flag: "", gives: "Cogwork Saw", tool: true },
    W { merchant: "Architect", scene: "Under_17", x: 0.47, y: 0.583, text: "BROLLYSPIKE_NAME", geo: 230, flag: "", gives: "Brolly Spike", tool: true },
    W { merchant: "Architect", scene: "Under_17", x: 0.47, y: 0.583, text: "SCUTTLEBRACE_NAME", geo: 140, flag: "", gives: "Scuttlebrace", tool: true },
    W { merchant: "Architect", scene: "Under_17", x: 0.47, y: 0.583, text: "INV_MSG_TOOLKIT", geo: 450, flag: "PurchasedArchitectToolKit", gives: "Tool Kit Pickup", tool: false },
    W { merchant: "Architect", scene: "Under_17", x: 0.47, y: 0.583, text: "INV_NAME_ARCHITECT_KEY", geo: 110, flag: "PurchasedArchitectKey", gives: "Architect Key", tool: false },
    W { merchant: "Architect", scene: "Under_17", x: 0.47, y: 0.583, text: "INV_NAME_SHARD_POUCH", geo: 100, flag: "", gives: "Shard Pouch", tool: false },
    W { merchant: "Ant Merchant", scene: "Ant_Merchant", x: 0.099, y: 0.567, text: "INV_NAME_SHARD_POUCH", geo: 100, flag: "", gives: "Shard Pouch", tool: false },
    W { merchant: "Ant Merchant", scene: "Ant_Merchant", x: 0.099, y: 0.567, text: "FRACTURED_MASK_NAME", geo: 260, flag: "", gives: "Fractured Mask", tool: true },
    W { merchant: "Ant Merchant", scene: "Ant_Merchant", x: 0.099, y: 0.567, text: "CURVE_CLAWS_NAME", geo: 140, flag: "", gives: "Curve Claws", tool: true },
];

/// The ware a 100 % entry is bought as: by its save flag, or the tool it gives.
pub fn ware_for(flags: &[String], tools: &[String]) -> Option<&'static W> {
    WARES.iter().find(|w| (!w.flag.is_empty() && flags.iter().any(|f| f == w.flag)) || (w.tool && tools.iter().any(|t| t == w.gives)))
}

/// "Grindle · 320 chapelets".
pub fn price(w: &W, fr: bool) -> String {
    format!("{} · {} {}", w.merchant, w.geo, if fr { "chapelets" } else { "rosaries" })
}
