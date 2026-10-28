//! Schedule I: a save is a folder of JSON files written by the game's own
//! serializer, each one stamped with `DataType` and `GameVersion`.

use hundo_sdk::{DriverError, Input, Item, Report, Section, Spoiler, Stat, Status};
use serde_json::Value;

struct ScheduleOne;

impl hundo_sdk::Guest for ScheduleOne {
    fn analyze(input: Input) -> Result<Report, DriverError> {
        analyze(&input)
    }
}

hundo_sdk::export!(ScheduleOne);

const RANKS: [&str; 11] = [
    "Street Rat", "Hoodlum", "Peddler", "Hustler", "Bagman", "Enforcer",
    "Shot Caller", "Block Boss", "Underlord", "Baron", "Kingpin",
];
const TIERS_PER_RANK: u32 = 5;
const REGIONS: [&str; 6] = ["Northtown", "Westville", "Downtown", "Docks", "Suburbia", "Uptown"];

pub fn analyze(input: &Input) -> Result<Report, DriverError> {
    let s = Save { input, fr: input.is_french() };
    let meta = s.json("Metadata.json")?;
    let mut report = Report::empty();
    report.game_version = str_of(&meta, "LastSaveVersion").or_else(|| str_of(&meta, "GameVersion"));
    report.title = s.json("Game.json").ok().and_then(|g| str_of(&g, "OrganisationName"));

    let rank = s.json("Rank.json")?;
    let money = s.json("Money.json").unwrap_or(Value::Null);
    let time = s.json("Time.json").unwrap_or(Value::Null);
    let rank_idx = int(&rank, "Rank") as usize;
    let tier = int(&rank, "Tier") as u32;
    let rank_name = format!("{} {}", RANKS.get(rank_idx).unwrap_or(&"?"), roman(tier + 1));

    report.stats = vec![
        Stat::new("rank", s.t("Rang", "Rank"), rank_name.clone()),
        Stat::new("playtime", s.t("Temps de jeu", "Play time"), hundo_sdk::format_duration(num(&time, "Playtime"))),
        Stat::new("days", s.t("Jours écoulés", "Days elapsed"), int(&time, "ElapsedDays").to_string()),
        Stat::new("networth", s.t("Valeur nette", "Net worth"), dollars(num(&money, "Networth"))),
        Stat::new("earnings", s.t("Gains cumulés", "Lifetime earnings"), dollars(num(&money, "LifetimeEarnings"))),
        Stat::new("contracts", s.t("Contrats terminés", "Contracts completed"), s.variable("Completed_Contracts_Count").unwrap_or("0".into())),
        Stat::new("players", s.t("Joueurs", "Players"), s.players().to_string()),
    ];
    if !report.game_version.as_deref().is_some_and(|v| v.starts_with("0.4")) {
        report.warnings.push(s.t(
            "Driver vérifié sur la 0.4 : cette sauvegarde vient d'une autre version, certaines données peuvent manquer.",
            "Driver checked on 0.4: this save comes from another version, some data may be missing.",
        ).into());
    }

    let progress = s.t("Progression", "Progression");
    let empire = s.t("Empire", "Empire");
    let max_tier = (RANKS.len() as u32 - 1) * TIERS_PER_RANK;
    let rank_item = Item::count("kingpin", s.t("Devenir Kingpin", "Reach Kingpin"), rank_idx as u32 * TIERS_PER_RANK + tier, max_tier)
        .hint(format!("{} : {rank_name} · {} XP", s.t("Rang actuel", "Current rank"), int(&rank, "TotalXP")));
    let regions = REGIONS
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let unlocked = rank["UnlockedRegions"].as_array().is_some_and(|r| r.iter().any(|v| v.as_u64() == Some(i as u64)));
            Item::check(format!("region{i}"), *name, unlocked)
        })
        .collect();

    report.sections = vec![
        Section::new("rank", s.t("Rang", "Rank"), vec![rank_item]).group(progress),
        Section::new("quests", s.t("Quêtes", "Quests"), s.quests()?).group(progress),
        Section::new("regions", s.t("Territoires", "Regions"), regions).group(progress),
        Section::new("products", s.t("Types de produit", "Product types"), s.product_types()).group(empire),
        Section::new("properties", s.t("Propriétés", "Properties"), s.owned("Properties/")).group(empire),
        Section::new("businesses", s.t("Commerces", "Businesses"), s.owned("Businesses/")).group(empire),
        Section::new("customers", s.t("Clients", "Customers"), s.customers()).group(empire),
    ];
    Ok(report)
}

struct Save<'a> {
    input: &'a Input,
    fr: bool,
}

impl Save<'_> {
    fn t<'s>(&self, fr: &'s str, en: &'s str) -> &'s str {
        if self.fr { fr } else { en }
    }

    fn json(&self, path: &str) -> Result<Value, DriverError> {
        let file = self.input.file(path)?;
        // The game rewrites every file on save; a half-written one fails to parse.
        serde_json::from_slice(&file.bytes).map_err(|e| DriverError::Incomplete(format!("{path} : {e}")))
    }

    fn variable(&self, name: &str) -> Option<String> {
        let vars = self.json("Variables.json").ok()?;
        vars["Variables"].as_array()?.iter().find(|v| v["Name"] == name).and_then(|v| str_of(v, "Value"))
    }

    fn players(&self) -> usize {
        self.input.files_under("Players/").filter(|f| f.path.ends_with("/Player.json")).count()
    }

    fn quests(&self) -> Result<Vec<Item>, DriverError> {
        let quests = self.json("Quests.json")?;
        let Some(list) = quests["Quests"].as_array() else { return Ok(Vec::new()) };
        Ok(list
            .iter()
            .map(|q| {
                let title = str_of(q, "Title").unwrap_or_default();
                let entries = q["Entries"].as_array().map(Vec::as_slice).unwrap_or_default();
                let done = entries.iter().filter(|e| e["State"] == 2).count() as u32;
                // EQuestState: 0 inactive, 1 active, 2 completed, 3 failed, 4 expired, 5 cancelled.
                let status = match int(q, "State") {
                    2 => Status::Done,
                    1 => Status::InProgress,
                    3..=5 => Status::Missed,
                    _ => Status::Todo,
                };
                let next = entries.iter().find(|e| e["State"] != 2).and_then(|e| str_of(e, "Name"));
                let mut item = Item::new(str_of(q, "GUID").unwrap_or_else(|| title.clone()), title, status);
                if !entries.is_empty() {
                    item.progress = Some(hundo_sdk::Counter { current: done, total: entries.len() as u32 });
                }
                if let (Some(next), false) = (next, status == Status::Done) {
                    item = item.hint(next);
                }
                if let Some(desc) = str_of(q, "Description") {
                    item = item.location(desc);
                }
                item
            })
            .collect())
    }

    fn product_types(&self) -> Vec<Item> {
        let Ok(p) = self.json("Products.json") else { return Vec::new() };
        let discovered: Vec<&str> = p["DiscoveredProducts"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
        let created = |key: &str| p[key].as_array().is_some_and(|a| !a.is_empty());
        let found = |id: &str| discovered.iter().any(|d| d.contains(id));
        vec![
            Item::check("weed", "Weed", created("CreatedWeed") || found("kush")),
            Item::check("meth", "Meth", created("CreatedMeth") || found("meth")),
            Item::check("shrooms", "Shrooms", created("CreatedShrooms") || found("shroom")),
            Item::check("cocaine", "Cocaine", created("CreatedCocaine") || found("cocaine")),
        ]
        .into_iter()
        .map(|i| i.hint(format!("{} {}", discovered.len(), self.t("produits découverts", "products discovered"))))
        .collect()
    }

    /// Properties and businesses: one JSON per place, named after it.
    fn owned(&self, dir: &str) -> Vec<Item> {
        self.input
            .files_under(dir)
            .filter(|f| f.path.matches('/').count() == 1)
            .filter_map(|f| {
                let v: Value = serde_json::from_slice(&f.bytes).ok()?;
                let name = f.path.trim_start_matches(dir).trim_end_matches(".json");
                let id = str_of(&v, "PropertyCode").unwrap_or_else(|| name.to_string());
                Some(Item::check(id, name, v["IsOwned"] == true))
            })
            .collect()
    }

    /// Every NPC that buys from the player, unlocked or not.
    fn customers(&self) -> Vec<Item> {
        let Ok(npcs) = self.json("NPCs.json") else { return Vec::new() };
        let mut items: Vec<Item> = npcs["NPCs"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|npc| {
                let datas = npc["AdditionalDatas"].as_array()?;
                datas.iter().find(|d| d["Name"] == "CustomerData")?;
                // NPC records embed their parts as JSON strings.
                let base: Value = serde_json::from_str(npc["BaseData"].as_str()?).ok()?;
                let id = str_of(&base, "ID")?;
                let rel: Value = datas
                    .iter()
                    .find(|d| d["Name"] == "Relationship")
                    .and_then(|d| serde_json::from_str(d["Contents"].as_str()?).ok())
                    .unwrap_or(Value::Null);
                Some(Item::check(id.clone(), person_name(&id), rel["Unlocked"] == true).spoiler(Spoiler::None))
            })
            .collect();
        items.sort_by(|a, b| a.title.cmp(&b.title));
        items
    }
}

fn str_of(v: &Value, key: &str) -> Option<String> {
    v.get(key)?.as_str().map(str::to_string)
}

fn int(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or(0)
}

fn num(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn dollars(v: f64) -> String {
    let whole = v.round() as i64;
    let digits = whole.abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(c);
    }
    format!("{}{out} $", if whole < 0 { "-" } else { "" })
}

/// `donna_martin` → `Donna Martin`.
fn person_name(id: &str) -> String {
    id.split('_')
        .map(|w| {
            let mut c = w.chars();
            c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn roman(n: u32) -> &'static str {
    ["I", "II", "III", "IV", "V"].get(n.saturating_sub(1) as usize).unwrap_or(&"V")
}
