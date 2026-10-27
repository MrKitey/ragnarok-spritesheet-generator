use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ActError {
    #[error("invalid signature (expected AC)")]
    BadSignature,
    #[error("unsupported ACT version: {0}")]
    UnsupportedVersion(f32),
    #[error("read: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct Anchor {
    pub x: i32,
    pub y: i32,
    pub attr: i32,
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub x: i32,
    pub y: i32,
    pub spr_no: i32,
    pub mirror: bool,
    pub color: [u8; 4],
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: i32,
    pub spr_type: i32,
    #[allow(dead_code)]
    pub width: i32,
    #[allow(dead_code)]
    pub height: i32,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub layers: Vec<Layer>,
    pub event_id: i32,
    pub anchors: Vec<Anchor>,
}

#[derive(Debug, Clone)]
pub struct Action {
    pub frames: Vec<Frame>,
    /// Frame duration in milliseconds.
    pub frame_ms: f32,
}

#[derive(Debug, Clone)]
pub struct ActFile {
    pub version: f32,
    pub actions: Vec<Action>,
    pub events: Vec<String>,
}

impl ActFile {
    pub fn open(path: &std::path::Path) -> Result<Self, ActError> {
        let data = std::fs::read(path)?;
        Self::parse(&data)
    }

    pub fn parse(data: &[u8]) -> Result<Self, ActError> {
        let mut r = Cursor::new(data);

        let mut sig = [0u8; 2];
        r.read_exact(&mut sig)?;
        if &sig != b"AC" {
            return Err(ActError::BadSignature);
        }

        let minor = r.read_u8()? as f32;
        let major = r.read_u8()? as f32;
        let version = major + minor / 10.0;
        if !(2.0..=2.5).contains(&version) {
            return Err(ActError::UnsupportedVersion(version));
        }

        let n_actions = r.read_u16::<LittleEndian>()? as usize;
        let mut reserved = [0u8; 10];
        r.read_exact(&mut reserved)?;

        let mut actions = Vec::with_capacity(n_actions);
        for _ in 0..n_actions {
            let n_frames = r.read_u32::<LittleEndian>()? as usize;
            let mut frames = Vec::with_capacity(n_frames);
            for _ in 0..n_frames {
                frames.push(read_frame(&mut r, version)?);
            }
            actions.push(Action {
                frames,
                frame_ms: 96.0, // 4 ticks * 24 ms default
            });
        }

        let mut events = Vec::new();
        if version >= 2.1 {
            let n_events = r.read_i32::<LittleEndian>()?;
            if n_events > 0 {
                events.reserve(n_events as usize);
                for _ in 0..n_events {
                    let mut buf = [0u8; 40];
                    r.read_exact(&mut buf)?;
                    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
                    events.push(String::from_utf8_lossy(&buf[..end]).into_owned());
                }
            }
        }

        if version >= 2.2 {
            for action in &mut actions {
                let ticks = r.read_f32::<LittleEndian>()?;
                action.frame_ms = ticks * 24.0;
            }
        }

        Ok(Self {
            version,
            actions,
            events,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;
    use std::path::PathBuf;

    #[test]
    fn parse_mini_act_v20() {
        let act = ActFile::parse(&fixtures::mini_act_bytes()).expect("act");
        assert!((act.version - 2.0).abs() < 0.01);
        assert_eq!(act.actions.len(), 1);
        assert_eq!(act.actions[0].frames.len(), 1);
        assert_eq!(act.actions[0].frames[0].layers.len(), 1);
        assert!(act.actions[0].frames[0].anchors.is_empty());
    }

    #[test]
    fn parse_mini_act_with_anchor() {
        let act = ActFile::parse(&fixtures::mini_act_bytes_with_anchor(Some((3, -2)))).expect("act");
        assert!((act.version - 2.5).abs() < 0.01);
        let frame = &act.actions[0].frames[0];
        assert_eq!(frame.anchors.len(), 1);
        assert_eq!(frame.anchors[0].x, 3);
        assert_eq!(frame.anchors[0].y, -2);
        assert!((act.actions[0].frame_ms - 96.0).abs() < 0.01);
    }

    #[test]
    #[ignore = "requires local data/"]
    fn parse_aira_act() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/sprite/npc/4_aira.act");
        let act = ActFile::open(&path).expect("act");
        assert!((act.version - 2.5).abs() < 0.01);
        assert!(!act.actions.is_empty());
    }
}

fn read_frame(r: &mut Cursor<&[u8]>, version: f32) -> Result<Frame, ActError> {
    // range1 + range2 (unused)
    for _ in 0..8 {
        let _ = r.read_u32::<LittleEndian>()?;
    }

    let n_layers = r.read_u32::<LittleEndian>()? as usize;
    let mut layers = Vec::with_capacity(n_layers);
    for _ in 0..n_layers {
        layers.push(read_layer(r, version)?);
    }

    let event_id = if version >= 2.0 {
        r.read_i32::<LittleEndian>()?
    } else {
        -1
    };

    let mut anchors = Vec::new();
    if version >= 2.3 {
        let n_anchors = r.read_i32::<LittleEndian>()?;
        if n_anchors > 0 {
            anchors.reserve(n_anchors as usize);
            for _ in 0..n_anchors {
                let _mystery = r.read_i32::<LittleEndian>()?;
                let x = r.read_i32::<LittleEndian>()?;
                let y = r.read_i32::<LittleEndian>()?;
                let attr = r.read_i32::<LittleEndian>()?;
                anchors.push(Anchor { x, y, attr });
            }
        }
    }

    Ok(Frame {
        layers,
        event_id,
        anchors,
    })
}

fn read_layer(r: &mut Cursor<&[u8]>, version: f32) -> Result<Layer, ActError> {
    let x = r.read_i32::<LittleEndian>()?;
    let y = r.read_i32::<LittleEndian>()?;
    let spr_no = r.read_i32::<LittleEndian>()?;
    let flags = r.read_u32::<LittleEndian>()?;
    let mirror = (flags & 1) != 0;

    let mut color = [255u8; 4];
    let mut scale_x = 1.0f32;
    let mut scale_y = 1.0f32;
    let mut rotation = 0i32;
    let mut spr_type = 0i32;
    let mut width = 0i32;
    let mut height = 0i32;

    if version >= 2.0 {
        r.read_exact(&mut color)?;
        scale_x = r.read_f32::<LittleEndian>()?;
        if version < 2.4 {
            scale_y = scale_x;
        } else {
            scale_y = r.read_f32::<LittleEndian>()?;
        }
        rotation = r.read_i32::<LittleEndian>()?;
        spr_type = r.read_i32::<LittleEndian>()?;
    }

    if version >= 2.5 {
        width = r.read_i32::<LittleEndian>()?;
        height = r.read_i32::<LittleEndian>()?;
    }

    Ok(Layer {
        x,
        y,
        spr_no,
        mirror,
        color,
        scale_x,
        scale_y,
        rotation,
        spr_type,
        width,
        height,
    })
}
