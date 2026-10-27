# Sprite Editor — Ragnarok ACT/SPR

Éditeur desktop pour composer et exporter des spritesheets Ragnarok Online.

Stack : Rust, eframe/egui, parsers `.spr` / `.act` / `.pal`.

## Prérequis

- Rust toolchain récente (edition 2024)
- Dossier `data/` extrait d’un GRF client RO

## Lancer

```bash
cargo run --release
```

Le binaire s’appelle `sprite-editor`.

## Usage

1. Ouvrir un dossier `data` ou une paire `.spr` / `.act`.
2. Choisir Body, Head, Weapons, Headgears, palettes.
3. Prévisualiser actions et frames.
4. Exporter un spritesheet PNG.

Bouton **Random** : combo body + head aléatoire.

Onglets latéraux : Dossier, Body, Head, Weapons, Headgears, Body Pal, Head Pal, Sprite.

## Données attendues

Le scanner cherche les chemins coréens et ANSI, par ex. :

- Body : `인간족/몸통` ou `ÀÎ°£Á·/¸öÅë`
- Head : `인간족/머리통`
- Headgears : `악세사리`
- Palettes body / head

Voir `folders.MD` pour le détail des dossiers et classes.

## Structure `src/`

| Module | Rôle |
|--------|------|
| `main.rs` | Entrée eframe / wgpu |
| `app.rs` | État UI et jobs async |
| `ui/` | Panneaux egui |
| `spr.rs` / `act.rs` / `pal.rs` | Parsers binaires |
| `scan.rs` | Parcours dossiers RO |
| `names.rs` / `jobs.rs` | Noms de classes, jobs UI |
| `palette.rs` | Index et filtres `.pal` |
| `compose.rs` | Composition body + attaches |
| `render.rs` | Rendu frames et spritesheet |
| `lib.rs` | Crate bibliothèque |

## Notes Windows

Sous Windows, wgpu privilégie DX12 puis Vulkan.

Cela évite des plantages Vulkan sur Intel UHD.

## Notes dépôt

- `data/`, `*.grf` et `target/` sont ignorés par git.
- Les GRF ne doivent pas être versionnés.

## Licence

Projet privé. Aucune licence publique pour l’instant.
