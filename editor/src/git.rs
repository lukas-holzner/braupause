//! Git- und npm-Aufrufe im Hintergrund-Thread, damit die Oberfläche nicht blockiert.
//! Genutzt wird das installierte `git`-CLI, also mit deinen vorhandenen
//! Zugangsdaten (SSH-Agent, Credential-Helper, Keychain).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{Receiver, Sender, channel};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Command,
    Output,
    Error,
    Success,
}

pub enum Msg {
    Line(LineKind, String),
    Status(RepoStatus),
    Done { ok: bool },
}

#[derive(Clone, Default)]
pub struct RepoStatus {
    pub branch: String,
    pub upstream: Option<String>,
    pub remotes: Vec<String>,
    /// (Status-Kürzel, Pfad)
    pub files: Vec<(String, String)>,
    pub ahead: u32,
    pub has_commits: bool,
}

pub struct CommitPlan {
    pub files: Vec<String>,
    pub message: String,
    pub run_npm_test: bool,
    pub commit: bool,
    pub push: bool,
}

pub struct Runner {
    rx: Option<Receiver<Msg>>,
}

impl Runner {
    pub fn new() -> Self {
        Self { rx: None }
    }

    pub fn busy(&self) -> bool {
        self.rx.is_some()
    }

    /// Liefert alle neuen Nachrichten; `None` beim Abschluss ist nicht nötig, `Done` signalisiert das Ende.
    pub fn poll(&mut self) -> Vec<Msg> {
        let mut out = Vec::new();
        if let Some(rx) = &self.rx {
            while let Ok(m) = rx.try_recv() {
                let done = matches!(m, Msg::Done { .. });
                out.push(m);
                if done {
                    self.rx = None;
                    break;
                }
            }
        }
        out
    }

    fn spawn(&mut self, ctx: &egui::Context, job: impl FnOnce(&Log) -> Result<(), String> + Send + 'static) {
        let (tx, rx) = channel();
        self.rx = Some(rx);
        let log = Log { tx, ctx: ctx.clone() };
        std::thread::spawn(move || {
            let result = job(&log);
            if let Err(e) = &result {
                log.line(LineKind::Error, e.clone());
            }
            log.send(Msg::Done { ok: result.is_ok() });
        });
    }

    pub fn refresh_status(&mut self, ctx: &egui::Context, repo: PathBuf) {
        self.spawn(ctx, move |log| {
            log.send(Msg::Status(status(&repo)?));
            Ok(())
        });
    }

    pub fn npm_test(&mut self, ctx: &egui::Context, repo: PathBuf) {
        self.spawn(ctx, move |log| {
            npm_test(&repo, log)?;
            log.line(LineKind::Success, "npm test erfolgreich.".into());
            Ok(())
        });
    }

    pub fn pull(&mut self, ctx: &egui::Context, repo: PathBuf) {
        self.spawn(ctx, move |log| {
            git(&repo, &["pull", "--ff-only"], log)?;
            log.send(Msg::Status(status(&repo)?));
            Ok(())
        });
    }

    pub fn commit_and_push(&mut self, ctx: &egui::Context, repo: PathBuf, plan: CommitPlan) {
        self.spawn(ctx, move |log| {
            let result = run_plan(&repo, &plan, log);
            if let Ok(s) = status(&repo) {
                log.send(Msg::Status(s));
            }
            result
        });
    }
}

fn run_plan(repo: &Path, plan: &CommitPlan, log: &Log) -> Result<(), String> {
    if plan.commit {
        if plan.files.is_empty() {
            return Err("Keine Dateien zum Committen ausgewählt.".into());
        }
        if plan.message.trim().is_empty() {
            return Err("Commit-Nachricht fehlt.".into());
        }
    }
    if plan.run_npm_test {
        npm_test(repo, log)?;
    }
    if plan.commit {
        let mut add = vec!["add", "--all", "--"];
        add.extend(plan.files.iter().map(String::as_str));
        git(repo, &add, log)?;
        let mut commit = vec!["commit", "-m", plan.message.trim(), "--"];
        commit.extend(plan.files.iter().map(String::as_str));
        git(repo, &commit, log)?;
    }
    if plan.push {
        push(repo, log)?;
    }
    log.line(LineKind::Success, "Fertig.".into());
    Ok(())
}

fn push(repo: &Path, log: &Log) -> Result<(), String> {
    let remotes = quiet(repo, &["remote"]).unwrap_or_default();
    let remote = remotes.lines().next().map(str::to_string);
    let Some(remote) = remote else {
        return Err(
            "Kein Git-Remote konfiguriert. Einmalig im Terminal: git remote add origin <URL>".into(),
        );
    };
    let has_upstream = quiet(repo, &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"]).is_ok();
    if has_upstream {
        git(repo, &["push"], log)
    } else {
        let remote = if remotes.lines().any(|r| r == "origin") { "origin".to_string() } else { remote };
        git(repo, &["push", "-u", &remote, "HEAD"], log)
    }
}

pub fn status(repo: &Path) -> Result<RepoStatus, String> {
    let branch = quiet(repo, &["branch", "--show-current"])?.trim().to_string();
    let upstream = quiet(repo, &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .ok()
        .map(|s| s.trim().to_string());
    let remotes = quiet(repo, &["remote"]).unwrap_or_default().lines().map(str::to_string).collect();
    let has_commits = quiet(repo, &["rev-parse", "--verify", "HEAD"]).is_ok();
    let ahead = if upstream.is_some() {
        quiet(repo, &["rev-list", "--count", "@{u}..HEAD"])
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
    } else {
        0
    };
    let porcelain = quiet(repo, &["status", "--porcelain=v1", "-uall", "-z"])?;
    let mut files = Vec::new();
    let mut parts = porcelain.split('\0').filter(|s| !s.is_empty());
    while let Some(entry) = parts.next() {
        if entry.len() < 4 {
            continue;
        }
        let code = entry[..2].to_string();
        let path = entry[3..].to_string();
        // Bei Umbenennungen folgt der alte Pfad als eigener Eintrag.
        if code.starts_with('R') || code.starts_with('C') {
            parts.next();
        }
        files.push((code, path));
    }
    Ok(RepoStatus { branch, upstream, remotes, files, ahead, has_commits })
}

pub struct Log {
    tx: Sender<Msg>,
    ctx: egui::Context,
}

impl Log {
    fn send(&self, m: Msg) {
        let _ = self.tx.send(m);
        self.ctx.request_repaint();
    }
    fn line(&self, kind: LineKind, text: String) {
        self.send(Msg::Line(kind, text));
    }
}

fn git_cmd(repo: &Path) -> Command {
    let mut c = Command::new("git");
    c.current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .stdin(Stdio::null());
    c
}

/// Git-Aufruf ohne Log-Ausgabe (für Statusabfragen).
fn quiet(repo: &Path, args: &[&str]) -> Result<String, String> {
    let out = git_cmd(repo).args(args).output().map_err(|e| format!("git nicht ausführbar: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

fn git(repo: &Path, args: &[&str], log: &Log) -> Result<(), String> {
    log.line(LineKind::Command, format!("$ git {}", args.join(" ")));
    let out = git_cmd(repo).args(args).output().map_err(|e| format!("git nicht ausführbar: {e}"))?;
    emit(&out, log);
    if out.status.success() {
        Ok(())
    } else {
        Err(format!("git {} fehlgeschlagen ({}).", args[0], out.status))
    }
}

/// `npm test` über die Login-Shell, damit nvm/Homebrew-Pfade auch beim Start aus dem Finder greifen.
fn npm_test(repo: &Path, log: &Log) -> Result<(), String> {
    log.line(LineKind::Command, "$ npm test".into());
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
    let script = "if [ ! -d node_modules ]; then npm install --no-audit --no-fund; fi && npm test";
    let mut out = Command::new(&shell)
        .args(["-lc", script])
        .current_dir(repo)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("Shell nicht ausführbar: {e}"))?;
    if out.status.code() == Some(127) {
        // npm nur in der interaktiven Shell-Konfiguration (z. B. ~/.zshrc) bekannt.
        out = Command::new(&shell)
            .args(["-ilc", script])
            .current_dir(repo)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| format!("Shell nicht ausführbar: {e}"))?;
    }
    emit(&out, log);
    if out.status.success() {
        Ok(())
    } else {
        Err("npm test fehlgeschlagen – Commit abgebrochen.".into())
    }
}

fn emit(out: &std::process::Output, log: &Log) {
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        log.line(LineKind::Output, line.to_string());
    }
    for line in String::from_utf8_lossy(&out.stderr).lines() {
        let kind = if out.status.success() { LineKind::Output } else { LineKind::Error };
        log.line(kind, line.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sh(dir: &Path, args: &[&str]) {
        let ok = Command::new("git").args(args).current_dir(dir).output().unwrap();
        assert!(ok.status.success(), "{args:?}: {}", String::from_utf8_lossy(&ok.stderr));
    }

    /// Erster Commit in leerem Repo, nur ausgewählte Dateien, Push mit Upstream-Einrichtung.
    #[test]
    fn commit_selected_files_and_push() {
        let base = std::env::temp_dir().join(format!("braupause-git-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let (repo, remote) = (base.join("repo"), base.join("remote.git"));
        std::fs::create_dir_all(repo.join("batches")).unwrap();
        sh(&base, &["init", "--bare", "-q", "remote.git"]);
        sh(&repo, &["init", "-q", "-b", "main"]);
        sh(&repo, &["config", "user.email", "test@example.invalid"]);
        sh(&repo, &["config", "user.name", "Test"]);
        sh(&repo, &["remote", "add", "origin", remote.to_str().unwrap()]);
        std::fs::write(repo.join("batches/MET-2026-09.json"), "{}\n").unwrap();
        std::fs::write(repo.join("notes.txt"), "nicht committen\n").unwrap();

        let s = status(&repo).unwrap();
        assert!(!s.has_commits && s.upstream.is_none());
        assert_eq!(s.files.len(), 2);

        let (tx, rx) = channel();
        let log = Log { tx, ctx: egui::Context::default() };
        let plan = CommitPlan {
            files: vec!["batches/MET-2026-09.json".into()],
            message: "Neuer Sud MET-2026-09".into(),
            run_npm_test: false,
            commit: true,
            push: true,
        };
        let result = run_plan(&repo, &plan, &log);
        let lines: Vec<String> = rx.try_iter().filter_map(|m| match m { Msg::Line(_, l) => Some(l), _ => None }).collect();
        assert!(result.is_ok(), "{result:?}\n{}", lines.join("\n"));

        let s = status(&repo).unwrap();
        assert!(s.has_commits);
        assert_eq!(s.upstream.as_deref(), Some("origin/main"));
        assert_eq!(s.files, vec![("??".to_string(), "notes.txt".to_string())]);
        let pushed = quiet(&remote, &["log", "--format=%s", "main"]).unwrap();
        assert_eq!(pushed.trim(), "Neuer Sud MET-2026-09");

        // Zweiter Durchlauf: Änderung committen, normaler Push mit Upstream.
        std::fs::write(repo.join("batches/MET-2026-09.json"), "{\"a\":1}\n").unwrap();
        let plan = CommitPlan { message: "Protokoll aktualisiert".into(), ..plan };
        run_plan(&repo, &plan, &log).unwrap();
        assert_eq!(status(&repo).unwrap().ahead, 0);
        let _ = std::fs::remove_dir_all(&base);
    }
}
