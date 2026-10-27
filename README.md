# Sprite Editor — Ragnarok ACT/SPR

Desktop editor to compose and export Ragnarok Online spritesheets.

Stack: Rust, eframe/egui, `.spr` / `.act` / `.pal` parsers.

## Requirements

- Recent Rust toolchain (edition 2024)
- A `data/` folder extracted from a RO client GRF

## Run

```bash
cargo run --release
```

Binary name: `ragnarok-spritesheet-generator`.

## Usage

1. Open a `data` folder or a `.spr` / `.act` pair.
2. Pick Body, Head, Weapons, Headgears, and palettes.
3. Preview actions and frames.
4. Export a PNG spritesheet.

**Random** picks a random body + head combo.

Side tabs: Folder, Body, Head, Weapons, Headgears, Body Pal, Head Pal, Sprite.

## Expected data layout

The scanner looks for Korean and ANSI paths, for example:

- Body: `인간족/몸통` or `ÀÎ°£Á·/¸öÅë`
- Head: `인간족/머리통`
- Headgears: `악세사리`
- Body / head palettes

See [folders.md](folders.md) for folder and class details.

## `src/` layout

| Module | Role |
|--------|------|
| `main.rs` | eframe / wgpu entry |
| `app.rs` | UI state and async jobs |
| `ui/` | egui panels |
| `strings.rs` | User-facing English strings |
| `spr.rs` / `act.rs` / `pal.rs` | Binary parsers |
| `scan.rs` | RO folder walk |
| `names.rs` / `jobs.rs` | Class names, UI jobs |
| `palette.rs` | `.pal` index and filters |
| `compose.rs` | Body + attach composition |
| `render.rs` | Frame and spritesheet render |
| `lib.rs` | Library crate |

## Windows notes

On Windows, wgpu prefers DX12, then Vulkan.

This avoids Vulkan crashes on Intel UHD.

## Repo notes

- `data/`, `*.grf`, and `target/` are gitignored.
- Do not commit GRF files.

## License

Distributed under [MIT](LICENSE).
