use crate::names::Gender;
use crate::scan::natural_cmp;
use std::fs;
use std::path::{Path, PathBuf};

pub struct PaletteEntry {
    pub label: String,
    pub path: PathBuf,
}

pub struct PaletteList {
    pub entries: Vec<PaletteEntry>,
    /// None = original SPR palette.
    pub selected: Option<PathBuf>,
}

impl Default for PaletteList {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            selected: None,
        }
    }
}

/// Pre-scanned palette files for instant filtering.
pub struct PaletteIndex {
    /// (filename stem without .pal, path, optional costume folder)
    pub body: Vec<(String, PathBuf, Option<String>)>,
    pub head: Vec<(String, PathBuf, Option<String>)>,
}

pub fn gender_tag(gender: Gender) -> &'static str {
    match gender {
        Gender::Male => "남",
        Gender::Female => "여",
    }
}

pub fn build_palette_index(body_dir: Option<&Path>, head_dir: Option<&Path>) -> PaletteIndex {
    let mut body = Vec::new();
    let mut head = Vec::new();
    if let Some(dir) = body_dir {
        index_pal_dir(dir, None, &mut body);
        if let Ok(read) = fs::read_dir(dir) {
            for entry in read.flatten() {
                let Ok(ft) = entry.file_type() else {
                    continue;
                };
                if !ft.is_dir() {
                    continue;
                }
                let costume = entry.file_name().to_string_lossy().into_owned();
                index_pal_dir(&entry.path(), Some(costume), &mut body);
            }
        }
    }
    if let Some(dir) = head_dir {
        index_pal_dir(dir, None, &mut head);
    }
    PaletteIndex { body, head }
}

fn index_pal_dir(
    dir: &Path,
    costume: Option<String>,
    out: &mut Vec<(String, PathBuf, Option<String>)>,
) {
    let Ok(read) = fs::read_dir(dir) else {
        return;
    };
    for entry in read.flatten() {
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if !ft.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name
            .strip_suffix(".pal")
            .or_else(|| name.strip_suffix(".PAL"))
        else {
            continue;
        };
        out.push((stem.to_string(), entry.path(), costume.clone()));
    }
}

pub fn filter_body_palettes(index: &PaletteIndex, stem: &str, gender: Gender) -> PaletteList {
    let tag = gender_tag(gender);
    let prefix = format!("{stem}_{tag}_");
    let mut entries = Vec::new();
    for (name, path, costume) in &index.body {
        if !name.starts_with(&prefix) {
            continue;
        }
        let rest = &name[prefix.len()..];
        let label = match costume {
            Some(c) => format!("{c} #{rest}"),
            None => rest.to_string(),
        };
        entries.push(PaletteEntry {
            label,
            path: path.clone(),
        });
    }
    entries.sort_by(|a, b| natural_cmp(&a.label, &b.label));
    PaletteList {
        entries,
        selected: None,
    }
}

pub fn filter_head_palettes(index: &PaletteIndex, stem: &str, gender: Gender) -> PaletteList {
    let tag = gender_tag(gender);
    let prefix = format!("머리{stem}_{tag}_");
    let mut entries = Vec::new();
    for (name, path, _) in &index.head {
        if !name.starts_with(&prefix) {
            continue;
        }
        let rest = &name[prefix.len()..];
        entries.push(PaletteEntry {
            label: rest.to_string(),
            path: path.clone(),
        });
    }
    entries.sort_by(|a, b| natural_cmp(&a.label, &b.label));
    PaletteList {
        entries,
        selected: None,
    }
}

pub fn scan_body_palettes(dir: &Path, stem: &str, gender: Gender) -> PaletteList {
    let index = build_palette_index(Some(dir), None);
    filter_body_palettes(&index, stem, gender)
}

pub fn scan_head_palettes(dir: &Path, stem: &str, gender: Gender) -> PaletteList {
    let index = build_palette_index(None, Some(dir));
    filter_head_palettes(&index, stem, gender)
}
