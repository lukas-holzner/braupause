use std::collections::HashMap;
use std::path::PathBuf;

use egui::{Key, KeyboardShortcut, Modifiers, RichText, Ui};

use std::sync::mpsc::{Receiver, channel};

use crate::git::{self, CommitPlan, LineKind, Msg, RepoStatus, Runner};
use crate::images::{self, Imported};
use crate::model::{self, BatchDoc, NewBatchSpec, Tab};
use crate::tabs;
use crate::theme;
use crate::validate::{Level, SchemaValidator};
use crate::vocab;

const STORAGE_REPO: &str = "repo_path";
const STORAGE_EDGE: &str = "image_max_edge";

struct ImportDone {
    file_id: String,
    step: Option<u64>,
    results: Vec<Result<Imported, String>>,
}

pub struct BrauApp {
    repo: Option<PathBuf>,
    docs: Vec<BatchDoc>,
    selected: Option<usize>,
    schema: Option<SchemaValidator>,
    runner: Runner,
    status: Option<RepoStatus>,
    commit_sel: HashMap<String, bool>,
    commit_msg: String,
    commit_msg_auto: bool,
    run_npm_test: bool,
    bump_modified: bool,
    log: Vec<(LineKind, String)>,
    notice: Option<(String, bool, f64)>,
    new_dialog: Option<NewDialog>,
    confirm_close: bool,
    allow_close: bool,
    confirm_reload: bool,
    max_edge: u32,
    import_rx: Option<Receiver<ImportDone>>,
    /// Bild-URIs, deren Vorschau neu geladen werden muss.
    ctx_forget: Vec<String>,
}

struct NewDialog {
    spec: NewBatchSpec,
    id_touched: bool,
    error: Option<String>,
}

impl BrauApp {
    pub fn new(cc: &eframe::CreationContext<'_>, cli_path: Option<PathBuf>) -> Self {
        theme::apply(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let max_edge = cc
            .storage
            .and_then(|s| s.get_string(STORAGE_EDGE))
            .and_then(|v| v.parse().ok())
            .filter(|e| images::EDGE_CHOICES.contains(e))
            .unwrap_or(images::DEFAULT_EDGE);
        let stored = cc.storage.and_then(|s| s.get_string(STORAGE_REPO)).map(PathBuf::from);
        let candidates = [
            cli_path,
            std::env::current_dir().ok(),
            std::env::current_exe().ok().and_then(|p| p.parent().map(PathBuf::from)),
            stored,
        ];
        let repo = candidates.into_iter().flatten().find_map(|p| model::find_repo_root(&p));
        let mut app = Self {
            repo: None,
            docs: Vec::new(),
            selected: None,
            schema: None,
            runner: Runner::new(),
            status: None,
            commit_sel: HashMap::new(),
            commit_msg: String::new(),
            commit_msg_auto: true,
            run_npm_test: true,
            bump_modified: true,
            log: Vec::new(),
            notice: None,
            new_dialog: None,
            confirm_close: false,
            allow_close: false,
            confirm_reload: false,
            max_edge,
            import_rx: None,
            ctx_forget: Vec::new(),
        };
        if let Some(repo) = repo {
            app.open_repo(&cc.egui_ctx, repo);
        }
        app
    }

    fn open_repo(&mut self, ctx: &egui::Context, repo: PathBuf) {
        match model::load_all(&repo) {
            Ok(docs) => {
                self.schema = Some(SchemaValidator::load(&repo));
                self.docs = docs;
                self.selected = if self.docs.is_empty() { None } else { Some(0) };
                self.repo = Some(repo.clone());
                self.status = None;
                self.commit_sel.clear();
                if !self.runner.busy() {
                    self.runner.refresh_status(ctx, repo);
                }
            }
            Err(e) => self.notify(e, true),
        }
    }

    fn notify(&mut self, msg: impl Into<String>, is_error: bool) {
        self.notice = Some((msg.into(), is_error, 0.0));
    }

    fn dirty_count(&self) -> usize {
        self.docs.iter().filter(|d| d.dirty).count()
    }

    fn save_all(&mut self) -> bool {
        let mut saved = Vec::new();
        for doc in self.docs.iter_mut().filter(|d| d.dirty) {
            match doc.save(self.bump_modified) {
                Ok(()) => saved.push(doc.file_id.clone()),
                Err(e) => {
                    self.notice = Some((format!("Speichern fehlgeschlagen: {e}"), true, 0.0));
                    return false;
                }
            }
        }
        if !saved.is_empty() {
            self.notify(format!("Gespeichert: {}", saved.join(", ")), false);
        }
        self.refresh_status_sync();
        true
    }

    /// Git-Status direkt abfragen (schnell), damit gerade gespeicherte Dateien sofort auftauchen.
    fn refresh_status_sync(&mut self) {
        if let Some(repo) = &self.repo
            && let Ok(s) = git::status(repo)
        {
            self.apply_status(s);
        }
    }

    fn apply_status(&mut self, s: RepoStatus) {
        let mut sel = HashMap::new();
        for (_, path) in &s.files {
            let default = path.starts_with("batches/") || path.starts_with("assets/batches/");
            sel.insert(path.clone(), *self.commit_sel.get(path).unwrap_or(&default));
        }
        self.commit_sel = sel;
        self.status = Some(s);
        if self.commit_msg_auto {
            self.commit_msg = self.suggest_message();
        }
    }

    fn selected_files(&self) -> Vec<String> {
        let Some(s) = &self.status else { return Vec::new() };
        s.files
            .iter()
            .filter(|(_, p)| self.commit_sel.get(p).copied().unwrap_or(false))
            .map(|(_, p)| p.clone())
            .collect()
    }

    fn suggest_message(&self) -> String {
        let Some(s) = &self.status else { return String::new() };
        let mut new = Vec::new();
        let mut changed = Vec::new();
        for (code, path) in &s.files {
            if !self.commit_sel.get(path).copied().unwrap_or(false) {
                continue;
            }
            let Some(id) = path.strip_prefix("batches/").and_then(|p| p.strip_suffix(".json")) else {
                continue;
            };
            if code == "??" || code.starts_with('A') {
                new.push(id.to_string());
            } else {
                changed.push(id.to_string());
            }
        }
        let mut parts = Vec::new();
        if !new.is_empty() {
            parts.push(format!("Neuer Sud {}", new.join(", ")));
        }
        if !changed.is_empty() {
            parts.push(format!("Protokoll {} aktualisiert", changed.join(", ")));
        }
        if parts.is_empty() && !self.selected_files().is_empty() {
            return "Braupause aktualisiert".into();
        }
        parts.join("; ")
    }

    fn start_commit(&mut self, ctx: &egui::Context, commit: bool, push: bool) {
        let Some(repo) = self.repo.clone() else { return };
        if commit {
            if !self.save_all() {
                return;
            }
            self.validate_all();
            let broken: Vec<String> = self
                .docs
                .iter()
                .filter(|d| d.parse_error.is_some() || d.issues.iter().any(|i| i.level == Level::Error))
                .map(|d| d.file_id.clone())
                .collect();
            if !broken.is_empty() {
                self.notify(format!("Commit abgebrochen – Validierungsfehler in: {}", broken.join(", ")), true);
                return;
            }
        }
        let plan = CommitPlan {
            files: self.selected_files(),
            message: self.commit_msg.clone(),
            run_npm_test: commit && self.run_npm_test,
            commit,
            push,
        };
        self.log.clear();
        self.runner.commit_and_push(ctx, repo, plan);
        if commit {
            self.commit_msg_auto = true;
        }
    }

    fn validate_all(&mut self) {
        let Some(schema) = &self.schema else { return };
        for doc in &mut self.docs {
            if doc.issues_gen != doc.generation {
                doc.issues = if doc.parse_error.is_some() {
                    Vec::new()
                } else {
                    schema.check(&doc.file_id, &doc.value)
                };
                doc.issues_gen = doc.generation;
            }
        }
    }

    fn start_import(&mut self, ctx: &egui::Context, file_id: String, req: tabs::ImportRequest) {
        let Some(repo) = self.repo.clone() else { return };
        let (tx, rx) = channel();
        self.import_rx = Some(rx);
        let max_edge = self.max_edge;
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let results = req.paths.iter().map(|p| images::import(&repo, &file_id, p, max_edge)).collect();
            let _ = tx.send(ImportDone { file_id, step: req.step, results });
            ctx.request_repaint();
        });
    }

    fn poll_imports(&mut self) {
        let Some(rx) = &self.import_rx else { return };
        let Ok(done) = rx.try_recv() else { return };
        self.import_rx = None;
        let mut ok = Vec::new();
        let mut errors = Vec::new();
        if let Some(doc) = self.docs.iter_mut().find(|d| d.file_id == done.file_id)
            && let Some(root) = doc.value.as_object_mut()
        {
            let mut entries = Vec::new();
            for r in done.results {
                match r {
                    Ok(img) => {
                        if let Some(repo) = &self.repo {
                            self.ctx_forget.push(format!("file://{}", repo.join(&img.content_url).display()));
                        }
                        entries.push(model::new_image(&img.content_url, &img.caption, done.step));
                        ok.push(format!("{} → {} × {} px, {}", img.source_name, img.width, img.height, images::human_size(img.bytes)));
                    }
                    Err(e) => errors.push(e),
                }
            }
            if !entries.is_empty() {
                let mut list = root.get("image").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                list.extend(entries);
                model::set_after(root, "image", serde_json::Value::Array(list), "dateModified");
                model::sync_step_images(root);
                doc.touch();
            }
        }
        for line in &ok {
            self.log.push((LineKind::Success, format!("Foto importiert: {line}")));
        }
        for e in &errors {
            self.log.push((LineKind::Error, format!("Foto-Import: {e}")));
        }
        if errors.is_empty() {
            self.notify(format!("{} Foto(s) importiert – Bildunterschriften im Tab „Fotos“ ergänzen.", ok.len()), false);
        } else {
            self.notify(format!("{} importiert, {} fehlgeschlagen: {}", ok.len(), errors.len(), errors.join("; ")), true);
        }
    }

    fn trash_files(&mut self, ctx: &egui::Context, urls: Vec<String>) {
        let Some(repo) = self.repo.clone() else { return };
        for url in urls {
            let abs = repo.join(&url);
            match trash::delete(&abs) {
                Ok(()) => {
                    ctx.forget_image(&format!("file://{}", abs.display()));
                    self.notify(format!("{url} in den Papierkorb verschoben."), false);
                }
                Err(e) => self.notify(format!("{url}: {e}"), true),
            }
        }
        self.refresh_status_sync();
    }

    fn poll_runner(&mut self) {
        for m in self.runner.poll() {
            match m {
                Msg::Line(k, t) => self.log.push((k, t)),
                Msg::Status(s) => self.apply_status(s),
                Msg::Done { ok } => {
                    if !ok {
                        self.notify("Vorgang fehlgeschlagen – siehe Protokoll rechts.", true);
                    }
                }
            }
        }
    }

    // ─────────────────────────── Oberfläche ───────────────────────────

    fn top_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add(egui::Image::new(egui::include_image!("../assets/editor-icon.png")).fit_to_exact_size(egui::vec2(30.0, 30.0)));
            ui.label(RichText::new("Brau").size(20.0).strong().color(theme::INK_PRIMARY));
            ui.add_space(-8.0);
            ui.label(RichText::new("pause").size(20.0).strong().color(theme::AMBER));
            ui.label(RichText::new("Sud-Editor").color(theme::INK_MUTED));
            ui.separator();
            let has_repo = self.repo.is_some();
            if ui.add_enabled(has_repo, egui::Button::new("+ Neuer Sud")).clicked() {
                self.open_new_dialog();
            }
            let dirty = self.dirty_count();
            let save_label = if dirty > 0 { format!("Speichern ({dirty})") } else { "Speichern".into() };
            if ui
                .add_enabled(dirty > 0, theme::primary_button(&save_label))
                .on_hover_text("Alle geänderten Sude speichern (⌘S)")
                .clicked()
            {
                self.save_all();
            }
            if ui
                .add_enabled(self.selected.is_some(), egui::Button::new("Neu laden"))
                .on_hover_text("Ausgewählten Sud von der Festplatte neu laden")
                .clicked()
            {
                self.request_reload();
            }
            ui.checkbox(&mut self.bump_modified, "dateModified beim Speichern setzen");
            ui.separator();
            if ui.button("Repo wählen …").clicked()
                && let Some(dir) = rfd::FileDialog::new().set_title("Braupause-Repository wählen").pick_folder()
            {
                match model::find_repo_root(&dir) {
                    Some(root) => self.open_repo(ui.ctx(), root),
                    None => self.notify("Kein Braupause-Repo (batches/ + schemas/) gefunden.", true),
                }
            }
            if let Some(repo) = &self.repo {
                ui.label(RichText::new(repo.display().to_string()).small().color(theme::INK_MUTED));
            }
        });
        if let Some((msg, is_error, _)) = &self.notice {
            let (fg, bg) = if *is_error { (theme::ERROR, theme::ERROR_TINT) } else { (theme::OLIVE, theme::OLIVE_TINT) };
            let mut close = false;
            egui::Frame::new().fill(bg).corner_radius(4.0).inner_margin(egui::Margin::symmetric(8, 4)).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(msg).color(fg));
                    close = ui.small_button("✕").clicked();
                });
            });
            if close {
                self.notice = None;
            }
        }
    }

    fn batch_list(&mut self, ui: &mut Ui) {
        ui.add_space(6.0);
        ui.label(RichText::new("Sude").size(16.0).strong());
        ui.add_space(4.0);
        if self.repo.is_none() {
            ui.label("Kein Repository geöffnet. Oben „Repo wählen …“ nutzen.");
            return;
        }
        egui::ScrollArea::vertical().id_salt("list").show(ui, |ui| {
            for (i, doc) in self.docs.iter().enumerate() {
                let selected = self.selected == Some(i);
                let errors = doc.issues.iter().filter(|x| x.level == Level::Error).count()
                    + usize::from(doc.parse_error.is_some());
                let fill = if selected { theme::AMBER_TINT } else { theme::SURFACE };
                let stroke = if selected { theme::AMBER } else { theme::HAIRLINE };
                let resp = egui::Frame::new()
                    .fill(fill)
                    .stroke(egui::Stroke::new(1.0, stroke))
                    .corner_radius(6.0)
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            crate::widgets::chip(ui, &doc.file_id, theme::INK_SECONDARY, theme::BG_SUBTLE);
                            if doc.dirty {
                                ui.label(RichText::new("●").color(theme::AMBER)).on_hover_text("Ungespeicherte Änderungen");
                            }
                            if errors > 0 {
                                ui.label(RichText::new(format!("⚠ {errors}")).color(theme::ERROR));
                            }
                        });
                        let name = doc.get_str("name");
                        ui.label(RichText::new(if name.is_empty() { "(ohne Namen)" } else { name }).strong());
                        let status = doc.get_str("brew:statusLabel");
                        let status = if status.is_empty() {
                            vocab::label(vocab::STATUS, doc.get_str("brew:status"))
                        } else {
                            status.to_string()
                        };
                        if !status.is_empty() {
                            ui.label(RichText::new(status).small().color(theme::OLIVE));
                        }
                    })
                    .response
                    .interact(egui::Sense::click());
                if resp.clicked() {
                    self.selected = Some(i);
                }
                ui.add_space(4.0);
            }
        });
    }

    fn git_panel(&mut self, ui: &mut Ui) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Git").size(16.0).strong());
            if self.runner.busy() {
                ui.spinner();
            }
        });
        let Some(repo) = self.repo.clone() else { return };
        let busy = self.runner.busy();

        if let Some(s) = &self.status {
            ui.horizontal_wrapped(|ui| {
                ui.label("Branch:");
                ui.label(RichText::new(if s.branch.is_empty() { "(detached)" } else { &s.branch }).monospace().strong());
                match &s.upstream {
                    Some(u) => {
                        ui.label(RichText::new(format!("→ {u}")).monospace().color(theme::INK_MUTED));
                        if s.ahead > 0 {
                            ui.label(RichText::new(format!("{} ungepusht", s.ahead)).color(theme::AMBER));
                        }
                    }
                    None if s.remotes.is_empty() => {
                        ui.label(RichText::new("kein Remote").color(theme::WARN));
                    }
                    None => {
                        ui.label(RichText::new("ohne Upstream").color(theme::INK_MUTED));
                    }
                }
            });
            if !s.has_commits {
                ui.label(RichText::new("Repository hat noch keinen Commit.").small().color(theme::INK_MUTED));
            }
        }

        ui.add_space(6.0);
        ui.label(RichText::new("Geänderte Dateien").strong());
        let mut sel_changed = false;
        egui::ScrollArea::vertical().id_salt("files").max_height(180.0).show(ui, |ui| {
            match &self.status {
                Some(s) if s.files.is_empty() => {
                    ui.label(RichText::new("Arbeitsverzeichnis sauber.").color(theme::INK_MUTED));
                }
                Some(s) => {
                    for (code, path) in &s.files {
                        let checked = self.commit_sel.entry(path.clone()).or_insert(false);
                        ui.horizontal(|ui| {
                            sel_changed |= ui.checkbox(checked, "").changed();
                            let color = if code == "??" { theme::OLIVE } else { theme::AMBER_HOVER };
                            ui.label(RichText::new(code.replace(' ', "·")).monospace().color(color));
                            ui.label(RichText::new(path).monospace().small());
                        });
                    }
                }
                None => {
                    ui.label(RichText::new("Status wird geladen …").color(theme::INK_MUTED));
                }
            }
        });
        if sel_changed && self.commit_msg_auto {
            self.commit_msg = self.suggest_message();
        }
        ui.horizontal(|ui| {
            if ui.add_enabled(!busy, egui::Button::new("↻ Status")).clicked() {
                self.runner.refresh_status(ui.ctx(), repo.clone());
            }
            if ui.add_enabled(!busy, egui::Button::new("Pull")).on_hover_text("git pull --ff-only").clicked() {
                self.log.clear();
                self.runner.pull(ui.ctx(), repo.clone());
            }
            if ui.add_enabled(!busy, egui::Button::new("npm test")).clicked() {
                self.log.clear();
                self.runner.npm_test(ui.ctx(), repo.clone());
            }
        });

        ui.add_space(8.0);
        ui.label(RichText::new("Commit-Nachricht").strong());
        let r = ui.add(
            egui::TextEdit::multiline(&mut self.commit_msg)
                .desired_rows(2)
                .desired_width(f32::INFINITY)
                .hint_text("z. B. MET-2026-01: Messung 20.11. eingetragen"),
        );
        if r.changed() {
            self.commit_msg_auto = self.commit_msg.trim().is_empty();
        }
        ui.checkbox(&mut self.run_npm_test, "Vorher npm test");

        let n_files = self.selected_files().len() + self.docs.iter().filter(|d| d.is_new).count();
        let can_commit = !busy && (n_files > 0 || self.dirty_count() > 0) && !self.commit_msg.trim().is_empty();
        ui.horizontal(|ui| {
            if ui
                .add_enabled(can_commit, theme::primary_button("Commit & Push"))
                .on_hover_text("Speichert alles, prüft, committet die ausgewählten Dateien und pusht.")
                .clicked()
            {
                self.start_commit(ui.ctx(), true, true);
            }
            if ui.add_enabled(can_commit, egui::Button::new("Nur Commit")).clicked() {
                self.start_commit(ui.ctx(), true, false);
            }
            let can_push = !busy
                && self
                    .status
                    .as_ref()
                    .is_some_and(|s| s.has_commits && (s.ahead > 0 || s.upstream.is_none()));
            if ui.add_enabled(can_push, egui::Button::new("Push")).clicked() {
                self.start_commit(ui.ctx(), false, true);
            }
        });

        ui.add_space(8.0);
        ui.label(RichText::new("Protokoll").strong());
        egui::Frame::new()
            .fill(theme::SURFACE)
            .stroke(egui::Stroke::new(1.0, theme::HAIRLINE))
            .corner_radius(4.0)
            .inner_margin(egui::Margin::same(6))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("log")
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if self.log.is_empty() {
                            ui.label(RichText::new("—").color(theme::INK_MUTED));
                        }
                        for (kind, line) in &self.log {
                            let color = match kind {
                                LineKind::Command => theme::BLUE,
                                LineKind::Output => theme::INK_SECONDARY,
                                LineKind::Error => theme::ERROR,
                                LineKind::Success => theme::OLIVE,
                            };
                            ui.label(RichText::new(line).monospace().size(11.5).color(color));
                        }
                    });
            });
    }

    fn issues_panel(&mut self, ui: &mut Ui) {
        let Some(doc) = self.selected.and_then(|i| self.docs.get(i)) else { return };
        let errors = doc.issues.iter().filter(|i| i.level == Level::Error).count();
        let warnings = doc.issues.len() - errors;
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Prüfung").strong());
            if let Some(e) = self.schema.as_ref().and_then(|s| s.load_error.as_ref()) {
                ui.label(RichText::new(e).color(theme::ERROR));
            } else if doc.parse_error.is_some() {
                ui.label(RichText::new("Kein gültiges JSON").color(theme::ERROR));
            } else if errors == 0 && warnings == 0 {
                ui.label(RichText::new("✓ Schema & Plausibilität in Ordnung").color(theme::OLIVE));
            } else {
                ui.label(RichText::new(format!("{errors} Fehler, {warnings} Hinweise")).color(if errors > 0 {
                    theme::ERROR
                } else {
                    theme::WARN
                }));
            }
        });
        egui::ScrollArea::vertical().id_salt("issues").auto_shrink([false, true]).show(ui, |ui| {
            for issue in &doc.issues {
                ui.horizontal_wrapped(|ui| {
                    let (sym, color) = match issue.level {
                        Level::Error => ("✕", theme::ERROR),
                        Level::Warning => ("!", theme::WARN),
                    };
                    ui.label(RichText::new(sym).color(color).strong());
                    ui.label(RichText::new(&issue.path).monospace().small().color(theme::INK_MUTED));
                    ui.label(&issue.message);
                });
            }
        });
    }

    fn editor(&mut self, ui: &mut Ui) {
        let Some(idx) = self.selected else {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Links einen Sud wählen oder einen neuen anlegen.").color(theme::INK_MUTED));
            });
            return;
        };
        let doc = &mut self.docs[idx];
        ui.horizontal(|ui| {
            ui.label(RichText::new(doc.get_str("name")).size(22.0).strong());
            crate::widgets::chip(ui, &doc.file_id, theme::INK_SECONDARY, theme::BG_SUBTLE);
            if doc.is_new {
                crate::widgets::chip(ui, "neu", theme::OLIVE, theme::OLIVE_TINT);
            }
        });
        ui.add_space(4.0);
        let parse_broken = doc.parse_error.is_some();
        if parse_broken {
            doc.ui.tab = Tab::Json;
        }
        ui.horizontal(|ui| {
            for (tab, label) in [
                (Tab::Overview, "Übersicht"),
                (Tab::Steps, "Protokoll"),
                (Tab::Photos, "Fotos"),
                (Tab::Volumes, "Volumen"),
                (Tab::Ingredients, "Zutaten"),
                (Tab::Tools, "Equipment"),
                (Tab::Json, "JSON"),
            ] {
                let enabled = !parse_broken || tab == Tab::Json;
                let selected = doc.ui.tab == tab;
                let text = if selected { RichText::new(label).strong().color(theme::AMBER_HOVER) } else { RichText::new(label) };
                if ui.add_enabled(enabled, egui::Button::selectable(selected, text)).clicked() {
                    doc.ui.tab = tab;
                }
            }
        });
        ui.separator();

        let repo = self.repo.clone().unwrap_or_default();
        let mut requests = Vec::new();
        let mut trash = Vec::new();
        let mut env = tabs::Env {
            repo: &repo,
            import: &mut requests,
            trash: &mut trash,
            importing: self.import_rx.is_some(),
            max_edge: &mut self.max_edge,
        };
        let changed = if doc.ui.tab == Tab::Json {
            tabs::json(ui, doc)
        } else {
            egui::ScrollArea::vertical()
                .id_salt(("editor", idx, doc.ui.tab as u8))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(ui.available_width().min(1100.0));
                    match doc.ui.tab {
                        Tab::Overview => tabs::overview(ui, doc),
                        Tab::Volumes => tabs::volumes(ui, doc),
                        Tab::Ingredients => tabs::ingredients(ui, doc),
                        Tab::Steps => tabs::steps(ui, doc, &mut env),
                        Tab::Photos => tabs::photos(ui, doc, &mut env),
                        Tab::Tools => tabs::tools(ui, doc),
                        Tab::Json => false,
                    }
                })
                .inner
        };
        if changed {
            if doc.ui.tab != Tab::Json
                && let Some(root) = doc.value.as_object_mut()
            {
                model::sync_step_images(root);
            }
            doc.touch();
        }
        let file_id = doc.file_id.clone();
        for req in requests {
            self.start_import(ui.ctx(), file_id.clone(), req);
        }
        if !trash.is_empty() {
            self.trash_files(ui.ctx(), trash);
        }
    }

    fn request_reload(&mut self) {
        let Some(i) = self.selected else { return };
        if self.docs[i].dirty {
            self.confirm_reload = true;
        } else {
            self.reload_selected();
        }
    }

    fn reload_selected(&mut self) {
        let Some(i) = self.selected else { return };
        // Vorschaubilder ebenfalls frisch von der Platte lesen
        self.ctx_forget.push(String::new());
        if self.docs[i].is_new {
            self.docs.remove(i);
            self.selected = if self.docs.is_empty() { None } else { Some(i.saturating_sub(1)) };
        } else {
            let path = self.docs[i].path.clone();
            let tab = self.docs[i].ui.tab;
            self.docs[i] = BatchDoc::load(&path);
            if self.docs[i].parse_error.is_none() {
                self.docs[i].ui.tab = tab;
            }
        }
    }

    fn open_new_dialog(&mut self) {
        let existing: Vec<String> = self.docs.iter().map(|d| d.file_id.clone()).collect();
        let template_author = self.docs.iter().rev().find_map(|d| {
            let a = d.value.get("author")?;
            Some((
                a.get("name")?.as_str()?.to_string(),
                a.get("worksFor").and_then(|w| w.get("name")).and_then(|n| n.as_str()).unwrap_or("").to_string(),
            ))
        });
        let (author, brewery) = template_author.unwrap_or(("Lukas Holzner".into(), "Heimbrauerei München-Sendling".into()));
        self.new_dialog = Some(NewDialog {
            spec: NewBatchSpec {
                identifier: model::suggest_identifier("Mead", &existing),
                name: String::new(),
                category: "Mead".into(),
                author,
                brewery,
                initial_volume: 10.0,
                date_created: model::now_iso(),
            },
            id_touched: false,
            error: None,
        });
    }

    fn new_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(dlg) = &mut self.new_dialog else { return };
        let existing: Vec<String> = self.docs.iter().map(|d| d.file_id.clone()).collect();
        let mut create = false;
        let mut cancel = false;
        let resp = egui::Modal::new(egui::Id::unique("new_batch")).show(ctx, |ui| {
            ui.set_width(460.0);
            ui.label(RichText::new("Neuer Sud").size(18.0).strong());
            ui.add_space(6.0);
            egui::Grid::new("new.grid").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                ui.label("Kategorie");
                let before = dlg.spec.category.clone();
                egui::ComboBox::from_id_salt("new.cat")
                    .selected_text(vocab::label(vocab::CATEGORIES, &dlg.spec.category))
                    .show_ui(ui, |ui| {
                        for (v, l) in vocab::CATEGORIES {
                            ui.selectable_value(&mut dlg.spec.category, (*v).to_string(), *l);
                        }
                    });
                if before != dlg.spec.category && !dlg.id_touched {
                    dlg.spec.identifier = model::suggest_identifier(&dlg.spec.category, &existing);
                }
                ui.end_row();
                ui.label("Kennung");
                let r = ui.add(egui::TextEdit::singleline(&mut dlg.spec.identifier).font(egui::TextStyle::Monospace));
                if r.changed() {
                    dlg.id_touched = true;
                    dlg.spec.identifier = dlg.spec.identifier.to_uppercase();
                }
                ui.end_row();
                ui.label("Name");
                ui.add(egui::TextEdit::singleline(&mut dlg.spec.name).hint_text("z. B. Münchner Helles"));
                ui.end_row();
                ui.label("Brauer:in");
                ui.text_edit_singleline(&mut dlg.spec.author);
                ui.end_row();
                ui.label("Brauerei");
                ui.text_edit_singleline(&mut dlg.spec.brewery);
                ui.end_row();
                ui.label("Anstellvolumen (L)");
                crate::widgets::number(ui, "new.vol", &mut dlg.spec.initial_volume, 80.0);
                ui.end_row();
                ui.label("Angestellt am");
                crate::datetime::edit(ui, "new.date", &mut dlg.spec.date_created);
                ui.end_row();
            });
            if let Some(e) = &dlg.error {
                ui.label(RichText::new(e).color(theme::ERROR));
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                create = ui.add(theme::primary_button("Anlegen")).clicked();
                cancel = ui.button("Abbrechen").clicked();
            });
        });
        if resp.should_close() {
            cancel = true;
        }
        if cancel {
            self.new_dialog = None;
            return;
        }
        if !create {
            return;
        }
        let Some(repo) = &self.repo else { return };
        let spec = &dlg.spec;
        let path = repo.join("batches").join(format!("{}.json", spec.identifier));
        let error = if !model::is_valid_identifier(&spec.identifier) {
            Some("Kennung: nur Großbuchstaben, Ziffern und Bindestriche.")
        } else if existing.contains(&spec.identifier) || path.exists() {
            Some("Diese Kennung existiert bereits.")
        } else if spec.name.trim().is_empty() {
            Some("Bitte einen Namen angeben.")
        } else if !model::is_valid_datetime(&spec.date_created) {
            Some("Datum muss ISO-8601 mit Zeitzone sein.")
        } else {
            None
        };
        if let Some(e) = error {
            dlg.error = Some(e.into());
            return;
        }
        let doc = BatchDoc::new_from_template(path, model::template(spec));
        self.docs.push(doc);
        self.docs.sort_by(|a, b| a.file_id.cmp(&b.file_id));
        let id = self.new_dialog.take().map(|d| d.spec.identifier).unwrap_or_default();
        self.selected = self.docs.iter().position(|d| d.file_id == id);
        self.notify(format!("{id} angelegt – noch nicht gespeichert. Zutaten ergänzen nicht vergessen."), false);
    }

    fn confirm_dialogs(&mut self, ctx: &egui::Context) {
        if self.confirm_close {
            let mut choice = None;
            egui::Modal::new(egui::Id::unique("confirm_close")).show(ctx, |ui| {
                ui.label(RichText::new("Ungespeicherte Änderungen").size(17.0).strong());
                ui.label(format!("{} Sud(e) haben ungespeicherte Änderungen.", self.dirty_count()));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.add(theme::primary_button("Speichern & beenden")).clicked() {
                        choice = Some(0);
                    }
                    if ui.button("Verwerfen").clicked() {
                        choice = Some(1);
                    }
                    if ui.button("Abbrechen").clicked() {
                        choice = Some(2);
                    }
                });
            });
            match choice {
                Some(0) => {
                    if self.save_all() {
                        self.allow_close = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    self.confirm_close = false;
                }
                Some(1) => {
                    self.allow_close = true;
                    self.confirm_close = false;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                Some(_) => self.confirm_close = false,
                None => {}
            }
        }
        if self.confirm_reload {
            let mut choice = None;
            egui::Modal::new(egui::Id::unique("confirm_reload")).show(ctx, |ui| {
                ui.label(RichText::new("Änderungen verwerfen?").size(17.0).strong());
                ui.label("Der Sud wird von der Festplatte neu geladen; ungespeicherte Änderungen gehen verloren.");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Verwerfen & neu laden").clicked() {
                        choice = Some(true);
                    }
                    if ui.button("Abbrechen").clicked() {
                        choice = Some(false);
                    }
                });
            });
            if let Some(c) = choice {
                self.confirm_reload = false;
                if c {
                    self.reload_selected();
                }
            }
        }
    }
}

impl eframe::App for BrauApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_runner();
        self.poll_imports();
        for uri in self.ctx_forget.drain(..) {
            if uri.is_empty() {
                ctx.forget_all_images();
            } else {
                ctx.forget_image(&uri);
            }
        }
        self.validate_all();

        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| images::is_image_file(p)).collect()
        });
        if !dropped.is_empty() {
            match self.selected.map(|i| self.docs[i].file_id.clone()) {
                Some(_) if self.import_rx.is_some() => self.notify("Es läuft bereits ein Import.", true),
                Some(id) => self.start_import(&ctx, id, tabs::ImportRequest { paths: dropped, step: None }),
                None => self.notify("Erst links einen Sud wählen, dann Fotos hineinziehen.", true),
            }
        }
        let hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());

        if let Some((_, _, t)) = &mut self.notice {
            let now = ctx.input(|i| i.time);
            if *t == 0.0 {
                *t = now;
            } else if now - *t > 8.0 {
                self.notice = None;
            }
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::S))) {
            self.save_all();
        }
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty_count() > 0 && !self.allow_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_close = true;
        }

        let bar = egui::Frame::new().fill(theme::SURFACE).inner_margin(egui::Margin::symmetric(12, 8));
        egui::Panel::top("top").frame(bar).show(ui, |ui| self.top_bar(ui));
        let side = egui::Frame::new().fill(theme::BG_PAGE).inner_margin(egui::Margin::symmetric(10, 6));
        egui::Panel::left("batches")
            .resizable(true)
            .default_size(250.0)
            .frame(side)
            .show(ui, |ui| self.batch_list(ui));
        egui::Panel::right("git")
            .resizable(true)
            .default_size(380.0)
            .frame(side)
            .show(ui, |ui| self.git_panel(ui));
        if self.selected.is_some() {
            egui::Panel::bottom("issues")
                .resizable(true)
                .default_size(110.0)
                .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(egui::Margin::symmetric(12, 4)))
                .show(ui, |ui| self.issues_panel(ui));
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BG_PAGE).inner_margin(egui::Margin::symmetric(16, 10)))
            .show(ui, |ui| self.editor(ui));

        if hovering && self.selected.is_some() {
            let rect = ctx.content_rect();
            let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::unique("drop")));
            painter.rect_filled(rect, 0.0, theme::AMBER_TINT.gamma_multiply(0.85));
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Fotos loslassen zum Importieren",
                egui::FontId::proportional(26.0),
                theme::AMBER_HOVER,
            );
        }
        self.new_dialog_ui(&ctx);
        self.confirm_dialogs(&ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Some(repo) = &self.repo {
            storage.set_string(STORAGE_REPO, repo.display().to_string());
        }
        storage.set_string(STORAGE_EDGE, self.max_edge.to_string());
    }
}
