use crate::names::{self, Gender};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct FsItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

#[derive(Clone)]
pub struct FolderRow {
    pub depth: usize,
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

pub struct FolderBrowser {
    pub root: PathBuf,
    pub listings: HashMap<PathBuf, Vec<FsItem>>,
    pub expanded: HashSet<PathBuf>,
    pub rows: Vec<FolderRow>,
    pub selected: Option<PathBuf>,
}

#[derive(Clone)]
pub struct SpriteVariant {
    pub label: String,
    pub path: PathBuf,
}

pub struct ClassEntry {
    pub stem: String,
    pub label: String,
    pub male: Vec<SpriteVariant>,
    pub female: Vec<SpriteVariant>,
}

pub struct ClassBrowser {
    pub entries: Vec<ClassEntry>,
    pub expanded: HashSet<String>,
    pub expanded_gender: HashSet<(String, bool)>,
    pub selected: Option<PathBuf>,
}

pub enum HeadgearRow {
    Class {
        stem: String,
        label: String,
        expanded: bool,
    },
    Gender {
        stem: String,
        is_male: bool,
        multi: bool,
        expanded: bool,
        selected: bool,
        path: Option<PathBuf>,
    },
    Variant {
        path: PathBuf,
        label: String,
        selected: bool,
    },
}

/// Korean then ANSI folder name candidates for body sprites.
pub const BODY_SEGMENTS: &[&[&str]] = &[
    &["몸통"],
    &["인간족", "몸통"],
    &["sprite", "인간족", "몸통"],
    &["¸öÅë"],
    &["ÀÎ°£Á·", "¸öÅë"],
    &["sprite", "ÀÎ°£Á·", "¸öÅë"],
];

/// Korean then ANSI folder name candidates for head sprites.
pub const HEAD_SEGMENTS: &[&[&str]] = &[
    &["머리통"],
    &["인간족", "머리통"],
    &["sprite", "인간족", "머리통"],
    &["¸Ó¸®Åë"],
    &["ÀÎ°£Á·", "¸Ó¸®Åë"],
    &["sprite", "ÀÎ°£Á·", "¸Ó¸®Åë"],
];

/// Root of class folders that hold weapon sprites (siblings of 몸통/머리통).
pub const WEAPON_ROOT_SEGMENTS: &[&[&str]] = &[
    &["인간족"],
    &["sprite", "인간족"],
    &["ÀÎ°£Á·"],
    &["sprite", "ÀÎ°£Á·"],
];

pub const WEAPON_EXCLUDE_DIRS: &[&str] = &[
    "몸통",
    "머리통",
    "아이템",
    "악세사리",
    "악세서리",
    "¸öÅë",
    "¸Ó¸®Åë",
    "¾ÆÀÌÅÛ",
    "¾Ç¼¼»ç¸®",
];

/// Headgears: 악세사리/{남,여}
pub const HEADGEAR_SEGMENTS: &[&[&str]] = &[
    &["악세사리"],
    &["sprite", "악세사리"],
    &["악세서리"],
    &["sprite", "악세서리"],
    &["¾Ç¼¼»ç¸®"],
    &["sprite", "¾Ç¼¼»ç¸®"],
];

pub const MALE_DIRS: &[&str] = &["남", "³²"];
pub const FEMALE_DIRS: &[&str] = &["여", "¿©"];

pub const BODY_PALETTE_SEGMENTS: &[&[&str]] = &[
    &["palette", "몸"],
    &["몸"],
];

pub const HEAD_PALETTE_SEGMENTS: &[&[&str]] = &[
    &["palette", "머리"],
    &["머리"],
];

pub fn build_headgear_rows(browser: &ClassBrowser) -> Vec<HeadgearRow> {
    let mut rows = Vec::with_capacity(browser.entries.len() * 2);
    for entry in &browser.entries {
        let expanded = browser.expanded.contains(&entry.stem);
        rows.push(HeadgearRow::Class {
            stem: entry.stem.clone(),
            label: entry.label.clone(),
            expanded,
        });
        if !expanded {
            continue;
        }
        for (is_male, variants) in [(true, &entry.male), (false, &entry.female)] {
            if variants.is_empty() {
                continue;
            }
            let gkey = (entry.stem.clone(), is_male);
            let multi = variants.len() > 1;
            let g_expanded = browser.expanded_gender.contains(&gkey);
            let selected = !multi
                && variants
                    .first()
                    .is_some_and(|v| browser.selected.as_ref() == Some(&v.path));
            rows.push(HeadgearRow::Gender {
                stem: entry.stem.clone(),
                is_male,
                multi,
                expanded: g_expanded,
                selected,
                path: variants.first().map(|v| v.path.clone()),
            });
            if multi && g_expanded {
                for v in variants {
                    rows.push(HeadgearRow::Variant {
                        path: v.path.clone(),
                        label: v.label.clone(),
                        selected: browser.selected.as_ref() == Some(&v.path),
                    });
                }
            }
        }
    }
    rows
}

pub fn collect_sprite_paths(browser: &ClassBrowser) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for entry in &browser.entries {
        for variants in [&entry.male, &entry.female] {
            for v in variants {
                paths.push(v.path.clone());
            }
        }
    }
    paths
}

pub fn collect_weapon_paths(browser: &ClassBrowser, stem: &str, gender: Gender) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for entry in &browser.entries {
        if entry.stem != stem {
            continue;
        }
        let variants = match gender {
            Gender::Male => &entry.male,
            Gender::Female => &entry.female,
        };
        for v in variants {
            paths.push(v.path.clone());
        }
    }
    paths
}

pub fn expand_for_sprite(browser: &mut ClassBrowser, path: &Path) {
    for entry in &browser.entries {
        for (is_male, variants) in [(true, &entry.male), (false, &entry.female)] {
            if !variants.iter().any(|v| v.path == path) {
                continue;
            }
            browser.expanded.insert(entry.stem.clone());
            if variants.len() > 1 {
                browser.expanded_gender.insert((entry.stem.clone(), is_male));
            }
            return;
        }
    }
}

/// Compare strings with numeric awareness: "2" < "10", "1" < "100".
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let mut ac = a.chars().peekable();
    let mut bc = b.chars().peekable();
    loop {
        match (ac.peek().copied(), bc.peek().copied()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(ca), Some(cb)) if ca.is_ascii_digit() && cb.is_ascii_digit() => {
                let mut na: u64 = 0;
                while let Some(c) = ac.next_if(|c| c.is_ascii_digit()) {
                    na = na.saturating_mul(10).saturating_add((c as u8 - b'0') as u64);
                }
                let mut nb: u64 = 0;
                while let Some(c) = bc.next_if(|c| c.is_ascii_digit()) {
                    nb = nb.saturating_mul(10).saturating_add((c as u8 - b'0') as u64);
                }
                match na.cmp(&nb) {
                    std::cmp::Ordering::Equal => {}
                    other => return other,
                }
            }
            (Some(ca), Some(cb)) => {
                ac.next();
                bc.next();
                let la = ca.to_ascii_lowercase();
                let lb = cb.to_ascii_lowercase();
                match la.cmp(&lb) {
                    std::cmp::Ordering::Equal => {}
                    other => return other,
                }
            }
        }
    }
}

pub fn find_class_dir(root: &Path, candidates: &[&[&str]]) -> Option<PathBuf> {
    for segs in candidates {
        let mut path = root.to_path_buf();
        for seg in *segs {
            path.push(seg);
        }
        if path.is_dir() {
            return Some(path);
        }
    }
    None
}

fn find_gender_dir(parent: &Path, names: &[&str]) -> Option<PathBuf> {
    for name in names {
        let path = parent.join(name);
        if path.is_dir() {
            return Some(path);
        }
    }
    None
}

fn entries_from_map(
    mut map: BTreeMap<String, (Vec<SpriteVariant>, Vec<SpriteVariant>)>,
) -> ClassBrowser {
    for (male, female) in map.values_mut() {
        sort_variants(male);
        sort_variants(female);
    }
    let mut entries: Vec<ClassEntry> = map
        .into_iter()
        .map(|(stem, (male, female))| {
            let label = names::display_name(&stem);
            ClassEntry {
                stem,
                label,
                male,
                female,
            }
        })
        .collect();

    entries.sort_by(|a, b| {
        natural_cmp(&a.label, &b.label).then_with(|| natural_cmp(&a.stem, &b.stem))
    });

    ClassBrowser {
        entries,
        expanded: HashSet::new(),
        expanded_gender: HashSet::new(),
        selected: None,
    }
}

pub fn scan_class_dir(dir: &Path) -> Result<ClassBrowser, String> {
    let mut map: BTreeMap<String, (Vec<SpriteVariant>, Vec<SpriteVariant>)> = BTreeMap::new();

    if let Some(male_dir) = find_gender_dir(dir, MALE_DIRS) {
        collect_spr_variants(&male_dir, Gender::Male, &mut map)?;
    }
    if let Some(female_dir) = find_gender_dir(dir, FEMALE_DIRS) {
        collect_spr_variants(&female_dir, Gender::Female, &mut map)?;
    }

    Ok(entries_from_map(map))
}

pub fn scan_weapon_dir(root: &Path) -> Result<ClassBrowser, String> {
    let mut map: BTreeMap<String, (Vec<SpriteVariant>, Vec<SpriteVariant>)> = BTreeMap::new();
    let entries = fs::read_dir(root).map_err(|e| format!("{}: {e}", root.display()))?;
    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if !ft.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if WEAPON_EXCLUDE_DIRS.iter().any(|&n| n == name) {
            continue;
        }
        collect_spr_both_genders(&entry.path(), &mut map)?;
    }
    let browser = entries_from_map(map);
    if browser.entries.is_empty() {
        return Err("Aucune arme".into());
    }
    Ok(browser)
}

pub fn scan_headgear_dir(root: &Path) -> Result<ClassBrowser, String> {
    let mut map: BTreeMap<String, (Vec<SpriteVariant>, Vec<SpriteVariant>)> = BTreeMap::new();
    if let Some(male_dir) = find_gender_dir(root, MALE_DIRS) {
        collect_headgear_variants(&male_dir, Gender::Male, &mut map)?;
    }
    if let Some(female_dir) = find_gender_dir(root, FEMALE_DIRS) {
        collect_headgear_variants(&female_dir, Gender::Female, &mut map)?;
    }
    let browser = entries_from_map(map);
    if browser.entries.is_empty() {
        return Err("Aucun headgear".into());
    }
    Ok(browser)
}

fn collect_spr_both_genders(
    dir: &Path,
    map: &mut BTreeMap<String, (Vec<SpriteVariant>, Vec<SpriteVariant>)>,
) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if !ft.is_file() {
            continue;
        }
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !ext.eq_ignore_ascii_case("spr") {
            continue;
        }
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let Some((class_key, gender)) = names::parse_sprite_stem(&stem) else {
            continue;
        };
        let label = variant_label(&stem, &class_key);
        let variant = SpriteVariant { label, path };
        let slot = map.entry(class_key).or_default();
        match gender {
            Gender::Male => slot.0.push(variant),
            Gender::Female => slot.1.push(variant),
        }
    }
    Ok(())
}

fn collect_headgear_variants(
    dir: &Path,
    gender: Gender,
    map: &mut BTreeMap<String, (Vec<SpriteVariant>, Vec<SpriteVariant>)>,
) -> Result<(), String> {
    let prefix = match gender {
        Gender::Male => "남_",
        Gender::Female => "여_",
    };
    let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if !ft.is_file() {
            continue;
        }
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !ext.eq_ignore_ascii_case("spr") {
            continue;
        }
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let Some(item) = stem.strip_prefix(prefix) else {
            continue;
        };
        if item.is_empty() {
            continue;
        }
        let variant = SpriteVariant {
            label: "default".into(),
            path,
        };
        let slot = map.entry(item.to_string()).or_default();
        match gender {
            Gender::Male => {
                if !slot.0.iter().any(|v| v.path == variant.path) {
                    slot.0.push(variant);
                }
            }
            Gender::Female => {
                if !slot.1.iter().any(|v| v.path == variant.path) {
                    slot.1.push(variant);
                }
            }
        }
    }
    Ok(())
}

fn collect_spr_variants(
    dir: &Path,
    gender: Gender,
    map: &mut BTreeMap<String, (Vec<SpriteVariant>, Vec<SpriteVariant>)>,
) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !ext.eq_ignore_ascii_case("spr") {
            continue;
        }
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let Some((class_key, parsed_gender)) = names::parse_sprite_stem(&stem) else {
            continue;
        };
        if parsed_gender != gender {
            continue;
        }
        let label = variant_label(&stem, &class_key);
        let variant = SpriteVariant { label, path };
        let slot = map.entry(class_key).or_default();
        match gender {
            Gender::Male => slot.0.push(variant),
            Gender::Female => slot.1.push(variant),
        }
    }

    for (male, female) in map.values_mut() {
        sort_variants(male);
        sort_variants(female);
    }
    Ok(())
}

fn variant_label(stem: &str, class_key: &str) -> String {
    for marker in ["_남_", "_여_"] {
        if let Some(idx) = stem.find(marker) {
            let after = &stem[idx + marker.len()..];
            if !after.is_empty() {
                return after.to_string();
            }
        }
    }
    if stem == class_key
        || stem == format!("{class_key}_남")
        || stem == format!("{class_key}_여")
    {
        return "default".into();
    }
    stem.to_string()
}

fn sort_variants(variants: &mut [SpriteVariant]) {
    variants.sort_by(|a, b| natural_cmp(&a.label, &b.label));
}

impl FolderBrowser {
    pub fn open(root: PathBuf) -> Result<Self, String> {
        let mut browser = Self {
            root: root.clone(),
            listings: HashMap::new(),
            expanded: HashSet::new(),
            rows: Vec::new(),
            selected: None,
        };
        browser.load_listing(&root)?;
        browser.rebuild_rows();
        Ok(browser)
    }

    pub fn load_listing(&mut self, dir: &Path) -> Result<(), String> {
        if self.listings.contains_key(dir) {
            return Ok(());
        }
        let items = read_listing(dir, &self.root)?;
        self.listings.insert(dir.to_path_buf(), items);
        Ok(())
    }

    pub fn toggle(&mut self, path: &Path) -> Result<(), String> {
        if self.expanded.contains(path) {
            self.expanded.remove(path);
        } else {
            self.load_listing(path)?;
            self.expanded.insert(path.to_path_buf());
        }
        self.rebuild_rows();
        Ok(())
    }

    pub fn rebuild_rows(&mut self) {
        self.rows.clear();
        append_rows(
            &mut self.rows,
            &self.listings,
            &self.expanded,
            &self.root,
            0,
        );
    }
}

fn read_listing(dir: &Path, browser_root: &Path) -> Result<Vec<FsItem>, String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut items = Vec::new();
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let is_dir = file_type.is_dir();
        if !is_dir && !file_type.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        if is_dir {
            if !is_useful_dir(browser_root, &path, &name) {
                continue;
            }
        } else if !is_useful_file(&name) {
            continue;
        }
        items.push(FsItem {
            name,
            path,
            is_dir,
        });
    }
    items.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then_with(|| {
            a.name
                .bytes()
                .map(|c| c.to_ascii_lowercase())
                .cmp(b.name.bytes().map(|c| c.to_ascii_lowercase()))
        })
    });
    Ok(items)
}

fn is_useful_file(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "spr" | "act" | "pal"
            )
        })
}

fn is_useful_dir_name(name: &str) -> bool {
    if name.starts_with("costume_") {
        return true;
    }
    const NAMES: &[&str] = &[
        "sprite",
        "palette",
        "npc",
        "emotion",
        "homun",
        "book",
        // Korean (folders.MD)
        "인간족",
        "몸통",
        "머리통",
        "남",
        "여",
        "몬스터",
        "도람족",
        "로브",
        "아이템",
        "악세사리",
        "악세서리",
        "방패",
        "이팩트",
        "이펙트",
        "몸",
        "머리",
        // ANSI mojibake variants
        "ÀÎ°£Á·",
        "¸öÅë",
        "¸Ó¸®Åë",
        "³²",
        "¿©",
        "¸ó½ºÅÍ",
        "µµ¶÷Á·",
        "·Îºê",
        "¾ÆÀÌÅÛ",
        "¾Ç¼¼»ç¸®",
        "¹æÆÐ",
        "ÀÌÆÑÆ®",
    ];
    NAMES.iter().any(|&n| n == name)
}

fn is_useful_dir(browser_root: &Path, dir_path: &Path, name: &str) -> bool {
    if is_useful_dir_name(name) {
        return true;
    }
    // Already inside a useful tree: show all subfolders (classes, weapons…).
    if browser_root
        .file_name()
        .is_some_and(|n| is_useful_dir_name(&n.to_string_lossy()))
    {
        return true;
    }
    for ancestor in dir_path.ancestors().skip(1) {
        if ancestor == browser_root {
            break;
        }
        if ancestor
            .file_name()
            .is_some_and(|n| is_useful_dir_name(&n.to_string_lossy()))
        {
            return true;
        }
    }
    false
}

fn append_rows(
    rows: &mut Vec<FolderRow>,
    listings: &HashMap<PathBuf, Vec<FsItem>>,
    expanded: &HashSet<PathBuf>,
    dir: &Path,
    depth: usize,
) {
    let count = listings.get(dir).map(|items| items.len()).unwrap_or(0);
    for index in 0..count {
        let (name, path, is_dir) = {
            let item = &listings[dir][index];
            (item.name.clone(), item.path.clone(), item.is_dir)
        };
        let expand = is_dir && expanded.contains(&path);
        rows.push(FolderRow {
            depth,
            name,
            path: path.clone(),
            is_dir,
        });
        if expand {
            append_rows(rows, listings, expanded, &path, depth + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    #[test]
    fn folder_browser_lists_spr() {
        let dir = tempfile::tempdir().expect("tmp");
        // Root named "sprite" so is_useful_dir accepts children.
        let root = dir.path().join("sprite");
        std::fs::create_dir_all(&root).expect("mkdir");
        fixtures::write_pair(&root, "npc_test").expect("pair");
        let browser = FolderBrowser::open(root.clone()).expect("open");
        assert!(!browser.rows.is_empty());
        assert!(browser.rows.iter().any(|r| r.name.ends_with(".spr")));
    }

    #[test]
    fn scan_class_dir_genders() {
        let dir = tempfile::tempdir().expect("tmp");
        let body = dir.path().join("몸통");
        let male = body.join("남");
        let female = body.join("여");
        fixtures::write_pair(&male, "초보자_남").expect("male");
        fixtures::write_pair(&female, "초보자_여").expect("female");
        let browser = scan_class_dir(&body).expect("scan");
        assert_eq!(browser.entries.len(), 1);
        assert_eq!(browser.entries[0].stem, "초보자");
        assert_eq!(browser.entries[0].male.len(), 1);
        assert_eq!(browser.entries[0].female.len(), 1);
    }

    #[test]
    fn natural_cmp_orders_numbers() {
        assert_eq!(natural_cmp("2", "10"), std::cmp::Ordering::Less);
        assert_eq!(natural_cmp("a10", "a2"), std::cmp::Ordering::Greater);
    }
}
