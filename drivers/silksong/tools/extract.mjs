// Builds data/checks.json from the database of Alba6rek/hollow-knight-silksong-completion-check
// (GPL-3.0): what each 100 % entry is, where it is, and how the save records it.
//
//   node tools/extract.mjs path/to/hk-database.js > data/checks.json
import { copyFileSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const src = resolve(process.argv[2]);
const tmp = join(mkdtempSync(join(tmpdir(), "hundo-")), "db.mjs");
copyFileSync(src, tmp);
const HK = (await import(pathToFileURL(tmp))).default;

// Items the reference checks with plain flags on playerData.
const FLAGS = new Set([
  "PurchasedBonebottomHeartPiece", "MerchantEnclaveShellFragment", "PurchasedBelltownSpoolSegment", "MetCaravanTroupeLeaderJudge",
  "MerchantEnclaveSpoolPiece", "purchasedGrindleSpoolPiece", "PurchasedForgeToolKit", "PurchasedArchitectToolKit", "purchasedGrindleToolKit",
  "PurchasedPilgrimsRestToolPouch", "PurchasedBonebottomToolMetal", "MerchantEnclaveToolMetal", "MerchantEnclaveSimpleKey", "CollectedDustCageKey",
  "HasSlabKeyA", "HasSlabKeyB", "HasSlabKeyC", "collectedWardBossKey", "BelltownGreeterHouseFullDlg", "PurchasedArchitectKey", "BallowGivenKey",
  "PurchasedPilgrimsRestMemoryLocket", "PurchasedBelltownMemoryLocket",
]);
const WISHES = new Set(["beastflyHuntWish", "sprintmasterRaceWish", "destroyThreadCoresWish", "antTrapperWish", "saveShermaWish", "crowFeathersWish", "journalWish", "rockRollersWish"]);
// Tools that count once whichever of their forms the player has.
const TOOL_FORMS = {
  webShot: ["WebShot Architect", "WebShot Forge", "WebShot Weaver"],
  curveClaws: ["Curve Claws", "Curve Claws Upgraded"],
  mosscreepTool: ["Mosscreep Tool 1", "Mosscreep Tool 2"],
  DazzleBind: ["Dazzle Bind", "Dazzle Bind Upgraded"],
  deadMansPurse: ["Dead Mans Purse", "Shell Satchel"],
};
const NEEDLE = ["startingNeedle", "sharpenedNeedle", "shiningNeedle", "hivesteelNeedle", "paleSteelNeedle"];

function check(section, key, e) {
  if (["redTools", "blueTools", "yellowTools", "equipment"].includes(section)) return { kind: "tool", names: TOOL_FORMS[key] ?? [e.id] };
  if (section === "NeedleUpgrades") return { kind: "needle", level: NEEDLE.indexOf(key) };
  if (section === "silkHeart") return { kind: "scene", scene: key };
  if (["crests", "skills", "miscellaneous", "flea"].includes(section)) return { kind: "flag", keys: [key] };
  if (FLAGS.has(key)) return { kind: "flag", keys: [key] };
  if (key === "wardKey") return { kind: "flag", keys: ["MerchantEnclaveWardKey", "collectedWardKey"] };
  if (key === "purchasedGrindleSimpleKey") return { kind: "flag", keys: ["purchasedGrindleSimpleKey", "PurchasedBonebottomFaithToken"] };
  if (key === "CrowSummonsAppearedScene") return { kind: "text", key };
  if (key === "CaravanTroupeLocation") return { kind: "intAtLeast", key, value: 3 };
  if (WISHES.has(key)) return { kind: "quest", name: e.sceneName };
  if (key === "ladybugCraftPickupToolPouch") return { kind: "world", scene: e.sceneName, id: e.id, orIntAtLeast: ["pinGalleriesCompleted", 1] };
  return { kind: "world", scene: e.sceneName, id: e.id };
}

// The reference's sections, in the order the game lists them, and how they count.
const SECTIONS = [
  ["crests", "complete"], ["NeedleUpgrades", "complete"], ["silkHeart", "complete"], ["redTools", "complete"],
  ["blueTools", "complete"], ["yellowTools", "complete"], ["equipment", "complete"], ["skills", "complete"],
  ["miscellaneous", "complete"], ["maskShards", "complete", 4], ["spoolFragments", "complete", 2],
  ["craftingKitUpgrades", "complete"], ["toolPouchUpgrades", "complete"],
  ["essentialsMemoryLocket", "beyond"], ["essentialsCraftmetal", "beyond"], ["essentialsKeys", "beyond"], ["flea", "beyond"],
];

const clean = (s) => String(s ?? "").replace(/<[^>]+>/g, "").replace(/^#\d+\s*/, "").replace(/\s+/g, " ").trim();
const out = SECTIONS.map(([id, group, perPercent]) => {
  const s = HK.sections[id];
  let entries = Object.entries(s.entries);
  if (id === "NeedleUpgrades") entries = entries.slice(1); // the starting needle is not an upgrade
  return {
    id,
    title: clean(s.h2),
    group,
    perPercent: perPercent ?? 1,
    items: entries.map(([key, e]) => ({
      id: key,
      name: clean(e.name),
      location: clean(e.spoiler),
      wiki: e.wiki ? `https://hollowknight.wiki/w/${e.wiki.replace(/\+/g, "_")}` : null,
      check: check(id, key, e),
    })),
  };
});
console.log(JSON.stringify({ source: "Alba6rek/hollow-knight-silksong-completion-check (GPL-3.0)", sections: out }, null, 1));
