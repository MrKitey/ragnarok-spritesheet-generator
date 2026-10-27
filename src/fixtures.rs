//! Builders of synthetic RO bytes for unit tests.

use std::io::Write;

/// SPR v1.1: one 2×2 indexed image, palette index 1 = opaque RGB.
pub fn mini_spr_bytes() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"SP");
    out.push(1); // minor
    out.push(1); // major → 1.1
    out.extend_from_slice(&1u16.to_le_bytes()); // bmp_count
    out.extend_from_slice(&2u16.to_le_bytes()); // w
    out.extend_from_slice(&2u16.to_le_bytes()); // h
    // indices: solid color index 1
    out.extend_from_slice(&[1, 1, 1, 1]);
    for i in 0..256u16 {
        if i == 1 {
            out.extend_from_slice(&[255, 0, 0, 0]); // R,G,B,_
        } else {
            out.extend_from_slice(&[0, 0, 0, 0]);
        }
    }
    out
}

/// ACT v2.0: one action, one frame, one layer at origin using spr 0.
pub fn mini_act_bytes() -> Vec<u8> {
    mini_act_bytes_with_anchor(None)
}

/// ACT v2.5: one action/frame/layer, optional single anchor.
pub fn mini_act_bytes_with_anchor(anchor: Option<(i32, i32)>) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"AC");
    if anchor.is_some() {
        out.push(5); // minor
        out.push(2); // major → 2.5
    } else {
        out.push(0);
        out.push(2); // → 2.0
    }
    out.extend_from_slice(&1u16.to_le_bytes()); // n_actions
    out.extend_from_slice(&[0u8; 10]); // reserved

    out.extend_from_slice(&1u32.to_le_bytes()); // n_frames
    // range1 + range2
    for _ in 0..8 {
        out.extend_from_slice(&0u32.to_le_bytes());
    }
    out.extend_from_slice(&1u32.to_le_bytes()); // n_layers

    // layer
    out.extend_from_slice(&0i32.to_le_bytes()); // x
    out.extend_from_slice(&0i32.to_le_bytes()); // y
    out.extend_from_slice(&0i32.to_le_bytes()); // spr_no
    out.extend_from_slice(&0u32.to_le_bytes()); // flags
    out.extend_from_slice(&[255, 255, 255, 255]); // color
    out.extend_from_slice(&1.0f32.to_le_bytes()); // scale_x
    if anchor.is_some() {
        out.extend_from_slice(&1.0f32.to_le_bytes()); // scale_y (v>=2.4)
    }
    out.extend_from_slice(&0i32.to_le_bytes()); // rotation
    out.extend_from_slice(&0i32.to_le_bytes()); // spr_type
    if anchor.is_some() {
        out.extend_from_slice(&2i32.to_le_bytes()); // width
        out.extend_from_slice(&2i32.to_le_bytes()); // height
    }

    out.extend_from_slice(&(-1i32).to_le_bytes()); // event_id

    if let Some((ax, ay)) = anchor {
        out.extend_from_slice(&1i32.to_le_bytes()); // n_anchors
        out.extend_from_slice(&0i32.to_le_bytes()); // mystery
        out.extend_from_slice(&ax.to_le_bytes());
        out.extend_from_slice(&ay.to_le_bytes());
        out.extend_from_slice(&0i32.to_le_bytes()); // attr
    }

    if anchor.is_some() {
        // events (v>=2.1)
        out.extend_from_slice(&0i32.to_le_bytes());
        // frame delay ticks (v>=2.2)
        out.extend_from_slice(&4.0f32.to_le_bytes());
    }

    out
}

/// Fixed 1024-byte `.pal` (256 × BGRA).
pub fn mini_pal_bytes() -> [u8; 1024] {
    let mut data = [0u8; 1024];
    // index 1 → green-ish BGRA
    data[4] = 0;
    data[5] = 255;
    data[6] = 0;
    data[7] = 0;
    data
}

/// Write spr+act pair under `dir` with `stem` basename.
pub fn write_pair(dir: &std::path::Path, stem: &str) -> std::io::Result<std::path::PathBuf> {
    std::fs::create_dir_all(dir)?;
    let spr_path = dir.join(format!("{stem}.spr"));
    let act_path = dir.join(format!("{stem}.act"));
    std::fs::write(&spr_path, mini_spr_bytes())?;
    std::fs::write(&act_path, mini_act_bytes())?;
    let _ = act_path;
    Ok(spr_path)
}

/// Write a palette file.
pub fn write_pal(path: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = std::fs::File::create(path)?;
    f.write_all(&mini_pal_bytes())?;
    Ok(())
}
