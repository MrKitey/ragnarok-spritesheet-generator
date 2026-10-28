use eframe::egui;
use rand::seq::IndexedRandom;
use sprite_editor::compose::{
    compose_frame, load_pair, sprite_sel_from_name, RandomResult, SpritePair, SpriteSel,
};
use sprite_editor::jobs::BusyJob;
use sprite_editor::names::{self, Gender};
use sprite_editor::pal;
use sprite_editor::palette::{
    build_palette_index, filter_body_palettes, filter_head_palettes, scan_body_palettes,
    scan_head_palettes, PaletteIndex, PaletteList,
};
use sprite_editor::render::{build_spritesheet, AttachMode, RenderedFrame};
use sprite_editor::scan::{
    collect_sprite_paths, collect_weapon_paths, expand_for_sprite, find_class_dir, scan_class_dir,
    scan_headgear_dir, scan_weapon_dir, ClassBrowser, FolderBrowser, BODY_PALETTE_SEGMENTS,
    BODY_SEGMENTS, HEADGEAR_SEGMENTS, HEAD_PALETTE_SEGMENTS, HEAD_SEGMENTS, WEAPON_ROOT_SEGMENTS,
};
use sprite_editor::strings;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SideTab {
    #[default]
    Folder,
    Body,
    Head,
    Weapons,
    Headgears,
    BodyPal,
    HeadPal,
    Sprite,
}

pub(crate) enum JobResult {
    Random(Result<RandomResult, String>),
    Export(Result<String, String>),
    LoadHeadgear {
        path: PathBuf,
        result: Result<SpritePair, String>,
    },
}

pub struct SpriteEditorApp {
    pub(crate) base: Option<SpritePair>,
    /// Head sprite (anchor-aligned).
    pub(crate) attach: Option<SpritePair>,
    pub(crate) weapon: Option<SpritePair>,
    pub(crate) headgear: Option<SpritePair>,
    pub(crate) folder: Option<FolderBrowser>,
    pub(crate) body_classes: Option<ClassBrowser>,
    pub(crate) head_classes: Option<ClassBrowser>,
    pub(crate) weapon_classes: Option<ClassBrowser>,
    pub(crate) headgear_classes: Option<ClassBrowser>,
    pub(crate) headgear_dir: Option<PathBuf>,
    pub(crate) headgear_rx: Option<Receiver<Result<ClassBrowser, String>>>,
    pub(crate) body_palette_dir: Option<PathBuf>,
    pub(crate) head_palette_dir: Option<PathBuf>,
    pub(crate) body_sel: Option<SpriteSel>,
    pub(crate) head_sel: Option<SpriteSel>,
    pub(crate) body_palettes: PaletteList,
    pub(crate) head_palettes: PaletteList,
    pub(crate) palette_index: Option<PaletteIndex>,
    pub(crate) palette_index_rx: Option<Receiver<PaletteIndex>>,
    pub(crate) side_tab: SideTab,
    pub(crate) action_idx: usize,
    pub(crate) frame_idx: usize,
    pub(crate) playing: bool,
    pub(crate) last_tick: Instant,
    pub(crate) zoom: f32,
    pub(crate) show_anchors: bool,
    pub(crate) show_origin: bool,
    pub(crate) show_checker: bool,
    pub(crate) status: String,
    pub(crate) texture: Option<egui::TextureHandle>,
    pub(crate) texture_key: String,
    pub(crate) busy: Option<BusyJob<JobResult>>,
    /// Runtime body-anchor offset per action index. Reset removes the entry.
    pub(crate) anchor_offsets: HashMap<usize, (i32, i32)>,
    pub(crate) dragging_anchor: bool,
}

impl Default for SpriteEditorApp {
    fn default() -> Self {
        Self {
            base: None,
            attach: None,
            weapon: None,
            headgear: None,
            folder: None,
            body_classes: None,
            head_classes: None,
            weapon_classes: None,
            headgear_classes: None,
            headgear_dir: None,
            headgear_rx: None,
            body_palette_dir: None,
            head_palette_dir: None,
            body_sel: None,
            head_sel: None,
            body_palettes: PaletteList::default(),
            head_palettes: PaletteList::default(),
            palette_index: None,
            palette_index_rx: None,
            side_tab: SideTab::Folder,
            action_idx: 0,
            frame_idx: 0,
            playing: false,
            last_tick: Instant::now(),
            zoom: 2.0,
            show_anchors: false,
            show_origin: false,
            show_checker: true,
            status: strings::STATUS_OPEN_HINT.into(),
            texture: None,
            texture_key: String::new(),
            busy: None,
            anchor_offsets: HashMap::new(),
            dragging_anchor: false,
        }
    }
}

impl eframe::App for SpriteEditorApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_palette_index();
        self.poll_headgear_index();
        self.poll_busy();
        self.tick_animation();
        if self.playing
            || self.busy.is_some()
            || self.palette_index_rx.is_some()
            || self.headgear_rx.is_some()
            || self.dragging_anchor
        {
            ctx.request_repaint();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let busy = self.busy.is_some();

        egui::Panel::top("top").show(ui, |ui| {
            ui.add_enabled_ui(!busy, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(strings::BTN_OPEN_FOLDER).clicked() {
                        self.open_folder();
                    }
                    if self.data_ready() && ui.button(strings::BTN_RANDOM).clicked() {
                        self.load_random();
                    }
                    if ui
                        .add_enabled(self.has_selection(), egui::Button::new(strings::BTN_RESET_ALL))
                        .clicked()
                    {
                        self.reset_selection();
                    }
                    ui.separator();
                    ui.checkbox(&mut self.show_anchors, strings::CHK_ANCHORS);
                    ui.checkbox(&mut self.show_origin, strings::CHK_ORIGIN);
                    ui.checkbox(&mut self.show_checker, strings::CHK_CHECKER);
                    ui.add(egui::Slider::new(&mut self.zoom, 0.5..=8.0).text(strings::SLIDER_ZOOM));
                });
            });
            ui.label(&self.status);
        });

        if self.has_side_content() {
            egui::Panel::left("side_tabs")
                .default_size(300.0)
                .min_size(240.0)
                .show(ui, |ui| {
                    ui.add_enabled_ui(!busy, |ui| {
                        self.draw_side_tabs(ui);
                    });
                });
        }

        egui::CentralPanel::default().show(ui, |ui| {
            self.draw_preview(ui);
        });

        if busy {
            self.draw_busy_overlay(ui.ctx());
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClassKind {
    Body,
    Head,
    Weapon,
    Headgear,
}

pub(crate) enum FolderAction {
    Toggle(PathBuf),
    Open(PathBuf),
}

pub(crate) enum ClassAction {
    ToggleClass(String),
    ToggleGender((String, bool)),
    Open(PathBuf),
    Navigate(i32),
}

pub(crate) enum PaletteAction {
    Reset,
    Apply(PathBuf),
    Navigate(i32),
}

impl SpriteEditorApp {
    pub(crate) fn has_side_content(&self) -> bool {
        self.folder.is_some()
            || self.body_classes.is_some()
            || self.head_classes.is_some()
            || self.weapon_classes.is_some()
            || self.headgear_dir.is_some()
            || self.base.is_some()
            || (self.body_palette_dir.is_some() && self.body_sel.is_some())
            || (self.head_palette_dir.is_some() && self.head_sel.is_some())
    }

    pub(crate) fn tab_available(&self, tab: SideTab) -> bool {
        match tab {
            SideTab::Folder => self.folder.is_some(),
            SideTab::Body => self.body_classes.is_some(),
            SideTab::Head => self.head_classes.is_some(),
            SideTab::Weapons => self.weapon_classes.is_some(),
            SideTab::Headgears => self.headgear_dir.is_some(),
            SideTab::BodyPal => self.body_palette_dir.is_some() && self.body_sel.is_some(),
            SideTab::HeadPal => self.head_palette_dir.is_some() && self.head_sel.is_some(),
            SideTab::Sprite => self.base.is_some(),
        }
    }

    pub(crate) fn ensure_side_tab(&mut self) {
        if self.tab_available(self.side_tab) {
            return;
        }
        for tab in [
            SideTab::Folder,
            SideTab::Body,
            SideTab::Head,
            SideTab::Weapons,
            SideTab::Headgears,
            SideTab::BodyPal,
            SideTab::HeadPal,
            SideTab::Sprite,
        ] {
            if self.tab_available(tab) {
                self.side_tab = tab;
                return;
            }
        }
    }

    pub(crate) fn class_browser(&self, kind: ClassKind) -> Option<&ClassBrowser> {
        match kind {
            ClassKind::Body => self.body_classes.as_ref(),
            ClassKind::Head => self.head_classes.as_ref(),
            ClassKind::Weapon => self.weapon_classes.as_ref(),
            ClassKind::Headgear => self.headgear_classes.as_ref(),
        }
    }

    pub(crate) fn class_browser_mut(&mut self, kind: ClassKind) -> Option<&mut ClassBrowser> {
        match kind {
            ClassKind::Body => self.body_classes.as_mut(),
            ClassKind::Head => self.head_classes.as_mut(),
            ClassKind::Weapon => self.weapon_classes.as_mut(),
            ClassKind::Headgear => self.headgear_classes.as_mut(),
        }
    }

    pub(crate) fn export_spritesheet(&mut self) {
        if self.busy.is_some() {
            return;
        }
        let Some(base) = &self.base else {
            self.status = strings::STATUS_NO_EXPORT.into();
            return;
        };

        let default_name = format!("{}_sheet.png", base.name);
        let Some(path) = rfd::FileDialog::new()
            .add_filter(strings::FILTER_PNG, &["png"])
            .set_file_name(&default_name)
            .save_file()
        else {
            return;
        };

        let base_spr = Arc::clone(&base.spr);
        let base_act = Arc::clone(&base.act);
        let n_actions = base.act.actions.len();
        let anchor_offsets = self.anchor_offsets.clone();
        let mut extras_owned = Vec::new();
        // Draw order: weapon (origin), head (anchor), headgear (anchor).
        if let Some(w) = &self.weapon {
            extras_owned.push((Arc::clone(&w.spr), Arc::clone(&w.act), AttachMode::Origin));
        }
        if let Some(a) = &self.attach {
            extras_owned.push((Arc::clone(&a.spr), Arc::clone(&a.act), AttachMode::Anchor));
        }
        if let Some(h) = &self.headgear {
            extras_owned.push((Arc::clone(&h.spr), Arc::clone(&h.act), AttachMode::Anchor));
        }

        let (job, tx) = BusyJob::start(strings::JOB_EXPORT_SHEET);
        self.status = job.label.clone();
        self.busy = Some(job);

        thread::spawn(move || {
            let result = (|| {
                let extras: Vec<_> = extras_owned
                    .iter()
                    .map(|(s, a, m)| (s.as_ref(), a.as_ref(), *m))
                    .collect();
                let sheet = build_spritesheet(&base_spr, &base_act, &extras, 2, &|action| {
                    anchor_offsets.get(&action).copied().unwrap_or((0, 0))
                })?;
                let width = sheet.width;
                let height = sheet.height;
                let img = image::RgbaImage::from_raw(width, height, sheet.pixels)
                    .ok_or_else(|| strings::ERR_INVALID_BUFFER.to_string())?;
                img.save(&path).map_err(|e| e.to_string())?;
                Ok(strings::status_export_ok(
                    path.display(),
                    width,
                    height,
                    n_actions,
                ))
            })();
            let _ = tx.send(JobResult::Export(result));
        });
    }

    pub(crate) fn start_job(&mut self, label: &str) -> Option<mpsc::Sender<JobResult>> {
        if self.busy.is_some() {
            return None;
        }
        let (job, tx) = BusyJob::start(label);
        self.status = job.label.clone();
        self.busy = Some(job);
        Some(tx)
    }

    fn poll_busy(&mut self) {
        let polled = match &self.busy {
            Some(job) => match job.try_poll() {
                Ok(result) => Some(Ok(result)),
                Err(mpsc::TryRecvError::Empty) => return,
                Err(mpsc::TryRecvError::Disconnected) => Some(Err(())),
            },
            None => return,
        };

        self.busy = None;
        match polled {
            Some(Ok(JobResult::Random(result))) => match result {
                Ok(ready) => self.apply_random_result(ready),
                Err(e) => self.status = strings::status_random_err(e),
            },
            Some(Ok(JobResult::Export(result))) => match result {
                Ok(msg) => self.status = msg,
                Err(e) => self.status = strings::status_export_err(e),
            },
            Some(Ok(JobResult::LoadHeadgear { path, result })) => match result {
                Ok(pair) => {
                    if let Some(b) = &mut self.headgear_classes {
                        b.selected = Some(path);
                    }
                    self.status = strings::status_headgear(&pair.name);
                    self.headgear = Some(pair);
                    self.texture_key.clear();
                }
                Err(e) => self.status = strings::status_headgear_err(e),
            },
            Some(Err(())) => {
                self.status = strings::STATUS_JOB_INTERRUPTED.into();
            }
            None => {}
        }
    }

    fn apply_random_result(&mut self, ready: RandomResult) {
        if let Some(browser) = self.body_classes.as_mut() {
            expand_for_sprite(browser, &ready.body_path);
            browser.selected = Some(ready.body_path.clone());
        }
        self.body_sel = sprite_sel_from_name(&ready.body.name);
        let body_name = ready.body.name.clone();
        self.base = Some(ready.body);
        self.action_idx = 0;
        self.frame_idx = 0;
        self.playing = false;
        self.anchor_offsets.clear();
        self.dragging_anchor = false;
        self.texture_key.clear();
        self.last_tick = Instant::now();
        self.refresh_body_palettes();
        self.body_palettes.selected = ready.body_pal;

        let head_name = if let Some((head, head_path)) = ready.head {
            if let Some(browser) = self.head_classes.as_mut() {
                expand_for_sprite(browser, &head_path);
                browser.selected = Some(head_path);
            }
            self.head_sel = sprite_sel_from_name(&head.name);
            let name = head.name.clone();
            self.attach = Some(head);
            self.refresh_head_palettes();
            self.head_palettes.selected = ready.head_pal;
            name
        } else {
            self.attach = None;
            self.head_sel = None;
            self.head_palettes = PaletteList::default();
            "-".into()
        };

        self.status = strings::status_random_ok(&body_name, &head_name);
        self.side_tab = SideTab::Sprite;
    }

    fn tick_animation(&mut self) {
        if !self.playing {
            return;
        }
        let Some(base) = &self.base else {
            return;
        };
        let Some(action) = base.act.actions.get(self.action_idx) else {
            return;
        };
        if action.frames.is_empty() {
            return;
        }
        let ms = action.frame_ms.max(1.0);
        let elapsed = self.last_tick.elapsed().as_secs_f32() * 1000.0;
        if elapsed >= ms {
            let steps = (elapsed / ms).floor() as usize;
            self.frame_idx = (self.frame_idx + steps) % action.frames.len();
            self.last_tick = Instant::now();
            self.texture_key.clear();
        }
    }

    pub(crate) fn open_folder(&mut self) {
        let Some(path) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        match FolderBrowser::open(path.clone()) {
            Ok(browser) => {
                let name = browser.root.display().to_string();
                self.status = strings::status_folder(&name);
                self.folder = Some(browser);
                self.body_classes = find_class_dir(&path, BODY_SEGMENTS)
                    .and_then(|p| scan_class_dir(&p).ok());
                self.head_classes = find_class_dir(&path, HEAD_SEGMENTS)
                    .and_then(|p| scan_class_dir(&p).ok());
                self.weapon_classes = find_class_dir(&path, WEAPON_ROOT_SEGMENTS)
                    .and_then(|p| scan_weapon_dir(&p).ok());
                self.headgear_classes = None;
                self.headgear_dir = find_class_dir(&path, HEADGEAR_SEGMENTS);
                self.headgear_rx = None;
                self.body_palette_dir = find_class_dir(&path, BODY_PALETTE_SEGMENTS);
                self.head_palette_dir = find_class_dir(&path, HEAD_PALETTE_SEGMENTS);
                self.body_sel = None;
                self.head_sel = None;
                self.body_palettes = PaletteList::default();
                self.head_palettes = PaletteList::default();
                self.palette_index = None;
                self.start_palette_index();
                self.start_headgear_index();
            }
            Err(e) => {
                self.status = strings::status_folder_err(e);
                self.body_classes = None;
                self.head_classes = None;
                self.weapon_classes = None;
                self.headgear_classes = None;
                self.headgear_dir = None;
                self.headgear_rx = None;
                self.body_palette_dir = None;
                self.head_palette_dir = None;
                self.body_sel = None;
                self.head_sel = None;
                self.body_palettes = PaletteList::default();
                self.head_palettes = PaletteList::default();
                self.palette_index = None;
                self.palette_index_rx = None;
            }
        }
    }

    fn start_headgear_index(&mut self) {
        let Some(dir) = self.headgear_dir.clone() else {
            self.headgear_rx = None;
            return;
        };
        let (tx, rx) = mpsc::channel();
        self.headgear_rx = Some(rx);
        thread::spawn(move || {
            let _ = tx.send(scan_headgear_dir(&dir));
        });
    }

    fn poll_headgear_index(&mut self) {
        let Some(rx) = &self.headgear_rx else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(browser)) => {
                self.headgear_rx = None;
                self.headgear_classes = Some(browser);
            }
            Ok(Err(e)) => {
                self.headgear_rx = None;
                self.headgear_classes = None;
                self.status = strings::status_headgears_err(e);
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                self.headgear_rx = None;
            }
        }
    }

    fn start_palette_index(&mut self) {
        let body_dir = self.body_palette_dir.clone();
        let head_dir = self.head_palette_dir.clone();
        if body_dir.is_none() && head_dir.is_none() {
            self.palette_index_rx = None;
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.palette_index_rx = Some(rx);
        thread::spawn(move || {
            let index = build_palette_index(body_dir.as_deref(), head_dir.as_deref());
            let _ = tx.send(index);
        });
    }

    fn poll_palette_index(&mut self) {
        let Some(rx) = &self.palette_index_rx else {
            return;
        };
        match rx.try_recv() {
            Ok(index) => {
                self.palette_index_rx = None;
                self.palette_index = Some(index);
                self.refresh_body_palettes();
                self.refresh_head_palettes();
                if self.status.starts_with(strings::STATUS_FOLDER_PREFIX) {
                    let next = strings::status_palettes_ready(&self.status);
                    self.status = next;
                }
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                self.palette_index_rx = None;
            }
        }
    }

    pub(crate) fn navigate_palettes(&mut self, kind: ClassKind, delta: i32) {
        let list = match kind {
            ClassKind::Body => &self.body_palettes,
            ClassKind::Head => &self.head_palettes,
            ClassKind::Weapon | ClassKind::Headgear => return,
        };
        if list.entries.is_empty() {
            return;
        }

        // Index: -1 = Default, 0..n-1 = entries.
        let cur = match &list.selected {
            None => -1i32,
            Some(path) => list
                .entries
                .iter()
                .position(|e| &e.path == path)
                .map(|i| i as i32)
                .unwrap_or(-1),
        };
        let max = (list.entries.len() as i32) - 1;
        let next = (cur + delta).clamp(-1, max);

        if next < 0 {
            self.reset_palette(kind);
        } else {
            let path = list.entries[next as usize].path.clone();
            self.apply_palette(kind, &path);
        }
    }

    pub(crate) fn apply_palette(&mut self, kind: ClassKind, path: &Path) {
        match pal::load_palette(path) {
            Ok(palette) => {
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| strings::FALLBACK_PALETTE_NAME.into());
                match kind {
                    ClassKind::Body => {
                        if let Some(base) = &mut self.base {
                            Arc::make_mut(&mut base.spr).apply_palette(palette);
                            self.body_palettes.selected = Some(path.to_path_buf());
                            self.status = strings::status_palette_body(&name);
                            self.texture_key.clear();
                        }
                    }
                    ClassKind::Head => {
                        if let Some(attach) = &mut self.attach {
                            Arc::make_mut(&mut attach.spr).apply_palette(palette);
                            self.head_palettes.selected = Some(path.to_path_buf());
                            self.status = strings::status_palette_head(&name);
                            self.texture_key.clear();
                        }
                    }
                    ClassKind::Weapon | ClassKind::Headgear => {}
                }
            }
            Err(e) => self.status = strings::status_palette_err(e),
        }
    }

    pub(crate) fn reset_palette(&mut self, kind: ClassKind) {
        match kind {
            ClassKind::Body => {
                if let Some(base) = &mut self.base {
                    Arc::make_mut(&mut base.spr).reset_palette();
                    self.body_palettes.selected = None;
                    self.status = strings::STATUS_PALETTE_BODY_DEFAULT.into();
                    self.texture_key.clear();
                }
            }
            ClassKind::Head => {
                if let Some(attach) = &mut self.attach {
                    Arc::make_mut(&mut attach.spr).reset_palette();
                    self.head_palettes.selected = None;
                    self.status = strings::STATUS_PALETTE_HEAD_DEFAULT.into();
                    self.texture_key.clear();
                }
            }
            ClassKind::Weapon | ClassKind::Headgear => {}
        }
    }

    fn refresh_body_palettes(&mut self) {
        let Some(sel) = &self.body_sel else {
            self.body_palettes = PaletteList::default();
            return;
        };
        if let Some(index) = &self.palette_index {
            self.body_palettes = filter_body_palettes(index, &sel.stem, sel.gender);
            return;
        }
        // Index not ready yet: keep list empty, avoid scanning 200k files on UI thread.
        self.body_palettes = PaletteList::default();
    }

    fn refresh_head_palettes(&mut self) {
        let Some(sel) = &self.head_sel else {
            self.head_palettes = PaletteList::default();
            return;
        };
        if let Some(index) = &self.palette_index {
            self.head_palettes = filter_head_palettes(index, &sel.stem, sel.gender);
            return;
        }
        self.head_palettes = PaletteList::default();
    }

    pub(crate) fn open_class_sprite(&mut self, kind: ClassKind, path: &Path) {
        match kind {
            ClassKind::Body => self.load_body_sprite(path),
            ClassKind::Head => self.load_head_sprite(path),
            ClassKind::Weapon => self.load_weapon_sprite(path),
            ClassKind::Headgear => self.load_headgear_sprite(path),
        }
    }

    pub(crate) fn navigate_class_sprites(&mut self, kind: ClassKind, delta: i32) {
        let paths = {
            let Some(browser) = self.class_browser(kind) else {
                return;
            };
            if kind == ClassKind::Weapon {
                let Some(sel) = &self.body_sel else {
                    return;
                };
                collect_weapon_paths(browser, &sel.stem, sel.gender)
            } else {
                collect_sprite_paths(browser)
            }
        };
        if paths.is_empty() {
            return;
        }

        let current = self
            .class_browser(kind)
            .and_then(|b| b.selected.clone());

        let new_idx = match current.as_ref().and_then(|p| paths.iter().position(|x| x == p)) {
            Some(idx) if delta < 0 => idx.saturating_sub((-delta) as usize),
            Some(idx) => (idx + delta as usize).min(paths.len() - 1),
            None if delta < 0 => paths.len() - 1,
            None => 0,
        };

        let path = paths[new_idx].clone();
        if let Some(browser) = self.class_browser_mut(kind) {
            expand_for_sprite(browser, &path);
        }

        self.open_class_sprite(kind, &path);
    }

    pub(crate) fn load_body_sprite(&mut self, path: &Path) {
        if let Some(b) = &mut self.body_classes {
            b.selected = Some(path.to_path_buf());
        }
        match load_pair(path) {
            Ok(pair) => {
                self.body_sel = sprite_sel_from_name(&pair.name);
                self.status = strings::status_body(&pair.name);
                self.base = Some(pair);
                self.action_idx = 0;
                self.frame_idx = 0;
                self.playing = false;
                self.anchor_offsets.clear();
                self.dragging_anchor = false;
                self.texture_key.clear();
                self.last_tick = Instant::now();
                self.refresh_body_palettes();
                // Drop weapon if it no longer matches the new body class/gender.
                self.clear_weapon_if_mismatch();
            }
            Err(e) => self.status = strings::status_body_err(e),
        }
    }

    fn clear_weapon_if_mismatch(&mut self) {
        let Some(sel) = &self.body_sel else {
            self.weapon = None;
            return;
        };
        let Some(weapon) = &self.weapon else {
            return;
        };
        let Some((stem, gender)) = names::parse_sprite_stem(&weapon.name) else {
            self.weapon = None;
            return;
        };
        if stem != sel.stem || gender != sel.gender {
            self.weapon = None;
            self.texture_key.clear();
            if let Some(b) = &mut self.weapon_classes {
                b.selected = None;
            }
        }
    }

    pub(crate) fn load_head_sprite(&mut self, path: &Path) {
        if let Some(b) = &mut self.head_classes {
            b.selected = Some(path.to_path_buf());
        }
        match load_pair(path) {
            Ok(pair) => {
                self.head_sel = sprite_sel_from_name(&pair.name);
                self.status = strings::status_head(&pair.name);
                self.attach = Some(pair);
                self.texture_key.clear();
                self.refresh_head_palettes();
            }
            Err(e) => self.status = strings::status_head_err(e),
        }
    }

    pub(crate) fn load_weapon_sprite(&mut self, path: &Path) {
        if let Some(b) = &mut self.weapon_classes {
            b.selected = Some(path.to_path_buf());
        }
        if self.base.is_none() {
            self.status = strings::STATUS_LOAD_BODY_BEFORE_WEAPON.into();
            return;
        }
        match load_pair(path) {
            Ok(pair) => {
                self.status = strings::status_weapon(&pair.name);
                self.weapon = Some(pair);
                self.texture_key.clear();
            }
            Err(e) => self.status = strings::status_weapon_err(e),
        }
    }

    pub(crate) fn load_headgear_sprite(&mut self, path: &Path) {
        if let Some(b) = &mut self.headgear_classes {
            b.selected = Some(path.to_path_buf());
        }
        if self.base.is_none() {
            self.status = strings::STATUS_LOAD_BODY_BEFORE_HEADGEAR.into();
            return;
        }
        let Some(tx) = self.start_job(strings::JOB_LOAD_HEADGEAR) else {
            return;
        };
        let path = path.to_path_buf();
        thread::spawn(move || {
            let result = load_pair(&path);
            let _ = tx.send(JobResult::LoadHeadgear { path, result });
        });
    }

    /// True when `path` is the current selection of the `kind` browser.
    pub(crate) fn is_class_selected(&self, kind: ClassKind, path: &Path) -> bool {
        self.class_browser(kind)
            .and_then(|b| b.selected.as_deref())
            .is_some_and(|p| p == path)
    }

    pub(crate) fn unload_class_sprite(&mut self, kind: ClassKind) {
        if let Some(b) = self.class_browser_mut(kind) {
            b.selected = None;
        }
        match kind {
            ClassKind::Body => {
                self.clear_body();
                // Weapons are filtered by body class/gender.
                self.clear_weapon();
                self.status = strings::STATUS_BODY_CLEARED.into();
            }
            ClassKind::Head => {
                self.attach = None;
                self.head_sel = None;
                self.head_palettes = PaletteList::default();
                self.status = strings::STATUS_HEAD_CLEARED.into();
            }
            ClassKind::Weapon => {
                self.clear_weapon();
                self.status = strings::STATUS_WEAPON_CLEARED.into();
            }
            ClassKind::Headgear => {
                self.headgear = None;
                self.status = strings::STATUS_HEADGEAR_CLEARED.into();
            }
        }
        self.texture_key.clear();
    }

    fn clear_body(&mut self) {
        self.base = None;
        self.body_sel = None;
        self.body_palettes = PaletteList::default();
        self.action_idx = 0;
        self.frame_idx = 0;
        self.playing = false;
        self.anchor_offsets.clear();
        self.dragging_anchor = false;
        if let Some(folder) = &mut self.folder {
            folder.selected = None;
        }
    }

    fn clear_weapon(&mut self) {
        self.weapon = None;
        if let Some(b) = &mut self.weapon_classes {
            b.selected = None;
        }
    }

    pub(crate) fn has_selection(&self) -> bool {
        self.base.is_some()
            || self.attach.is_some()
            || self.weapon.is_some()
            || self.headgear.is_some()
    }

    /// Drop every loaded sprite and palette; keeps the open folder and indexes.
    pub(crate) fn reset_selection(&mut self) {
        for kind in [
            ClassKind::Body,
            ClassKind::Head,
            ClassKind::Weapon,
            ClassKind::Headgear,
        ] {
            self.unload_class_sprite(kind);
        }
        self.texture = None;
        self.status = strings::STATUS_RESET.into();
    }

    pub(crate) fn toggle_dir(&mut self, path: &Path) {
        let Some(folder) = &mut self.folder else {
            return;
        };
        if let Err(e) = folder.toggle(path) {
            self.status = strings::status_folder_err(e);
        }
    }

    pub(crate) fn load_from_folder(&mut self, path: &Path) {
        if let Some(folder) = &mut self.folder {
            folder.selected = Some(path.to_path_buf());
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase());
        match ext.as_deref() {
            Some("spr") | Some("act") => match load_pair(path) {
                Ok(pair) => {
                    self.body_sel = sprite_sel_from_name(&pair.name);
                    self.status = strings::status_loaded(&pair.name);
                    self.base = Some(pair);
                    self.action_idx = 0;
                    self.frame_idx = 0;
                    self.playing = false;
                    self.anchor_offsets.clear();
                    self.dragging_anchor = false;
                    self.texture_key.clear();
                    self.last_tick = Instant::now();
                    self.refresh_body_palettes();
                }
                Err(e) => self.status = strings::status_err(e),
            },
            _ => {
                self.status = strings::status_file(path.display());
            }
        }
    }

    fn data_ready(&self) -> bool {
        self.body_classes.is_some() && self.head_classes.is_some()
    }

    pub(crate) fn load_random(&mut self) {
        let Some(tx) = self.start_job(strings::JOB_RANDOM) else {
            return;
        };

        let mut rng = rand::rng();
        // Skip orphan .spr without matching .act (random used to abort entirely).
        let body_paths: Vec<PathBuf> = self
            .body_classes
            .as_ref()
            .map(collect_sprite_paths)
            .unwrap_or_default()
            .into_iter()
            .filter(|p| p.with_extension("act").is_file())
            .collect();
        let head_paths_all: Vec<PathBuf> = self
            .head_classes
            .as_ref()
            .map(collect_sprite_paths)
            .unwrap_or_default()
            .into_iter()
            .filter(|p| p.with_extension("act").is_file())
            .collect();
        let cur_body = self
            .body_classes
            .as_ref()
            .and_then(|b| b.selected.clone());
        let cur_head = self
            .head_classes
            .as_ref()
            .and_then(|b| b.selected.clone());

        let body_path = pick_random_path(&body_paths, cur_body.as_deref(), &mut rng);
        let Some(body_path) = body_path else {
            self.busy = None;
            self.status = strings::STATUS_RANDOM_NO_BODY.into();
            return;
        };

        let body_gender = body_path
            .file_stem()
            .and_then(|s| sprite_sel_from_name(&s.to_string_lossy()))
            .map(|s| s.gender);
        let head_paths: Vec<PathBuf> = match body_gender {
            Some(g) => head_paths_all
                .into_iter()
                .filter(|p| {
                    p.file_stem()
                        .and_then(|s| sprite_sel_from_name(&s.to_string_lossy()))
                        .is_some_and(|sel| sel.gender == g)
                })
                .collect(),
            None => head_paths_all,
        };
        let head_path = pick_random_path(&head_paths, cur_head.as_deref(), &mut rng);

        let body_pal_pick = body_path
            .file_stem()
            .and_then(|s| sprite_sel_from_name(&s.to_string_lossy()))
            .and_then(|sel| self.pick_random_body_pal(&sel.stem, sel.gender, &mut rng));
        let head_pal_pick = head_path
            .as_ref()
            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .and_then(|name| sprite_sel_from_name(&name))
            .and_then(|sel| self.pick_random_head_pal(&sel.stem, sel.gender, &mut rng));

        let body_palette_dir = self.body_palette_dir.clone();
        let head_palette_dir = self.head_palette_dir.clone();
        let has_index = self.palette_index.is_some();

        thread::spawn(move || {
            let result = (|| {
                let mut body = load_pair(&body_path)?;
                let body_pal = if let Some(path) = body_pal_pick {
                    if let Ok(palette) = pal::load_palette(&path) {
                        Arc::make_mut(&mut body.spr).apply_palette(palette);
                    }
                    Some(path)
                } else if !has_index {
                    sprite_sel_from_name(&body.name).and_then(|sel| {
                        let dir = body_palette_dir.as_ref()?;
                        let list = scan_body_palettes(dir, &sel.stem, sel.gender);
                        let mut rng = rand::rng();
                        let path = list.entries.choose(&mut rng)?.path.clone();
                        let palette = pal::load_palette(&path).ok()?;
                        Arc::make_mut(&mut body.spr).apply_palette(palette);
                        Some(path)
                    })
                } else {
                    None
                };

                // Head failure must not abort body apply.
                let head = if let Some(head_path) = head_path {
                    match load_pair(&head_path) {
                        Ok(mut head) => {
                            let head_pal = if let Some(path) = head_pal_pick {
                                if let Ok(palette) = pal::load_palette(&path) {
                                    Arc::make_mut(&mut head.spr).apply_palette(palette);
                                }
                                Some(path)
                            } else if !has_index {
                                sprite_sel_from_name(&head.name).and_then(|sel| {
                                    let dir = head_palette_dir.as_ref()?;
                                    let list = scan_head_palettes(dir, &sel.stem, sel.gender);
                                    let mut rng = rand::rng();
                                    let path = list.entries.choose(&mut rng)?.path.clone();
                                    let palette = pal::load_palette(&path).ok()?;
                                    Arc::make_mut(&mut head.spr).apply_palette(palette);
                                    Some(path)
                                })
                            } else {
                                None
                            };
                            Some((head, head_path, head_pal))
                        }
                        Err(_) => None,
                    }
                } else {
                    None
                };

                let (head_pair, head_pal) = match head {
                    Some((h, p, pal)) => (Some((h, p)), pal),
                    None => (None, None),
                };

                Ok(RandomResult {
                    body,
                    body_path,
                    head: head_pair,
                    body_pal,
                    head_pal,
                })
            })();
            let _ = tx.send(JobResult::Random(result));
        });
    }

}

/// Prefer a path different from `current` when the pool has more than one entry.
fn pick_random_path(
    paths: &[PathBuf],
    current: Option<&Path>,
    rng: &mut impl rand::Rng,
) -> Option<PathBuf> {
    if paths.is_empty() {
        return None;
    }
    if paths.len() == 1 {
        return paths.first().cloned();
    }
    if let Some(cur) = current {
        let others: Vec<&PathBuf> = paths.iter().filter(|p| p.as_path() != cur).collect();
        if !others.is_empty() {
            return others.choose(rng).map(|p| (*p).clone());
        }
    }
    paths.choose(rng).cloned()
}

impl SpriteEditorApp {
    fn pick_random_body_pal(
        &self,
        stem: &str,
        gender: Gender,
        rng: &mut impl rand::Rng,
    ) -> Option<PathBuf> {
        let index = self.palette_index.as_ref()?;
        let list = filter_body_palettes(index, stem, gender);
        list.entries.choose(rng).map(|e| e.path.clone())
    }

    fn pick_random_head_pal(
        &self,
        stem: &str,
        gender: Gender,
        rng: &mut impl rand::Rng,
    ) -> Option<PathBuf> {
        let index = self.palette_index.as_ref()?;
        let list = filter_head_palettes(index, stem, gender);
        list.entries.choose(rng).map(|e| e.path.clone())
    }

    pub(crate) fn current_anchor_offset(&self) -> (i32, i32) {
        self.anchor_offsets
            .get(&self.action_idx)
            .copied()
            .unwrap_or((0, 0))
    }

    pub(crate) fn set_anchor_offset(&mut self, dx: i32, dy: i32) {
        if dx == 0 && dy == 0 {
            self.anchor_offsets.remove(&self.action_idx);
        } else {
            self.anchor_offsets.insert(self.action_idx, (dx, dy));
        }
        self.texture_key.clear();
    }

    pub(crate) fn reset_anchor_offset(&mut self) {
        self.anchor_offsets.remove(&self.action_idx);
        self.dragging_anchor = false;
        self.texture_key.clear();
    }

    pub(crate) fn current_render(&self) -> Option<RenderedFrame> {
        let base = self.base.as_ref()?;
        compose_frame(
            base,
            self.action_idx,
            self.frame_idx,
            self.weapon.as_ref(),
            self.attach.as_ref(),
            self.headgear.as_ref(),
            self.current_anchor_offset(),
        )
    }
}
