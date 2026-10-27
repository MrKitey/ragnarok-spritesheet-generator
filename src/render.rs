use crate::act::{ActFile, Anchor, Frame};
use crate::spr::{RgbaImage, SprFile};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct RenderedFrame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    /// Ancres dans l'espace image (origine = coin haut-gauche du canvas).
    pub anchors: Vec<(f32, f32)>,
    /// Première ancre body (avec delta), espace image. Pour édition drag.
    pub body_anchor: Option<(f32, f32)>,
    pub origin_x: f32,
    pub origin_y: f32,
}

#[derive(Clone, Copy)]
pub enum AttachMode {
    /// Align sprite origins (weapons).
    Origin,
    /// Align first anchors (head / headgears).
    Anchor,
}

pub struct AttachLayer<'a> {
    pub spr: &'a SprFile,
    pub frame: &'a Frame,
    pub mode: AttachMode,
}

pub fn render_frame(
    spr: &SprFile,
    frame: &Frame,
    attaches: &[AttachLayer<'_>],
    padding: i32,
    base_anchor_delta: (i32, i32),
) -> RenderedFrame {
    let mut layers: Vec<BlitLayer> = Vec::new();

    collect_layers(spr, frame, 0.0, 0.0, &mut layers);

    for attach in attaches {
        let (dx, dy) = match attach.mode {
            AttachMode::Origin => (0.0, 0.0),
            AttachMode::Anchor => anchor_offset(frame, attach.frame, base_anchor_delta),
        };
        collect_layers(attach.spr, attach.frame, dx, dy, &mut layers);
    }

    let body_world = body_anchor_world(frame, base_anchor_delta);

    if layers.is_empty() {
        return RenderedFrame {
            width: 64,
            height: 64,
            pixels: vec![0; 64 * 64 * 4],
            anchors: Vec::new(),
            body_anchor: Some((body_world.0 + 32.0, body_world.1 + 32.0)),
            origin_x: 32.0,
            origin_y: 32.0,
        };
    }

    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for layer in &layers {
        min_x = min_x.min(layer.min_x);
        min_y = min_y.min(layer.min_y);
        max_x = max_x.max(layer.max_x);
        max_y = max_y.max(layer.max_y);
    }

    // Inclure les ancres dans le bounding box (body avec delta).
    let mut anchor_world: Vec<(f32, f32)> = frame
        .anchors
        .iter()
        .enumerate()
        .map(|(i, a)| {
            if i == 0 {
                body_world
            } else {
                (a.x as f32, a.y as f32)
            }
        })
        .collect();
    if frame.anchors.is_empty() {
        anchor_world.push(body_world);
    }
    for attach in attaches {
        let (dx, dy) = match attach.mode {
            AttachMode::Origin => (0.0, 0.0),
            AttachMode::Anchor => anchor_offset(frame, attach.frame, base_anchor_delta),
        };
        for a in &attach.frame.anchors {
            anchor_world.push((a.x as f32 + dx, a.y as f32 + dy));
        }
    }
    for &(ax, ay) in &anchor_world {
        min_x = min_x.min(ax);
        min_y = min_y.min(ay);
        max_x = max_x.max(ax);
        max_y = max_y.max(ay);
    }

    let pad = padding as f32;
    min_x = min_x.floor() - pad;
    min_y = min_y.floor() - pad;
    max_x = max_x.ceil() + pad;
    max_y = max_y.ceil() + pad;

    let width = ((max_x - min_x).max(1.0)) as u32;
    let height = ((max_y - min_y).max(1.0)) as u32;
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    for layer in &layers {
        blit(&mut pixels, width, height, layer, -min_x, -min_y);
    }

    let anchors: Vec<(f32, f32)> = anchor_world
        .into_iter()
        .map(|(ax, ay)| (ax - min_x, ay - min_y))
        .collect();
    let body_anchor = Some((body_world.0 - min_x, body_world.1 - min_y));

    RenderedFrame {
        width,
        height,
        pixels,
        anchors,
        body_anchor,
        origin_x: -min_x,
        origin_y: -min_y,
    }
}

pub fn action_frame<'a>(act: &'a ActFile, action: usize, frame: usize) -> Option<&'a Frame> {
    act.actions.get(action)?.frames.get(frame)
}

fn fallback_frame<'a>(act: &'a ActFile, action: usize, frame: usize) -> Option<&'a Frame> {
    action_frame(act, action, frame)
        .or_else(|| action_frame(act, action, 0))
        .or_else(|| action_frame(act, 0, 0))
}

/// Build a spritesheet: one action per row, frames left to right.
/// Cells are origin-aligned. Extra layers use the same fallback as the preview.
pub fn build_spritesheet(
    base_spr: &SprFile,
    base_act: &ActFile,
    extras: &[(&SprFile, &ActFile, AttachMode)],
    padding: i32,
    anchor_delta_for: &dyn Fn(usize) -> (i32, i32),
) -> Result<RenderedFrame, String> {
    let n_actions = base_act.actions.len();
    if n_actions == 0 {
        return Err("Aucune action a exporter".into());
    }

    let max_frames = base_act
        .actions
        .iter()
        .map(|a| a.frames.len())
        .max()
        .unwrap_or(0);
    if max_frames == 0 {
        return Err("Aucune frame a exporter".into());
    }

    let mut rendered: Vec<Vec<Option<RenderedFrame>>> = Vec::with_capacity(n_actions);
    let mut max_left = 0.0f32;
    let mut max_right = 0.0f32;
    let mut max_top = 0.0f32;
    let mut max_bottom = 0.0f32;

    for action_idx in 0..n_actions {
        let n = base_act.actions[action_idx].frames.len();
        let mut row = Vec::with_capacity(max_frames);
        for frame_idx in 0..max_frames {
            if frame_idx >= n {
                row.push(None);
                continue;
            }
            let Some(frame) = action_frame(base_act, action_idx, frame_idx) else {
                row.push(None);
                continue;
            };
            let mut attaches = Vec::new();
            for &(spr, act, mode) in extras {
                if let Some(af) = fallback_frame(act, action_idx, frame_idx) {
                    attaches.push(AttachLayer {
                        spr,
                        frame: af,
                        mode,
                    });
                }
            }
            let delta = anchor_delta_for(action_idx);
            let rf = render_frame(base_spr, frame, &attaches, padding, delta);
            max_left = max_left.max(rf.origin_x);
            max_right = max_right.max(rf.width as f32 - rf.origin_x);
            max_top = max_top.max(rf.origin_y);
            max_bottom = max_bottom.max(rf.height as f32 - rf.origin_y);
            row.push(Some(rf));
        }
        rendered.push(row);
    }

    let cell_w = (max_left + max_right).ceil().max(1.0) as u32;
    let cell_h = (max_top + max_bottom).ceil().max(1.0) as u32;
    let sheet_w = cell_w.saturating_mul(max_frames as u32).max(1);
    let sheet_h = cell_h.saturating_mul(n_actions as u32).max(1);
    let mut pixels = vec![0u8; (sheet_w as usize) * (sheet_h as usize) * 4];

    for (action_idx, row) in rendered.iter().enumerate() {
        for (frame_idx, cell) in row.iter().enumerate() {
            let Some(rf) = cell else {
                continue;
            };
            let dest_x = frame_idx as u32 * cell_w;
            let dest_y = action_idx as u32 * cell_h;
            let ox = (max_left - rf.origin_x).round() as i32;
            let oy = (max_top - rf.origin_y).round() as i32;
            blit_rgba(
                &mut pixels,
                sheet_w,
                sheet_h,
                &rf.pixels,
                rf.width,
                rf.height,
                dest_x as i32 + ox,
                dest_y as i32 + oy,
            );
        }
    }

    Ok(RenderedFrame {
        width: sheet_w,
        height: sheet_h,
        pixels,
        anchors: Vec::new(),
        body_anchor: None,
        origin_x: max_left,
        origin_y: max_top,
    })
}

fn blit_rgba(
    dest: &mut [u8],
    dest_w: u32,
    dest_h: u32,
    src: &[u8],
    src_w: u32,
    src_h: u32,
    dx: i32,
    dy: i32,
) {
    for y in 0..src_h as i32 {
        let dyi = dy + y;
        if dyi < 0 || dyi >= dest_h as i32 {
            continue;
        }
        for x in 0..src_w as i32 {
            let dxi = dx + x;
            if dxi < 0 || dxi >= dest_w as i32 {
                continue;
            }
            let si = ((y as u32 * src_w + x as u32) * 4) as usize;
            let di = ((dyi as u32 * dest_w + dxi as u32) * 4) as usize;
            let sa = src[si + 3] as u16;
            if sa == 0 {
                continue;
            }
            if sa == 255 || dest[di + 3] == 0 {
                dest[di..di + 4].copy_from_slice(&src[si..si + 4]);
            } else {
                let da = dest[di + 3] as u16;
                let out_a = sa + da * (255 - sa) / 255;
                if out_a == 0 {
                    continue;
                }
                for c in 0..3 {
                    let sc = src[si + c] as u16;
                    let dc = dest[di + c] as u16;
                    dest[di + c] =
                        ((sc * sa + dc * da * (255 - sa) / 255) / out_a).min(255) as u8;
                }
                dest[di + 3] = out_a.min(255) as u8;
            }
        }
    }
}

fn body_anchor_world(base: &Frame, delta: (i32, i32)) -> (f32, f32) {
    let b = base.anchors.first().unwrap_or(&Anchor {
        x: 0,
        y: 0,
        attr: 0,
    });
    ((b.x + delta.0) as f32, (b.y + delta.1) as f32)
}

fn anchor_offset(base: &Frame, attach: &Frame, delta: (i32, i32)) -> (f32, f32) {
    let (bx, by) = body_anchor_world(base, delta);
    let a = attach.anchors.first().unwrap_or(&Anchor {
        x: 0,
        y: 0,
        attr: 0,
    });
    (bx - a.x as f32, by - a.y as f32)
}

struct BlitLayer<'a> {
    pixels: Cow<'a, [u8]>,
    width: u32,
    height: u32,
    // Bounding box in world space (sprite local after transform).
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    // Transformed pixel grid already baked into `pixels` at origin (0,0),
    // with world offset stored here.
    world_x: f32,
    world_y: f32,
}

fn collect_layers<'a>(
    spr: &'a SprFile,
    frame: &Frame,
    ox: f32,
    oy: f32,
    out: &mut Vec<BlitLayer<'a>>,
) {
    for layer in &frame.layers {
        if layer.spr_no < 0 {
            continue;
        }
        let Some(img) = spr.get(layer.spr_no, layer.spr_type) else {
            continue;
        };
        if let Some(blit) = prepare_layer(img, layer, ox, oy) {
            out.push(blit);
        }
    }
}

fn prepare_layer<'a>(
    img: &'a RgbaImage,
    layer: &crate::act::Layer,
    ox: f32,
    oy: f32,
) -> Option<BlitLayer<'a>> {
    let sx = if layer.scale_x.abs() < 1e-6 {
        1.0
    } else {
        layer.scale_x
    };
    let sy = if layer.scale_y.abs() < 1e-6 {
        1.0
    } else {
        layer.scale_y
    };
    let no_scale = (sx - 1.0).abs() <= 1e-4 && (sy - 1.0).abs() <= 1e-4;
    let no_rotate = layer.rotation.rem_euclid(360) == 0;
    let no_tint = layer.color == [255, 255, 255, 255];

    // Hot path: borrow source pixels when no transform is needed.
    if !layer.mirror && no_tint && no_scale && no_rotate {
        let w = img.width;
        let h = img.height;
        let world_x = layer.x as f32 + ox - w as f32 * 0.5;
        let world_y = layer.y as f32 + oy - h as f32 * 0.5;
        return Some(BlitLayer {
            pixels: Cow::Borrowed(img.pixels.as_slice()),
            width: w,
            height: h,
            min_x: world_x,
            min_y: world_y,
            max_x: world_x + w as f32,
            max_y: world_y + h as f32,
            world_x,
            world_y,
        });
    }

    let mut src = img.pixels.clone();
    let mut w = img.width;
    let mut h = img.height;

    if layer.mirror {
        flip_horizontal(&mut src, w, h);
    }

    if !no_tint {
        apply_tint(&mut src, layer.color);
    }

    if !no_scale {
        let (nw, nh, scaled) = scale_nearest(&src, w, h, sx, sy);
        src = scaled;
        w = nw;
        h = nh;
    }

    if !no_rotate {
        let angle = (layer.rotation as f32).to_radians();
        let (nw, nh, rotated, min_x, min_y) = rotate_rgba(&src, w, h, angle);
        let cx = w as f32 * 0.5;
        let cy = h as f32 * 0.5;
        let world_cx = layer.x as f32 + ox;
        let world_cy = layer.y as f32 + oy;
        let world_x = world_cx - cx + min_x;
        let world_y = world_cy - cy + min_y;
        return Some(BlitLayer {
            pixels: Cow::Owned(rotated),
            width: nw,
            height: nh,
            min_x: world_x,
            min_y: world_y,
            max_x: world_x + nw as f32,
            max_y: world_y + nh as f32,
            world_x,
            world_y,
        });
    }

    let world_x = layer.x as f32 + ox - w as f32 * 0.5;
    let world_y = layer.y as f32 + oy - h as f32 * 0.5;
    Some(BlitLayer {
        pixels: Cow::Owned(src),
        width: w,
        height: h,
        min_x: world_x,
        min_y: world_y,
        max_x: world_x + w as f32,
        max_y: world_y + h as f32,
        world_x,
        world_y,
    })
}

fn flip_horizontal(pixels: &mut [u8], w: u32, h: u32) {
    for y in 0..h {
        for x in 0..(w / 2) {
            let a = ((y * w + x) * 4) as usize;
            let b = ((y * w + (w - 1 - x)) * 4) as usize;
            for c in 0..4 {
                pixels.swap(a + c, b + c);
            }
        }
    }
}

fn apply_tint(pixels: &mut [u8], color: [u8; 4]) {
    for px in pixels.chunks_exact_mut(4) {
        px[0] = ((px[0] as u16 * color[0] as u16) / 255) as u8;
        px[1] = ((px[1] as u16 * color[1] as u16) / 255) as u8;
        px[2] = ((px[2] as u16 * color[2] as u16) / 255) as u8;
        px[3] = ((px[3] as u16 * color[3] as u16) / 255) as u8;
    }
}

fn scale_nearest(src: &[u8], w: u32, h: u32, sx: f32, sy: f32) -> (u32, u32, Vec<u8>) {
    let nw = ((w as f32 * sx.abs()).round().max(1.0)) as u32;
    let nh = ((h as f32 * sy.abs()).round().max(1.0)) as u32;
    let mut out = vec![0u8; (nw * nh * 4) as usize];
    for y in 0..nh {
        let syi = ((y as f32 / nh as f32) * h as f32).floor() as u32;
        let syi = syi.min(h - 1);
        for x in 0..nw {
            let sxi = ((x as f32 / nw as f32) * w as f32).floor() as u32;
            let sxi = sxi.min(w - 1);
            let si = ((syi * w + sxi) * 4) as usize;
            let di = ((y * nw + x) * 4) as usize;
            out[di..di + 4].copy_from_slice(&src[si..si + 4]);
        }
    }
    (nw, nh, out)
}

fn rotate_rgba(src: &[u8], w: u32, h: u32, angle: f32) -> (u32, u32, Vec<u8>, f32, f32) {
    let cos = angle.cos();
    let sin = angle.sin();
    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;

    let corners = [
        rotate_point(0.0, 0.0, cx, cy, cos, sin),
        rotate_point(w as f32, 0.0, cx, cy, cos, sin),
        rotate_point(0.0, h as f32, cx, cy, cos, sin),
        rotate_point(w as f32, h as f32, cx, cy, cos, sin),
    ];
    let min_x = corners.iter().map(|p| p.0).fold(f32::INFINITY, f32::min);
    let min_y = corners.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
    let max_x = corners
        .iter()
        .map(|p| p.0)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = corners
        .iter()
        .map(|p| p.1)
        .fold(f32::NEG_INFINITY, f32::max);

    let nw = ((max_x - min_x).ceil().max(1.0)) as u32;
    let nh = ((max_y - min_y).ceil().max(1.0)) as u32;
    let mut out = vec![0u8; (nw * nh * 4) as usize];

    let inv_cos = cos;
    let inv_sin = -sin;
    for y in 0..nh {
        for x in 0..nw {
            let dx = x as f32 + min_x;
            let dy = y as f32 + min_y;
            let sx = inv_cos * (dx - cx) - inv_sin * (dy - cy) + cx;
            let sy = inv_sin * (dx - cx) + inv_cos * (dy - cy) + cy;
            if sx < 0.0 || sy < 0.0 || sx >= w as f32 || sy >= h as f32 {
                continue;
            }
            let sxi = sx.floor() as u32;
            let syi = sy.floor() as u32;
            if sxi >= w || syi >= h {
                continue;
            }
            let si = ((syi * w + sxi) * 4) as usize;
            let di = ((y * nw + x) * 4) as usize;
            out[di..di + 4].copy_from_slice(&src[si..si + 4]);
        }
    }

    (nw, nh, out, min_x, min_y)
}

fn rotate_point(x: f32, y: f32, cx: f32, cy: f32, cos: f32, sin: f32) -> (f32, f32) {
    let dx = x - cx;
    let dy = y - cy;
    (cos * dx - sin * dy + cx, sin * dx + cos * dy + cy)
}

fn blit(
    dst: &mut [u8],
    dst_w: u32,
    dst_h: u32,
    layer: &BlitLayer<'_>,
    offset_x: f32,
    offset_y: f32,
) {
    let ox = (layer.world_x + offset_x).round() as i32;
    let oy = (layer.world_y + offset_y).round() as i32;
    for y in 0..layer.height {
        let dy = oy + y as i32;
        if dy < 0 || dy as u32 >= dst_h {
            continue;
        }
        for x in 0..layer.width {
            let dx = ox + x as i32;
            if dx < 0 || dx as u32 >= dst_w {
                continue;
            }
            let si = ((y * layer.width + x) * 4) as usize;
            let di = ((dy as u32 * dst_w + dx as u32) * 4) as usize;
            let sa = layer.pixels[si + 3] as u32;
            if sa == 0 {
                continue;
            }
            if sa == 255 {
                dst[di..di + 4].copy_from_slice(&layer.pixels[si..si + 4]);
                continue;
            }
            let inv = 255 - sa;
            for c in 0..3 {
                let s = layer.pixels[si + c] as u32;
                let d = dst[di + c] as u32;
                dst[di + c] = ((s * sa + d * inv) / 255) as u8;
            }
            let da = dst[di + 3] as u32;
            dst[di + 3] = (sa + (da * inv) / 255).min(255) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::act::ActFile;
    use crate::fixtures;
    use crate::spr::SprFile;

    fn mini_pair() -> (SprFile, ActFile) {
        let spr = SprFile::parse(&fixtures::mini_spr_bytes()).expect("spr");
        let act = ActFile::parse(&fixtures::mini_act_bytes()).expect("act");
        (spr, act)
    }

    #[test]
    fn render_single_layer() {
        let (spr, act) = mini_pair();
        let frame = action_frame(&act, 0, 0).expect("frame");
        let rf = render_frame(&spr, frame, &[], 2, (0, 0));
        assert!(rf.width >= 2);
        assert!(rf.height >= 2);
        assert!(!rf.pixels.is_empty());
        assert!(rf.pixels.iter().any(|&b| b != 0));
    }

    #[test]
    fn render_with_origin_attach() {
        let (spr, act) = mini_pair();
        let frame = action_frame(&act, 0, 0).expect("frame");
        let attaches = [AttachLayer {
            spr: &spr,
            frame,
            mode: AttachMode::Origin,
        }];
        let rf = render_frame(&spr, frame, &attaches, 2, (0, 0));
        assert!(rf.width >= 2);
        assert!(rf.height >= 2);
    }

    #[test]
    fn render_with_anchor_attach() {
        let spr = SprFile::parse(&fixtures::mini_spr_bytes()).expect("spr");
        let act = ActFile::parse(&fixtures::mini_act_bytes_with_anchor(Some((0, 0)))).expect("act");
        let frame = action_frame(&act, 0, 0).expect("frame");
        assert!(!frame.anchors.is_empty());
        let attaches = [AttachLayer {
            spr: &spr,
            frame,
            mode: AttachMode::Anchor,
        }];
        let rf = render_frame(&spr, frame, &attaches, 2, (0, 0));
        assert!(rf.width >= 2);
    }

    #[test]
    fn render_anchor_delta_moves_body_anchor() {
        let spr = SprFile::parse(&fixtures::mini_spr_bytes()).expect("spr");
        let act = ActFile::parse(&fixtures::mini_act_bytes_with_anchor(Some((0, 0)))).expect("act");
        let frame = action_frame(&act, 0, 0).expect("frame");
        let a = render_frame(&spr, frame, &[], 2, (0, 0));
        let b = render_frame(&spr, frame, &[], 2, (5, -3));
        let pa = a.body_anchor.expect("a");
        let pb = b.body_anchor.expect("b");
        // World space = image coords relative to sprite origin.
        let wa = (pa.0 - a.origin_x, pa.1 - a.origin_y);
        let wb = (pb.0 - b.origin_x, pb.1 - b.origin_y);
        assert!((wb.0 - wa.0 - 5.0).abs() < 0.1);
        assert!((wb.1 - wa.1 + 3.0).abs() < 0.1);
    }

    #[test]
    fn build_sheet_one_action() {
        let (spr, act) = mini_pair();
        let sheet = build_spritesheet(&spr, &act, &[], 2, &|_| (0, 0)).expect("sheet");
        assert!(sheet.width >= 2);
        assert!(sheet.height >= 2);
        assert_eq!(sheet.pixels.len(), (sheet.width * sheet.height * 4) as usize);
    }
}
