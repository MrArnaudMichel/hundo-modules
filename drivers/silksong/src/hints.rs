//! The 100 % list's hints in French. Its wording is ours; the names in it
//! (places, wishes, foes, abilities) are markers to the game's own text,
//! `{t:KEY|English}`, so a reader sees the names the French game uses — and
//! the English ones when the game's text cannot be read.

use std::collections::HashMap;

use crate::Words;

const FRENCH: &str = include_str!("../data/hints-fr.json");

/// A hint of the list in French, when there is a wording for it.
pub fn french(hint: &str, words: &Words, table: &HashMap<String, String>) -> Option<String> {
    table.get(hint).map(|template| fill(template, words))
}

/// The table, read once per report.
pub fn table() -> HashMap<String, String> {
    serde_json::from_str(FRENCH).expect("data/hints-fr.json is checked by the tests")
}

/// `{t:KEY|English}` → the game's text for KEY, or the English name.
fn fill(template: &str, words: &Words) -> String {
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find("{t:") {
        out.push_str(&rest[..start]);
        let Some(end) = rest[start..].find('}') else { break };
        let marker = &rest[start + 3..start + end];
        let (key, english) = marker.split_once('|').unwrap_or((marker, marker));
        out.push_str(&words.get(key).unwrap_or_else(|| english.to_string()));
        rest = &rest[start + end + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_come_from_the_game_or_stay_english() {
        let words = Words::from_pairs(&[("SWAMP", "Bilesac")]);
        assert_eq!(fill("{t:SWAMP|Bilewater}, à l'est", &words), "Bilesac, à l'est");
        assert_eq!(fill("{t:DOCKS|Deep Docks}", &words), "Deep Docks");
    }

    #[test]
    fn every_hint_of_the_list_has_a_wording() {
        let table = table();
        let data: serde_json::Value = serde_json::from_str(crate::CHECKS).unwrap();
        for s in data["sections"].as_array().unwrap() {
            for i in s["items"].as_array().unwrap() {
                if let Some(l) = i["location"].as_str().filter(|l| !l.is_empty()) {
                    assert!(table.contains_key(l), "pas de traduction pour « {l} »");
                }
            }
        }
    }
}
