use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SprError {
    #[error("signature invalide (attendu SP)")]
    BadSignature,
    #[error("version SPR non supportée: {0}")]
    UnsupportedVersion(f32),
    #[error("lecture: {0}")]
    Io(#[from] std::io::Error),
    #[error("données RLE invalides")]
    BadRle,
}

#[derive(Debug, Clone)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SprFile {
    pub version: f32,
    pub palette: [[u8; 4]; 256],
    original_palette: [[u8; 4]; 256],
    indexed: Vec<(u16, u16, Vec<u8>)>,
    pub images: Vec<RgbaImage>,
    pub palette_count: usize,
    pub rgba_count: usize,
}

impl SprFile {
    pub fn open(path: &std::path::Path) -> Result<Self, SprError> {
        let data = std::fs::read(path)?;
        Self::parse(&data)
    }

    pub fn parse(data: &[u8]) -> Result<Self, SprError> {
        let mut r = Cursor::new(data);

        let mut sig = [0u8; 2];
        r.read_exact(&mut sig)?;
        if &sig != b"SP" {
            return Err(SprError::BadSignature);
        }

        let minor = r.read_u8()? as f32;
        let major = r.read_u8()? as f32;
        let version = major + minor / 10.0;
        if !(version == 1.1 || (2.0..=2.1).contains(&version)) {
            return Err(SprError::UnsupportedVersion(version));
        }

        let bmp_count = r.read_u16::<LittleEndian>()? as usize;
        let tga_count = if version > 1.1 {
            r.read_u16::<LittleEndian>()? as usize
        } else {
            0
        };

        let mut indexed: Vec<(u16, u16, Vec<u8>)> = Vec::with_capacity(bmp_count);
        if version >= 2.1 {
            for _ in 0..bmp_count {
                let w = r.read_u16::<LittleEndian>()?;
                let h = r.read_u16::<LittleEndian>()?;
                let compressed_len = r.read_u16::<LittleEndian>()? as usize;
                let mut compressed = vec![0u8; compressed_len];
                r.read_exact(&mut compressed)?;
                let indices = if w == 0 || h == 0 {
                    vec![0u8]
                } else {
                    decompress_rle(&compressed, (w as usize) * (h as usize))?
                };
                indexed.push((w, h, indices));
            }
        } else {
            for _ in 0..bmp_count {
                let w = r.read_u16::<LittleEndian>()?;
                let h = r.read_u16::<LittleEndian>()?;
                let len = (w as usize).saturating_mul(h as usize).max(1);
                let mut indices = vec![0u8; len];
                r.read_exact(&mut indices)?;
                indexed.push((w, h, indices));
            }
        }

        let mut truecolor: Vec<(u16, u16, Vec<u8>)> = Vec::with_capacity(tga_count);
        for _ in 0..tga_count {
            let w = r.read_u16::<LittleEndian>()?;
            let h = r.read_u16::<LittleEndian>()?;
            let len = (w as usize).saturating_mul(h as usize).saturating_mul(4);
            let mut abgr = vec![0u8; len];
            r.read_exact(&mut abgr)?;
            truecolor.push((w, h, abgr));
        }

        let mut palette = [[0u8; 4]; 256];
        for entry in &mut palette {
            r.read_exact(entry)?;
        }

        let mut images = Vec::with_capacity(bmp_count + tga_count);
        for (w, h, indices) in &indexed {
            images.push(indexed_to_rgba(*w, *h, indices, &palette));
        }
        for (w, h, abgr) in truecolor {
            images.push(abgr_to_rgba(w, h, &abgr));
        }

        Ok(Self {
            version,
            palette,
            original_palette: palette,
            indexed,
            images,
            palette_count: bmp_count,
            rgba_count: tga_count,
        })
    }

    pub fn apply_palette(&mut self, palette: [[u8; 4]; 256]) {
        self.palette = palette;
        for (i, (w, h, indices)) in self.indexed.iter().enumerate() {
            if let Some(img) = self.images.get_mut(i) {
                *img = indexed_to_rgba(*w, *h, indices, &palette);
            }
        }
    }

    pub fn reset_palette(&mut self) {
        self.apply_palette(self.original_palette);
    }

    pub fn get(&self, index: i32, image_type: i32) -> Option<&RgbaImage> {
        if index < 0 {
            return None;
        }
        let idx = index as usize;
        match image_type {
            0 => self.images.get(idx),
            1 => self.images.get(self.palette_count + idx),
            _ => None,
        }
    }
}

fn decompress_rle(compressed: &[u8], expected: usize) -> Result<Vec<u8>, SprError> {
    let mut out = Vec::with_capacity(expected);
    let mut i = 0;
    while i < compressed.len() {
        let b = compressed[i];
        i += 1;
        if b == 0 {
            if i >= compressed.len() {
                return Err(SprError::BadRle);
            }
            let count = compressed[i] as usize;
            i += 1;
            if count == 0 {
                return Err(SprError::BadRle);
            }
            out.extend(std::iter::repeat_n(0u8, count));
        } else {
            out.push(b);
        }
    }
    if out.len() < expected {
        out.resize(expected, 0);
    } else if out.len() > expected {
        out.truncate(expected);
    }
    Ok(out)
}

fn indexed_to_rgba(w: u16, h: u16, indices: &[u8], palette: &[[u8; 4]; 256]) -> RgbaImage {
    let width = if w == 0 { 1 } else { w as u32 };
    let height = if h == 0 { 1 } else { h as u32 };
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    for (i, &idx) in indices.iter().enumerate().take((width * height) as usize) {
        let c = palette[idx as usize];
        let o = i * 4;
        if idx == 0 {
            pixels[o..o + 4].copy_from_slice(&[0, 0, 0, 0]);
        } else {
            pixels[o] = c[0];
            pixels[o + 1] = c[1];
            pixels[o + 2] = c[2];
            pixels[o + 3] = 255;
        }
    }
    RgbaImage {
        width,
        height,
        pixels,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;
    use std::path::PathBuf;

    #[test]
    fn parse_mini_spr() {
        let spr = SprFile::parse(&fixtures::mini_spr_bytes()).expect("spr");
        assert!((spr.version - 1.1).abs() < 0.01);
        assert_eq!(spr.palette_count, 1);
        assert_eq!(spr.rgba_count, 0);
        assert_eq!(spr.images.len(), 1);
        assert_eq!(spr.images[0].width, 2);
        assert_eq!(spr.images[0].height, 2);
        // index 1 → opaque red
        assert_eq!(&spr.images[0].pixels[0..4], &[255, 0, 0, 255]);
    }

    #[test]
    fn apply_and_reset_palette() {
        let mut spr = SprFile::parse(&fixtures::mini_spr_bytes()).expect("spr");
        let mut pal = spr.palette;
        pal[1] = [0, 255, 0, 0];
        spr.apply_palette(pal);
        assert_eq!(&spr.images[0].pixels[0..4], &[0, 255, 0, 255]);
        spr.reset_palette();
        assert_eq!(&spr.images[0].pixels[0..4], &[255, 0, 0, 255]);
    }

    #[test]
    #[ignore = "requires local data/"]
    fn parse_aira_spr() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/sprite/npc/4_aira.spr");
        let spr = SprFile::open(&path).expect("spr");
        assert!((spr.version - 2.1).abs() < 0.01);
        assert!(!spr.images.is_empty());
        assert_eq!(spr.images.len(), spr.palette_count + spr.rgba_count);
    }
}

fn abgr_to_rgba(w: u16, h: u16, abgr: &[u8]) -> RgbaImage {
    let width = if w == 0 { 1 } else { w as u32 };
    let height = if h == 0 { 1 } else { h as u32 };
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    // TGA segment is stored bottom-up (ABGR).
    for y in 0..height {
        let src_y = height - 1 - y;
        for x in 0..width {
            let si = ((src_y * width + x) * 4) as usize;
            let di = ((y * width + x) * 4) as usize;
            if si + 3 >= abgr.len() {
                break;
            }
            pixels[di] = abgr[si + 3]; // R
            pixels[di + 1] = abgr[si + 2]; // G
            pixels[di + 2] = abgr[si + 1]; // B
            pixels[di + 3] = abgr[si]; // A
        }
    }
    RgbaImage {
        width,
        height,
        pixels,
    }
}
