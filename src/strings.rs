//! User-facing English strings.

// --- Window / app identity ---

pub const WINDOW_TITLE: &str = "Sprite Editor - Ragnarok ACT/SPR";
pub const APP_NAME: &str = "Sprite Editor";

// --- Toolbar ---

pub const BTN_OPEN_FOLDER: &str = "Open folder…";
pub const BTN_RANDOM: &str = "Random";
pub const CHK_ANCHORS: &str = "Anchors";
pub const CHK_ORIGIN: &str = "Origin";
pub const CHK_CHECKER: &str = "Checkerboard";
pub const SLIDER_ZOOM: &str = "Zoom";

// --- Side tabs ---

pub const TAB_FOLDER: &str = "Folder";
pub const TAB_BODY: &str = "Body";
pub const TAB_HEAD: &str = "Head";
pub const TAB_WEAPONS: &str = "Weapons";
pub const TAB_HEADGEARS: &str = "Headgears";
pub const TAB_BODY_PAL: &str = "Body Pal";
pub const TAB_HEAD_PAL: &str = "Head Pal";
pub const TAB_SPRITE: &str = "Sprite";

// --- Panels ---

pub const FOLDER_NONE: &str = "No folder open.";
pub const FOLDER_EMPTY: &str = "Empty folder.";
pub const PREVIEW_EMPTY: &str = "Load a .spr/.act pair to preview.";
pub const CLASS_SELECT_BODY: &str = "Select a body first.";
pub const CLASS_NO_SPRITE: &str = "No sprites.";
pub const CLASS_NO_WEAPON: &str = "No weapon for this body.";
pub const HEADGEAR_INDEXING: &str = "Indexing headgears…";
pub const HEADGEAR_NONE: &str = "No headgear.";
pub const PALETTE_INDEXING: &str = "Indexing palettes…";
pub const PALETTE_NONE: &str = "No palettes.";
pub const PALETTE_DEFAULT: &str = "Default";
pub const SPRITE_NONE: &str = "No sprite loaded.";
pub const BTN_EXPORT_SHEET: &str = "Export spritesheet…";
pub const BTN_PLAY: &str = "Play";
pub const BTN_PAUSE: &str = "Pause";
pub const BTN_RESET: &str = "Reset";
pub const LABEL_ACTION: &str = "Action";
pub const LABEL_FRAME: &str = "Frame";
pub const LABEL_MALE: &str = "Male";
pub const LABEL_FEMALE: &str = "Female";
pub const FILTER_PNG: &str = "PNG";

// --- Jobs ---

pub const JOB_EXPORT_SHEET: &str = "Export spritesheet…";
pub const JOB_LOAD_HEADGEAR: &str = "Loading headgear…";
pub const JOB_RANDOM: &str = "Random…";

// --- Status / prefixes (keep in sync with format helpers) ---

pub const STATUS_OPEN_HINT: &str = "Open a .spr / .act pair or a folder";
pub const STATUS_NO_EXPORT: &str = "No sprite to export";
pub const STATUS_JOB_INTERRUPTED: &str = "Job interrupted";
pub const STATUS_LOAD_BODY_BEFORE_WEAPON: &str = "Load a body before the weapon";
pub const STATUS_LOAD_BODY_BEFORE_HEADGEAR: &str = "Load a body before the headgear";
pub const STATUS_RANDOM_NO_BODY: &str = "Random: no body";
pub const STATUS_PALETTE_BODY_DEFAULT: &str = "Palette body: Default";
pub const STATUS_PALETTE_HEAD_DEFAULT: &str = "Palette head: Default";
pub const STATUS_FOLDER_PREFIX: &str = "Folder:";
pub const STATUS_PALETTES_READY_SUFFIX: &str = " (palettes ready)";
pub const FALLBACK_SPRITE_NAME: &str = "sprite";
pub const FALLBACK_PALETTE_NAME: &str = "palette";
pub const VARIANT_DEFAULT: &str = "default";

// --- Errors ---

pub const ERR_CHOOSE_SPR_ACT: &str = "Choose a .spr or .act file";
pub const ERR_SPR_MISSING_PREFIX: &str = "Missing SPR: ";
pub const ERR_ACT_MISSING_PREFIX: &str = "Missing ACT: ";
pub const ERR_NO_ACTION_EXPORT: &str = "No action to export";
pub const ERR_NO_FRAME_EXPORT: &str = "No frame to export";
pub const ERR_NO_WEAPON: &str = "No weapon";
pub const ERR_NO_HEADGEAR: &str = "No headgear";
pub const ERR_INVALID_BUFFER: &str = "invalid buffer";
pub const ERR_NO_DX12_VULKAN_PREFIX: &str = "No compatible DX12/Vulkan adapter (";
pub const ERR_NO_DX12_VULKAN_SUFFIX: &str = " found)";

pub fn status_random_err(e: impl std::fmt::Display) -> String {
    format!("Random: {e}")
}

pub fn status_export_err(e: impl std::fmt::Display) -> String {
    format!("Export: {e}")
}

pub fn status_export_ok(
    path: impl std::fmt::Display,
    width: u32,
    height: u32,
    n_actions: usize,
) -> String {
    format!("Export: {path} ({width}x{height}, {n_actions} actions)")
}

pub fn status_headgear(name: &str) -> String {
    format!("Headgear: {name}")
}

pub fn status_headgear_err(e: impl std::fmt::Display) -> String {
    format!("Headgear error: {e}")
}

pub fn status_random_ok(body: &str, head: &str) -> String {
    format!("Random: {body} + {head}")
}

pub fn status_folder(name: &str) -> String {
    format!("{STATUS_FOLDER_PREFIX} {name}")
}

pub fn status_folder_err(e: impl std::fmt::Display) -> String {
    format!("Folder error: {e}")
}

pub fn status_headgears_err(e: impl std::fmt::Display) -> String {
    format!("Headgears: {e}")
}

pub fn status_palettes_ready(base: &str) -> String {
    format!("{base}{STATUS_PALETTES_READY_SUFFIX}")
}

pub fn status_palette_body(name: &str) -> String {
    format!("Palette body: {name}")
}

pub fn status_palette_head(name: &str) -> String {
    format!("Palette head: {name}")
}

pub fn status_palette_err(e: impl std::fmt::Display) -> String {
    format!("Palette error: {e}")
}

pub fn status_body(name: &str) -> String {
    format!("Body: {name}")
}

pub fn status_body_err(e: impl std::fmt::Display) -> String {
    format!("Body error: {e}")
}

pub fn status_head(name: &str) -> String {
    format!("Head: {name}")
}

pub fn status_head_err(e: impl std::fmt::Display) -> String {
    format!("Head error: {e}")
}

pub fn status_weapon(name: &str) -> String {
    format!("Weapon: {name}")
}

pub fn status_weapon_err(e: impl std::fmt::Display) -> String {
    format!("Weapon error: {e}")
}

pub fn status_loaded(name: &str) -> String {
    format!("Loaded: {name}")
}

pub fn status_err(e: impl std::fmt::Display) -> String {
    format!("Error: {e}")
}

pub fn status_file(path: impl std::fmt::Display) -> String {
    format!("File: {path}")
}

pub fn status_anchor_origin(action: usize) -> String {
    format!("Anchor action {action}: origin")
}

pub fn status_anchor_offset(action: usize, dx: i32, dy: i32) -> String {
    format!("Anchor action {action}: ({dx}, {dy})")
}

pub fn spr_version(v: f32) -> String {
    format!("SPR v{v:.1}")
}

pub fn act_version(v: f32) -> String {
    format!("ACT v{v:.1}")
}

pub fn images_counts(pal: usize, rgba: usize) -> String {
    format!("Images: {pal} pal + {rgba} rgba")
}

pub fn actions_count(n: usize) -> String {
    format!("Actions: {n}")
}

pub fn interval_ms(ms: f32) -> String {
    format!("Interval: {ms:.0} ms")
}

pub fn layers_count(n: usize) -> String {
    format!("Layers: {n}")
}

pub fn anchors_count(n: usize) -> String {
    format!("Anchors: {n}")
}

pub fn event_label(id: i32, name: &str) -> String {
    format!("Event: {id} ({name})")
}

pub fn anchor_offset_label(action: usize) -> String {
    format!("Anchor offset (action {action})")
}

pub fn attached_head(name: &str) -> String {
    format!("Head: {name}")
}

pub fn attached_weapon(name: &str) -> String {
    format!("Weapon: {name}")
}

pub fn attached_headgear(name: &str) -> String {
    format!("Headgear: {name}")
}

pub fn missing_spr(path: impl std::fmt::Display) -> String {
    format!("{ERR_SPR_MISSING_PREFIX}{path}")
}

pub fn missing_act(path: impl std::fmt::Display) -> String {
    format!("{ERR_ACT_MISSING_PREFIX}{path}")
}

pub fn no_dx12_vulkan(count: usize) -> String {
    format!("{ERR_NO_DX12_VULKAN_PREFIX}{count}{ERR_NO_DX12_VULKAN_SUFFIX}")
}
