use crate::act::{ActFile, Frame};
use crate::names::{self, Gender};
use crate::render::{action_frame, render_frame, AttachLayer, AttachMode, RenderedFrame};
use crate::spr::SprFile;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct SpritePair {
    pub name: String,
    pub spr: Arc<SprFile>,
    pub act: Arc<ActFile>,
}

pub struct SpriteSel {
    pub stem: String,
    pub gender: Gender,
}

pub struct RandomResult {
    pub body: SpritePair,
    pub body_path: PathBuf,
    pub head: Option<(SpritePair, PathBuf)>,
    pub body_pal: Option<PathBuf>,
    pub head_pal: Option<PathBuf>,
}

pub fn sprite_sel_from_name(name: &str) -> Option<SpriteSel> {
    let (stem, gender) = names::parse_sprite_stem(name)?;
    Some(SpriteSel { stem, gender })
}

pub fn load_pair(path: &Path) -> Result<SpritePair, String> {
    let (spr_path, act_path) = match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("spr") => (path.to_path_buf(), path.with_extension("act")),
        Some("act") => (path.with_extension("spr"), path.to_path_buf()),
        _ => return Err("Choisissez un fichier .spr ou .act".into()),
    };

    if !spr_path.exists() {
        return Err(format!("SPR manquant: {}", spr_path.display()));
    }
    if !act_path.exists() {
        return Err(format!("ACT manquant: {}", act_path.display()));
    }

    let spr = Arc::new(SprFile::open(&spr_path).map_err(|e| e.to_string())?);
    let act = Arc::new(ActFile::open(&act_path).map_err(|e| e.to_string())?);
    let name = spr_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "sprite".into());

    Ok(SpritePair { name, spr, act })
}

pub fn overlay_frame(
    pair: &SpritePair,
    action_idx: usize,
    frame_idx: usize,
) -> Option<(&SprFile, &Frame)> {
    let af = action_frame(&pair.act, action_idx, frame_idx)
        .or_else(|| action_frame(&pair.act, action_idx, 0))
        .or_else(|| action_frame(&pair.act, 0, 0))?;
    Some((pair.spr.as_ref(), af))
}

pub fn compose_frame(
    base: &SpritePair,
    action_idx: usize,
    frame_idx: usize,
    weapon: Option<&SpritePair>,
    head: Option<&SpritePair>,
    headgear: Option<&SpritePair>,
    base_anchor_delta: (i32, i32),
) -> Option<RenderedFrame> {
    let frame = action_frame(&base.act, action_idx, frame_idx)?;
    let mut attaches = Vec::new();
    // Order: weapon (origin), head (anchor), headgear (anchor).
    if let Some(w) = weapon {
        if let Some((spr, af)) = overlay_frame(w, action_idx, frame_idx) {
            attaches.push(AttachLayer {
                spr,
                frame: af,
                mode: AttachMode::Origin,
            });
        }
    }
    if let Some(a) = head {
        if let Some((spr, af)) = overlay_frame(a, action_idx, frame_idx) {
            attaches.push(AttachLayer {
                spr,
                frame: af,
                mode: AttachMode::Anchor,
            });
        }
    }
    if let Some(h) = headgear {
        if let Some((spr, af)) = overlay_frame(h, action_idx, frame_idx) {
            attaches.push(AttachLayer {
                spr,
                frame: af,
                mode: AttachMode::Anchor,
            });
        }
    }
    Some(render_frame(
        base.spr.as_ref(),
        frame,
        &attaches,
        16,
        base_anchor_delta,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    #[test]
    fn load_pair_from_temp() {
        let dir = tempfile::tempdir().expect("tmp");
        let spr_path = fixtures::write_pair(dir.path(), "초보자_남").expect("write");
        let pair = load_pair(&spr_path).expect("load");
        assert_eq!(pair.name, "초보자_남");
        assert_eq!(pair.spr.images.len(), 1);
        assert_eq!(pair.act.actions.len(), 1);
    }

    #[test]
    fn compose_single_body() {
        let dir = tempfile::tempdir().expect("tmp");
        let spr_path = fixtures::write_pair(dir.path(), "test_남").expect("write");
        let pair = load_pair(&spr_path).expect("load");
        let rf = compose_frame(&pair, 0, 0, None, None, None, (0, 0)).expect("compose");
        assert!(rf.width > 0);
        assert!(rf.pixels.iter().any(|&b| b != 0));
    }

    #[test]
    fn sprite_sel_parses_gender() {
        let sel = sprite_sel_from_name("검사_여").expect("sel");
        assert_eq!(sel.stem, "검사");
        assert_eq!(sel.gender, Gender::Female);
    }
}
