//! What the merchants sell: each purchase is a save flag (`slyShellFrag1`,
//! `salubraNotch2`, `gotCharm_8`), each merchant a pin on the game's map.

use hundo_sdk::Item;

use crate::Analysis;

/// What a merchant sells, by save flag, with its price in geo.
pub struct Ware {
    pub key: &'static str,
    /// The merchant's pin on the game's map.
    pub pin: &'static str,
    pub what: What,
    pub geo: u32,
    /// Only once Sly has his Shopkeeper's Key back.
    pub with_key: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum What {
    Charm,
    Mask,
    Vessel,
    Notch,
    Blessing,
    Key,
    Egg,
    Lantern,
}

const fn w(key: &'static str, pin: &'static str, what: What, geo: u32, with_key: bool) -> Ware {
    Ware { key, pin, what, geo, with_key }
}

pub const WARES: &[Ware] = &[
    w("gotCharm_1", "pin_shop_sly", What::Charm, 300, false),
    w("gotCharm_4", "pin_shop_sly", What::Charm, 200, false),
    w("slyShellFrag1", "pin_shop_sly", What::Mask, 150, false),
    w("slyShellFrag2", "pin_shop_sly", What::Mask, 500, false),
    w("slyVesselFrag1", "pin_shop_sly", What::Vessel, 550, false),
    w("slyRancidEgg", "pin_shop_sly", What::Egg, 60, false),
    w("slySimpleKey", "pin_shop_sly", What::Key, 950, false),
    w("hasLantern", "pin_shop_sly", What::Lantern, 1800, false),
    w("gotCharm_15", "pin_shop_sly", What::Charm, 350, true),
    w("gotCharm_37", "pin_shop_sly", What::Charm, 400, true),
    w("slyShellFrag3", "pin_shop_sly", What::Mask, 800, true),
    w("slyShellFrag4", "pin_shop_sly", What::Mask, 1500, true),
    w("slyVesselFrag2", "pin_shop_sly", What::Vessel, 900, true),
    w("gotCharm_8", "pin_charm_slug", What::Charm, 250, false),
    w("gotCharm_18", "pin_charm_slug", What::Charm, 300, false),
    w("gotCharm_14", "pin_charm_slug", What::Charm, 120, false),
    w("gotCharm_19", "pin_charm_slug", What::Charm, 220, false),
    w("gotCharm_7", "pin_charm_slug", What::Charm, 800, false),
    w("salubraNotch1", "pin_charm_slug", What::Notch, 120, false),
    w("salubraNotch2", "pin_charm_slug", What::Notch, 500, false),
    w("salubraNotch3", "pin_charm_slug", What::Notch, 900, false),
    w("salubraNotch4", "pin_charm_slug", What::Notch, 1400, false),
    w("salubraBlessing", "pin_charm_slug", What::Blessing, 800, false),
    w("gotCharm_23", "pin_shop_leg_eater", What::Charm, 350, false),
    w("gotCharm_24", "pin_shop_leg_eater", What::Charm, 250, false),
    w("gotCharm_25", "pin_shop_leg_eater", What::Charm, 600, false),
    w("gotCharm_2", "pin_shop_mapper", What::Charm, 220, false),
];

pub fn ware(key: &str) -> Option<&'static Ware> {
    WARES.iter().find(|w| w.key == key)
}

impl Analysis<'_> {
    pub(crate) fn merchant(&self, pin: &str) -> &'static str {
        match pin {
            "pin_shop_sly" => "Sly",
            "pin_charm_slug" => "Salubra",
            "pin_shop_leg_eater" => self.t("Mange-Jambes", "Leg Eater"),
            _ => "Iselda",
        }
    }

    /// Where and for how much: "Sly · 800 géo, avec la clé du boutiquier".
    pub(crate) fn price(&self, w: &Ware) -> String {
        let key = if w.with_key { self.t(", avec la clé du boutiquier", ", with the Shopkeeper's Key") } else { "" };
        format!("{} · {} {}{key}", self.merchant(w.pin), w.geo, self.t("géo", "geo"))
    }

    /// The wares that are not charms (those are listed with the charms).
    pub(crate) fn wares(&self) -> Vec<Item> {
        WARES
            .iter()
            .filter(|w| w.what != What::Charm)
            .filter_map(|w| {
                let (fr, en, image) = match w.what {
                    What::Mask => ("Fragment de masque", "Mask shard", Some("select_game_HUD_0001_health")),
                    What::Vessel => ("Fragment de réceptacle", "Vessel fragment", Some("Inv_0031_inv_soul_piece_full")),
                    What::Notch => ("Encoche de charme", "Charm notch", Some("charm_cost_full")),
                    What::Blessing => ("Bénédiction de Salubra", "Salubra's Blessing", None),
                    What::Key => ("Clé simple", "Simple Key", None),
                    What::Egg => ("Œuf rance", "Rancid Egg", Some("inv_rancid_egg")),
                    What::Lantern => ("Lanterne lumoucheron", "Lumafly Lantern", None),
                    What::Charm => return None,
                };
                let mut item = Item::check(w.key, self.t(fr, en), self.bool(w.key)?).hint(self.price(w));
                if let Some(i) = image {
                    item = item.image(format!("game:{i}"));
                }
                Some(item)
            })
            .collect()
    }
}
