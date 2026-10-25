//! 05 §1/§3/§20: the static check runner and its report. `check --stage Bn`
//! runs every `check_*` whose subject exists at that stage, writes
//! `gates/checks/stage-Bn.json`, and lists the disabled ids with the stage they
//! turn on. The spec calls these engineering-only legs (OD-53); they need no
//! session.

pub mod checks;
pub mod gates;
pub mod panel;
pub mod runner;
pub mod scenario;
pub mod stages;
pub mod sweep;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Serialize;

pub use checks::CheckOutcome;
pub use stages::Stage;

/// The build identity a check report carries (05 §1), minus the run-only fields:
/// a static check has no transcript, seed or variant.
#[derive(Clone, Debug, Serialize)]
pub struct BuildIdentity {
    pub hash_format: u16,
    pub seed_format: u16,
    pub consts_version: u16,
    pub build_variant: &'static str,
    pub cargo_lock_md5: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DisabledRecord {
    pub id: &'static str,
    pub turns_on_at: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct StageReport {
    pub stage: String,
    pub pass: bool,
    pub enabled: Vec<CheckRecord>,
    pub disabled: Vec<DisabledRecord>,
    pub placeholders: Vec<si_core::consts::Placeholder>,
    pub build: BuildIdentity,
}

#[derive(Clone, Debug, Serialize)]
pub struct CheckRecord {
    pub id: &'static str,
    pub verdict: &'static str,
    pub detail: String,
}

/// A read-only view of the workspace for source scans. The harness is the only
/// crate allowed to open `gates/` (05 §1); this struct also reads the crates'
/// sources, which are not loop inputs.
#[derive(Clone, Debug)]
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    pub fn locate() -> Result<Self> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .context("locate the workspace root from si-cli's manifest dir")?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    pub fn read(&self, relative: &str) -> Result<String> {
        let path = self.path(relative);
        fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))
    }

    pub fn exists(&self, relative: &str) -> bool {
        self.path(relative).exists()
    }

    /// The thirteen package names of 00 §2, in manifest order.
    pub fn crate_names(&self) -> Result<Vec<String>> {
        let manifest = self.read("Cargo.toml")?;
        let mut names = Vec::new();
        let mut in_members = false;
        for line in manifest.lines() {
            let line = line.trim();
            if line.starts_with("members") {
                in_members = true;
            }
            if in_members {
                for piece in line.split('"').skip(1).step_by(2) {
                    if let Some(name) = piece.rsplit('/').next() {
                        names.push(name.to_string());
                    }
                }
            }
            if in_members && line.contains(']') {
                break;
            }
        }
        if names.is_empty() {
            bail!("no members parsed from the root Cargo.toml");
        }
        Ok(names)
    }

    pub fn crate_manifest(&self, name: &str) -> Result<String> {
        self.read(&format!("crates/{name}/Cargo.toml"))
    }

    /// Every `.rs` under `crates/<name>/src` and `crates/<name>/tests`, sorted by
    /// path so scan order is deterministic.
    pub fn source_files(&self) -> Result<Vec<PathBuf>> {
        let mut out = Vec::new();
        let crates_dir = self.path("crates");
        for entry in fs::read_dir(&crates_dir).with_context(|| format!("read {}", crates_dir.display()))? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            for sub in ["src", "tests"] {
                let dir = entry.path().join(sub);
                if dir.is_dir() {
                    collect_rs(&dir, &mut out)?;
                }
            }
        }
        out.sort();
        Ok(out)
    }

    pub fn relative(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
    }

    pub fn source_text(&self) -> Result<Vec<(String, String)>> {
        self.source_files()?
            .into_iter()
            .map(|path| {
                let text = fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?;
                Ok((self.relative(&path), text))
            })
            .collect()
    }

    /// True when a source path is the auditor itself: the harness holds the
    /// label words and the vocabulary allowlist in order to audit them.
    pub fn is_auditor(relative: &str) -> bool {
        relative.starts_with("crates/si-cli/src/harness/")
    }
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_rs(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

pub fn build_identity(ws: &Workspace) -> BuildIdentity {
    let cargo_lock_md5 = fs::read(ws.path("Cargo.lock")).ok().map(|bytes| {
        use md5::{Digest, Md5};
        si_core::hash::hex(Md5::digest(bytes).as_slice())
    });
    BuildIdentity {
        hash_format: si_core::hash::HASH_FORMAT,
        seed_format: si_core::seed::SEED_FORMAT,
        consts_version: si_core::consts::CONSTS_VERSION,
        build_variant: "default",
        cargo_lock_md5,
    }
}

/// OD-93 (owner ruling 2026-10-23, B7 batch OD-E): a variant is a build-graph state, pinned by
/// the deps_lock digest and not by a constants-registry row. The determinism
/// clause reads per-variant, so every run, replay and export report carries this
/// pair — without it two hashes are not comparable things.
pub fn build_stamp(ws: &Workspace, variant: &str) -> serde_json::Value {
    let base = build_identity(ws);
    serde_json::json!({
        "build_variant": variant,
        "graph_digest": base.cargo_lock_md5,
    })
}

pub fn report_path(stage: Stage) -> PathBuf {
    PathBuf::from(format!("gates/checks/stage-{}.json", stage.label()))
}

pub fn run_stage(stage: Stage, ws: &Workspace) -> Result<StageReport> {
    let mut enabled = Vec::new();
    let mut failed = false;
    for id in stage.checks() {
        let outcome = checks::run(id, ws)?;
        if outcome.verdict != "pass" {
            failed = true;
        }
        enabled.push(CheckRecord {
            id: outcome.id,
            verdict: outcome.verdict,
            detail: outcome.detail,
        });
    }
    let disabled = Stage::ALL
        .into_iter()
        .filter(|later| *later > stage)
        .flat_map(|later| {
            later.introduced().into_iter().map(move |id| DisabledRecord {
                id,
                turns_on_at: later.label(),
            })
        })
        .collect();
    Ok(StageReport {
        stage: stage.label().to_string(),
        pass: !failed,
        enabled,
        disabled,
        placeholders: si_core::consts::placeholders(),
        build: build_identity(ws),
    })
}

pub fn run_all(ws: &Workspace) -> Result<StageReport> {
    run_stage(Stage::union(), ws)
}

pub fn write_report(report: &StageReport, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(report)?;
    fs::write(path, json).with_context(|| format!("write {}", path.display()))
}
