//! The Hundo driver SDK.
//!
//! A driver implements [`Guest`] and exports it:
//!
//! ```ignore
//! struct MyGame;
//!
//! impl hundo_sdk::Guest for MyGame {
//!     fn analyze(input: Input) -> Result<Report, DriverError> { … }
//! }
//!
//! hundo_sdk::export!(MyGame);
//! ```

// The generated constructors take one argument per field of a record.
#[allow(clippy::too_many_arguments)]
pub mod bindings {
    wit_bindgen::generate!({
        path: "../../wit",
        world: "driver",
        pub_export_macro: true,
        default_bindings_module: "hundo_sdk::bindings",
    });
}

pub use bindings::{Guest, export};
pub use bindings::hundo::driver::types::*;

impl Input {
    /// The file at `path`, relative to the slot's root.
    pub fn file(&self, path: &str) -> Result<&SaveFile, DriverError> {
        self.files
            .iter()
            .find(|f| f.path == path)
            .ok_or_else(|| DriverError::MissingFile(path.to_string()))
    }

    /// Every file whose path starts with `prefix`.
    pub fn files_under<'a>(&'a self, prefix: &'a str) -> impl Iterator<Item = &'a SaveFile> {
        self.files.iter().filter(move |f| f.path.starts_with(prefix))
    }

    pub fn capture(&self, name: &str) -> Option<&str> {
        self.slot.captures.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }

    /// Whether the reader asked for French; drivers translate their own labels.
    pub fn is_french(&self) -> bool {
        self.locale.to_ascii_lowercase().starts_with("fr")
    }

    /// The reader's value for an option of driver.toml's `[[options]]`: the
    /// id of a choice, or "true" / "false" for a switch.
    pub fn option(&self, id: &str) -> Option<&str> {
        self.options.iter().find(|(k, _)| k == id).map(|(_, v)| v.as_str())
    }

    /// A switch of driver.toml's `[[options]]`; `default` if the host did not send it.
    pub fn flag(&self, id: &str, default: bool) -> bool {
        self.option(id).map_or(default, |v| v == "true")
    }
}

impl Item {
    pub fn new(id: impl Into<String>, title: impl Into<String>, status: Status) -> Self {
        Item {
            id: id.into(),
            title: title.into(),
            status,
            progress: None,
            hint: None,
            location: None,
            spoiler: Spoiler::None,
            link: None,
            dlc: None,
            image: None,
            details: None,
            areas: Vec::new(),
        }
    }

    /// Done or to do.
    pub fn check(id: impl Into<String>, title: impl Into<String>, done: bool) -> Self {
        Self::new(id, title, if done { Status::Done } else { Status::Todo })
    }

    /// `current` out of `total`: done once reached, in progress once started.
    pub fn count(id: impl Into<String>, title: impl Into<String>, current: u32, total: u32) -> Self {
        let status = match current {
            c if c >= total => Status::Done,
            0 => Status::Todo,
            _ => Status::InProgress,
        };
        let mut item = Self::new(id, title, status);
        item.progress = Some(Counter { current: current.min(total), total });
        item
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    pub fn spoiler(mut self, spoiler: Spoiler) -> Self {
        self.spoiler = spoiler;
        self
    }

    pub fn link(mut self, link: impl Into<String>) -> Self {
        self.link = Some(link.into());
        self
    }

    /// Marks this item as part of a content pack declared in `[[dlc]]`.
    pub fn dlc(mut self, id: impl Into<String>) -> Self {
        self.dlc = Some(id.into());
        self
    }

    /// A picture: a path in the driver's `assets/`, or `game:<sprite>`
    /// drawn by the host from the installed game.
    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    /// Longer text, shown when the reader opens the item.
    pub fn details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// The areas it belongs to; duplicates are dropped, order is kept.
    pub fn areas<S: Into<String>>(mut self, areas: impl IntoIterator<Item = S>) -> Self {
        for a in areas {
            let a = a.into();
            if !self.areas.contains(&a) {
                self.areas.push(a);
            }
        }
        self
    }
}

impl Section {
    pub fn new(id: impl Into<String>, title: impl Into<String>, items: Vec<Item>) -> Self {
        Section { id: id.into(), title: title.into(), group: None, view: SectionView::List, optional: false, items }
    }

    /// Needed for no completion: left out of the list, the map and the
    /// counts until the reader asks for them.
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Shown as cards with pictures, grouped by area.
    pub fn gallery(mut self) -> Self {
        self.view = SectionView::Gallery;
        self
    }

    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Marks every item of the section as part of a content pack.
    pub fn dlc(mut self, id: &str) -> Self {
        for item in &mut self.items {
            item.dlc = Some(id.into());
        }
        self
    }
}

impl Stat {
    pub fn new(id: impl Into<String>, label: impl Into<String>, value: impl Into<String>) -> Self {
        Stat { id: id.into(), label: label.into(), value: value.into() }
    }
}

impl Report {
    pub fn empty() -> Self {
        Report {
            game_version: None,
            title: None,
            completion: None,
            stats: Vec::new(),
            sections: Vec::new(),
            maps: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl GameMap {
    /// A map `width` by `height` in its own units.
    pub fn new(id: impl Into<String>, title: impl Into<String>, width: f32, height: f32) -> Self {
        GameMap {
            id: id.into(),
            title: title.into(),
            width,
            height,
            background: None,
            regions: Vec::new(),
            markers: Vec::new(),
            links: Vec::new(),
            hidden_layers: Vec::new(),
        }
    }

    /// Hides a layer until the reader shows it: on the map for completeness,
    /// needed by no one.
    pub fn hide_layer(mut self, category: impl Into<String>) -> Self {
        let c = category.into();
        if !self.hidden_layers.contains(&c) {
            self.hidden_layers.push(c);
        }
        self
    }

    /// An image from the driver's `assets/` folder, stretched over the frame.
    pub fn background(mut self, asset: impl Into<String>) -> Self {
        self.background = Some(asset.into());
        self
    }

    pub fn region(mut self, id: &str, label: impl Into<String>, points: &[(f32, f32)]) -> Self {
        self.regions.push(MapRegion { id: id.into(), label: label.into(), points: points.to_vec(), items: Vec::new() });
        self
    }

    pub fn marker(mut self, marker: MapMarker) -> Self {
        self.markers.push(marker);
        self
    }

    pub fn link(mut self, a: &str, b: &str) -> Self {
        self.links.push((a.into(), b.into()));
        self
    }

    /// Puts every item whose area `region_of` knows into that region, with an
    /// approximate marker spread inside its polygon. For drivers that know
    /// where things are only to the area.
    pub fn place_in_regions(
        mut self,
        sections: &[Section],
        region_of: impl Fn(&Item) -> Option<String>,
        category_of: impl Fn(&Section, &Item) -> String,
    ) -> Self {
        let mut per_region: Vec<(usize, Vec<(&Section, &Item)>)> = Vec::new();
        for section in sections {
            for item in &section.items {
                let Some(rid) = region_of(item) else { continue };
                let Some(r) = self.regions.iter().position(|r| r.id == rid) else { continue };
                match per_region.iter_mut().find(|(i, _)| *i == r) {
                    Some((_, list)) => list.push((section, item)),
                    None => per_region.push((r, vec![(section, item)])),
                }
            }
        }
        for (r, list) in per_region {
            let spots = spread(&self.regions[r].points, list.len());
            for ((section, item), (x, y)) in list.into_iter().zip(spots) {
                self.regions[r].items.push(item.id.clone());
                self.markers.push(MapMarker {
                    id: format!("m-{}", item.id),
                    label: item.title.clone(),
                    x,
                    y,
                    item: Some(item.id.clone()),
                    category: category_of(section, item),
                    approximate: true,
                    icon: None,
                });
            }
        }
        self
    }
}

impl MapMarker {
    /// A marker at an exact spot.
    pub fn at(id: impl Into<String>, label: impl Into<String>, x: f32, y: f32) -> Self {
        MapMarker { id: id.into(), label: label.into(), x, y, item: None, category: String::new(), approximate: false, icon: None }
    }

    /// A picture for the marker, as an item's `image` (`game:<sprite>`).
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn item(mut self, id: impl Into<String>) -> Self {
        self.item = Some(id.into());
        self
    }

    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }
}

/// `n` points inside `polygon`, on the finest grid that holds them, away from its edges.
fn spread(polygon: &[(f32, f32)], n: usize) -> Vec<(f32, f32)> {
    if n == 0 || polygon.len() < 3 {
        return Vec::new();
    }
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for &(x, y) in polygon {
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x);
        y1 = y1.max(y);
    }
    let mut step = ((x1 - x0) * (y1 - y0) / n as f32).sqrt();
    for _ in 0..40 {
        let margin = step * 0.35;
        let mut found = Vec::new();
        let mut y = y0 + step / 2.0;
        while y < y1 {
            let mut x = x0 + step / 2.0;
            while x < x1 {
                let clear = [(0.0, 0.0), (margin, 0.0), (-margin, 0.0), (0.0, margin), (0.0, -margin)]
                    .iter()
                    .all(|(dx, dy)| inside(polygon, x + dx, y + dy));
                if clear {
                    found.push((x, y));
                }
                x += step;
            }
            y += step;
        }
        if found.len() >= n {
            // Take every k-th spot so the markers fill the area evenly.
            let k = found.len() as f32 / n as f32;
            return (0..n).map(|i| found[(i as f32 * k) as usize]).collect();
        }
        step *= 0.85;
    }
    let c = polygon.iter().fold((0.0, 0.0), |a, p| (a.0 + p.0, a.1 + p.1));
    vec![(c.0 / polygon.len() as f32, c.1 / polygon.len() as f32); n]
}

fn inside(polygon: &[(f32, f32)], x: f32, y: f32) -> bool {
    let mut odd = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let (xi, yi) = polygon[i];
        let (xj, yj) = polygon[j];
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            odd = !odd;
        }
        j = i;
    }
    odd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spread_points_stay_inside_and_apart() {
        let square = [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)];
        let pts = spread(&square, 9);
        assert_eq!(pts.len(), 9);
        assert!(pts.iter().all(|&(x, y)| inside(&square, x, y)));
        let mut unique = pts.clone();
        unique.dedup();
        assert_eq!(unique.len(), 9);
    }
}

/// `3h 07` from a number of seconds.
pub fn format_duration(seconds: f64) -> String {
    let total = seconds.max(0.0) as u64;
    format!("{} h {:02}", total / 3600, (total % 3600) / 60)
}
