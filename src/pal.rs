//! Ragnarok `.pal` files (256 × BGRA, 1024 bytes).

use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PalError {
    #[error("invalid size: {0} (expected 1024)")]
    BadSize(usize),
    #[error("read: {0}")]
    Io(#[from] std::io::Error),
}

pub fn parse_palette(data: &[u8]) -> Result<[[u8; 4]; 256], PalError> {
    if data.len() != 1024 {
        return Err(PalError::BadSize(data.len()));
    }
    let mut palette = [[0u8; 4]; 256];
    for (i, entry) in palette.iter_mut().enumerate() {
        let o = i * 4;
        entry.copy_from_slice(&data[o..o + 4]);
    }
    Ok(palette)
}

pub fn load_palette(path: &Path) -> Result<[[u8; 4]; 256], PalError> {
    let data = fs::read(path)?;
    parse_palette(&data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;
    use std::path::PathBuf;

    #[test]
    fn parse_mini_pal() {
        let pal = parse_palette(&fixtures::mini_pal_bytes()).expect("pal");
        assert_eq!(pal[1], [0, 255, 0, 0]);
    }

    #[test]
    fn reject_bad_size() {
        assert!(matches!(
            parse_palette(&[0u8; 10]),
            Err(PalError::BadSize(10))
        ));
    }

    #[test]
    fn load_pal_file() {
        let dir = tempfile::tempdir().expect("tmp");
        let path = dir.path().join("t.pal");
        fixtures::write_pal(&path).expect("write");
        let pal = load_palette(&path).expect("load");
        assert_eq!(pal[1], [0, 255, 0, 0]);
    }

    #[test]
    #[ignore = "requires local data/"]
    fn load_novice_pal() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/palette/몸/초보자_남_0.pal");
        let pal = load_palette(&path).expect("pal");
        assert_ne!(pal[0], [0, 0, 0, 0]);
    }
}
