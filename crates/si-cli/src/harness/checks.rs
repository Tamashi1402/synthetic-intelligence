//! 05 §20: the static legs. Each check is a real scan or a structural assertion,
//! never a stub: a check that cannot run yet is disabled by the stage table
//! instead of reporting green.

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use md5::{Digest, Md5};

use super::stages::Stage;
use super::Workspace;

/// The complete check set of 05 §20, in the registry's order. `--all` runs
/// every entry; a stage runs the prefix its stage table enables.
pub const CHECK_IDS: [&'static str; 52] = [
    "check_audit_edges",
    "check_bake_is_the_exception",
    "check_bake_no_event_kind",
    "check_bake_write_once",
    "check_calm_view_is_terminal",
    "check_candidate_unanswerable",
    "check_compose_writes_nothing",
    "check_config_guard",
    "check_crystallize_no_commit",
    "check_curriculum_is_inbox_only",
    "check_deps",
    "check_drive_doors",
    "check_emotion_words",
    "check_ghost_is_frame_scoped",
    "check_gui_observer_only",
    "check_headless_run",
    "check_hop_cap_is_claim_side",
    "check_idle_no_commit",
    "check_membrane",
    "check_no_affective_setter",
    "check_no_embeddings",
    "check_no_json_readback",
    "check_no_mood_field",
    "check_no_seed_control_surface",
    "check_no_stage_on_supporter",
    "check_no_teach_vocabulary",
    "check_no_viewer_state_write",
    "check_nomination_owner_lane",
    "check_one_clock",
    "check_prov_link_not_stored",
    "check_record_layout",
    "check_registry_size",
    "check_relight_writes_nothing",
    "check_signal_kinds",
    "check_skills_no_commit",
    "check_silt_ignition_only",
    "check_silt_writes_nothing",
    "check_sleep_never_brightens",
    "check_sleep_writes_nothing",
    "check_speaker_class_derived",
    "check_style_never_installed",
    "check_sweep_never_touches_temperament",
    "check_string_constants",
    "check_teach_left_side",
    "check_teach_literals",
    "check_temperament_closed",
    "check_tongue_closed",
    "check_tongue_is_leaf",
    "check_tongue_license_only",
    "check_tongue_reads_no_a",
    "check_tongue_writes_nothing",
    "check_windows_suppress_only",
];

/// The stage that turns a check on (11 §4's C0–C10 sets).
pub fn enable_stage(id: &'static str) -> Stage {
    match id {
        "check_deps"
        | "check_emotion_words"
        | "check_membrane"
        | "check_no_embeddings"
        | "check_no_json_readback"
        | "check_record_layout"
        | "check_registry_size"
        | "check_string_constants"
        | "check_one_clock"
        | "check_teach_literals"
        | "check_temperament_closed"
        | "check_sweep_never_touches_temperament" => Stage::B0,
        "check_config_guard" | "check_bake_is_the_exception" | "check_no_mood_field" => Stage::B1,
        "check_signal_kinds" | "check_bake_no_event_kind" | "check_drive_doors" => Stage::B2,
        "check_idle_no_commit" => Stage::B3,
        "check_teach_left_side"
        | "check_no_teach_vocabulary"
        | "check_hop_cap_is_claim_side"
        | "check_candidate_unanswerable" => Stage::B4,
        "check_bake_write_once"
        | "check_compose_writes_nothing"
        | "check_curriculum_is_inbox_only"
        | "check_no_seed_control_surface" => Stage::B5,
        "check_audit_edges" => Stage::B6,
        "check_sleep_writes_nothing" | "check_sleep_never_brightens" | "check_calm_view_is_terminal" => {
            Stage::B8
        }
        "check_no_affective_setter" | "check_no_stage_on_supporter" => Stage::B9,
        "check_crystallize_no_commit" | "check_ghost_is_frame_scoped" => Stage::B14,
        "check_silt_writes_nothing" | "check_silt_ignition_only" => Stage::B15,
        "check_prov_link_not_stored" | "check_relight_writes_nothing" => Stage::B16,
        "check_skills_no_commit" | "check_nomination_owner_lane" => Stage::B17,
        "check_speaker_class_derived" | "check_windows_suppress_only" => Stage::B18,
        "check_gui_observer_only" | "check_no_viewer_state_write" | "check_headless_run" => Stage::B11,
        "check_style_never_installed"
        | "check_tongue_closed"
        | "check_tongue_is_leaf"
        | "check_tongue_license_only"
        | "check_tongue_reads_no_a"
        | "check_tongue_writes_nothing" => Stage::B20,
        other => unreachable!("unknown check id {other}"),
    }
}

#[derive(Clone, Debug)]
pub struct CheckOutcome {
    pub id: &'static str,
    pub verdict: &'static str,
    pub detail: String,
}

pub fn run(id: &'static str, ws: &Workspace) -> Result<CheckOutcome> {
    let result: Result<String> = match id {
        "check_deps" => check_deps(ws),
        "check_emotion_words" => check_emotion_words(ws),
        "check_membrane" => check_membrane(ws),
        "check_no_embeddings" => check_no_embeddings(ws),
        "check_no_json_readback" => check_no_json_readback(ws),
        "check_record_layout" => check_record_layout(ws),
        "check_registry_size" => check_registry_size(ws),
        "check_string_constants" => check_string_constants(ws),
        "check_one_clock" => check_one_clock(ws),
        "check_teach_literals" => check_teach_literals(ws),
        "check_temperament_closed" => check_temperament_closed(ws),
        "check_sweep_never_touches_temperament" => check_sweep_never_touches_temperament(ws),
        "check_config_guard" => check_config_guard(ws),
        "check_bake_is_the_exception" => check_bake_is_the_exception(ws),
        "check_no_mood_field" => check_no_mood_field(ws),
        "check_signal_kinds" => check_signal_kinds(ws),
        "check_bake_no_event_kind" => check_bake_no_event_kind(ws),
        "check_drive_doors" => check_drive_doors(ws),
        "check_idle_no_commit" => check_idle_no_commit(ws),
        "check_teach_left_side" => check_teach_left_side(ws),
        "check_no_teach_vocabulary" => check_no_teach_vocabulary(ws),
        "check_hop_cap_is_claim_side" => check_hop_cap_is_claim_side(ws),
        "check_candidate_unanswerable" => check_candidate_unanswerable(ws),
        "check_bake_write_once" => check_bake_write_once(ws),
        "check_compose_writes_nothing" => check_compose_writes_nothing(ws),
        "check_curriculum_is_inbox_only" => check_curriculum_is_inbox_only(ws),
        "check_no_seed_control_surface" => check_no_seed_control_surface(ws),
        "check_audit_edges" => check_audit_edges(ws),
        "check_sleep_writes_nothing" => check_sleep_writes_nothing(ws),
        "check_sleep_never_brightens" => check_sleep_never_brightens(ws),
        "check_calm_view_is_terminal" => check_calm_view_is_terminal(ws),
        "check_no_affective_setter" => check_no_affective_setter(ws),
        "check_no_stage_on_supporter" => check_no_stage_on_supporter(ws),
        "check_crystallize_no_commit" => check_crystallize_no_commit(ws),
        "check_ghost_is_frame_scoped" => check_ghost_is_frame_scoped(ws),
        "check_silt_writes_nothing" => check_silt_writes_nothing(ws),
        "check_silt_ignition_only" => check_silt_ignition_only(ws),
        "check_prov_link_not_stored" => check_prov_link_not_stored(ws),
        "check_relight_writes_nothing" => check_relight_writes_nothing(ws),
        "check_skills_no_commit" => check_skills_no_commit(ws),
        "check_nomination_owner_lane" => check_nomination_owner_lane(ws),
        "check_speaker_class_derived" => check_speaker_class_derived(ws),
        "check_windows_suppress_only" => check_windows_suppress_only(ws),
        "check_gui_observer_only" => check_gui_observer_only(ws),
        "check_no_viewer_state_write" => check_no_viewer_state_write(ws),
        "check_headless_run" => check_headless_run(ws),
        "check_style_never_installed" => check_style_never_installed(ws),
        "check_tongue_closed" => check_tongue_closed(ws),
        "check_tongue_is_leaf" => check_tongue_is_leaf(ws),
        "check_tongue_license_only" => check_tongue_license_only(ws),
        "check_tongue_reads_no_a" => check_tongue_reads_no_a(ws),
        "check_tongue_writes_nothing" => check_tongue_writes_nothing(ws),
        _ => bail!("{id} turns on at {}; its leg is not implemented yet", enable_stage(id).label()),
    };
    Ok(match result {
        Ok(detail) => CheckOutcome { id, verdict: "pass", detail },
        Err(error) => CheckOutcome { id, verdict: "fail", detail: format!("{error:#}") },
    })
}

// ---------------------------------------------------------------- helpers

const CRATE_NAMES: [&str; 17] = [
    "si-core",
    "si-store",
    "si-affect",
    "si-walk",
    "si-learn",
    "si-read",
    "si-front",
    "si-supporter",
    "si-vulkan",
    "si-sleep",
    "si-audit",
    "si-session",
    "si-crystallize",
    "si-silt",
    "si-skills",
    "si-tongue",
    "si-cli",
];

const EXTERNAL_DEPS: [&str; 9] = [
    "anyhow",
    "serde",
    "serde_json",
    "bytemuck",
    "clap",
    "md-5",
    "ash",
    "winit",
    "raw-window-handle",
];

/// A dependency whose name suggests a shipped lexicon, corpus or numeric model
/// is a build failure regardless of what it does (G2, R-14).
fn forbidden_dependency(name: &str) -> Option<&'static str> {
    const BANNED: [&str; 12] = [
        "wordnet",
        "conceptnet",
        "embedding",
        "onnx",
        "tokenizers",
        "transformer",
        "bert",
        "lexicon",
        "corpus",
        "llama",
        "ggml",
        "fasttext",
    ];
    let lower = name.to_ascii_lowercase();
    BANNED.into_iter().find(|needle| lower.contains(needle))
}

fn dependency_keys(manifest: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut section = String::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            section = line.trim_matches(|c| c == '[' || c == ']').to_string();
            continue;
        }
        let dependency_section = matches!(
            section.as_str(),
            "dependencies" | "dev-dependencies" | "build-dependencies" | "workspace.dependencies"
        );
        if !dependency_section {
            continue;
        }
        if let Some((key, _)) = line.split_once('=') {
            let key = key.trim();
            // `si-core.workspace = true` is how every crate declares an edge, so
            // the dotted form is an edge name, not a section key. Dropping it
            // here would make every manifest assertion below read an empty graph
            // and pass over nothing.
            let key = key.strip_suffix(".workspace").unwrap_or(key);
            if !key.is_empty() && !key.contains(' ') && !key.contains('.') {
                keys.push(key.to_string());
            }
        }
    }
    keys
}

/// Double-quoted literals, ignoring `//` line comments. Good enough for the
/// audit scans, which never need macro-expansion.
fn string_literals(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let mut in_string = false;
        let mut escaped = false;
        let mut start = 0usize;
        let bytes = line.as_bytes();
        let mut index = 0usize;
        while index < bytes.len() {
            let byte = bytes[index];
            if !in_string {
                if byte == b'/' && bytes.get(index + 1) == Some(&b'/') {
                    break;
                }
                if byte == b'"' {
                    in_string = true;
                    start = index + 1;
                }
            } else if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                out.push(line[start..index].to_string());
                in_string = false;
            }
            index += 1;
        }
    }
    out
}

fn contains_word(haystack: &str, word: &str) -> bool {
    let bytes = haystack.as_bytes();
    let target = word.as_bytes();
    if target.is_empty() || target.len() > bytes.len() {
        return false;
    }
    let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut start = 0;
    while let Some(found) = haystack[start..].find(word) {
        let at = start + found;
        let before_ok = at == 0 || !is_word(bytes[at - 1]);
        let end = at + target.len();
        let after_ok = end == bytes.len() || !is_word(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        start = at + 1;
    }
    false
}

fn md5_hex(bytes: &[u8]) -> String {
    si_core::hash::hex(Md5::digest(bytes).as_slice())
}

// ---------------------------------------------------------------- checks

fn check_deps(ws: &Workspace) -> Result<String> {
    let mut names = ws.crate_names()?;
    names.sort();
    let mut expected: Vec<String> = CRATE_NAMES.iter().map(|n| n.to_string()).collect();
    expected.sort();
    if names != expected {
        bail!("workspace members are {names:?}, want 00 §2's {expected:?}");
    }

    let mut allowed: Vec<String> = CRATE_NAMES.iter().map(|n| n.to_string()).collect();
    allowed.extend(EXTERNAL_DEPS.iter().map(|n| n.to_string()));

    let mut declarations = 0usize;
    let root = ws.read("Cargo.toml")?;
    let mut manifests = vec![("Cargo.toml".to_string(), root)];
    for name in &names {
        manifests.push((format!("crates/{name}/Cargo.toml"), ws.crate_manifest(name)?));
    }
    for (path, manifest) in &manifests {
        for key in dependency_keys(manifest) {
            declarations += 1;
            if !allowed.contains(&key) {
                bail!("{path} depends on `{key}`, which is not on the allowlist");
            }
            if let Some(needle) = forbidden_dependency(&key) {
                bail!("{path} depends on `{key}` (contains `{needle}`): no shipped lexicon or model");
            }
        }
    }

    let lock_path = ws.path("Cargo.lock");
    let lock = fs::read(&lock_path).with_context(|| format!("read {}", lock_path.display()))?;
    let actual = md5_hex(&lock);
    let published = ws
        .read("gates/deps_lock.md5")
        .context("publish the dependency allowlist value at gates/deps_lock.md5")?
        .trim()
        .to_string();
    if actual != published {
        bail!("Cargo.lock md5 {actual} does not match the published value {published}");
    }
    let lock_text = String::from_utf8_lossy(&lock);
    for line in lock_text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("name = \"") {
            let name = rest.trim_end_matches('"');
            if let Some(needle) = forbidden_dependency(name) {
                bail!("Cargo.lock contains `{name}` (contains `{needle}`)");
            }
        }
    }
    Ok(format!(
        "{} crates; {declarations} dependency declarations, all allowlisted; Cargo.lock md5 {actual} matches gates/deps_lock.md5; no lexicon/corpus/model package",
        names.len()
    ))
}

fn check_emotion_words(ws: &Workspace) -> Result<String> {
    const LABEL_WORDS: [&str; 6] =
        ["curiosity", "surprise", "satisfaction", "caution", "boredom", "frustration"];
    let mut hits = Vec::new();
    for (relative, text) in ws.source_text()? {
        if relative.starts_with("crates/si-audit/") || Workspace::is_auditor(&relative) {
            continue;
        }
        let lower = text.to_ascii_lowercase();
        for word in LABEL_WORDS {
            if contains_word(&lower, word) {
                hits.push(format!("{relative}: {word}"));
            }
        }
    }
    if !hits.is_empty() {
        bail!("label words outside si-audit: {}", hits.join("; "));
    }
    Ok(format!(
        "six label words appear in si-audit only; si-affect, si-walk, si-front and the {} file(s) of every \
         other member — `si-tongue` among them (13 §6.3) — are wordless",
        ws.source_files()?
            .iter()
            .filter(|path| {
                let relative = ws.relative(path);
                !relative.starts_with("crates/si-audit/") && !Workspace::is_auditor(&relative)
            })
            .count()
    ))
}

fn check_membrane(ws: &Workspace) -> Result<String> {
    let names = ws.crate_names()?;
    if names.iter().any(|name| name == "grimm") {
        bail!("`grimm` is a workspace member; the supporter is a separate process, not a crate");
    }
    let needles = ["grimm/", "grimm::", "use grimm", "mod grimm", "extern crate grimm"];
    let mut hits = Vec::new();
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) {
            continue;
        }
        for needle in needles {
            if text.contains(needle) {
                hits.push(format!("{relative}: `{needle}`"));
            }
        }
    }
    for name in &names {
        let manifest = ws.crate_manifest(name)?;
        if manifest.contains("name = \"grimm\"") {
            hits.push(format!("crates/{name}/Cargo.toml: name = \"grimm\""));
        }
    }
    if !hits.is_empty() {
        bail!("module membrane crossed: {}", hits.join("; "));
    }
    Ok("no crate, module or path named grimm anywhere under src/; the seam is process-to-process".into())
}

fn check_no_embeddings(ws: &Workspace) -> Result<String> {
    let mut hits = Vec::new();
    collect_model_files(ws.root(), &mut hits)?;
    if !hits.is_empty() {
        bail!("numeric model artifact(s) present: {}", hits.join(", "));
    }
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) {
            continue;
        }
        let lower = text.to_ascii_lowercase();
        for needle in ["cosine_similarity", "cosine(", "dot_product", "embedding", "onnxruntime"] {
            if lower.contains(needle) {
                bail!("{relative} contains `{needle}`: no float-vector similarity API");
            }
        }
    }
    for name in ws.crate_names()? {
        for key in dependency_keys(&ws.crate_manifest(&name)?) {
            if let Some(needle) = forbidden_dependency(&key) {
                bail!("crates/{name} depends on `{key}` (contains `{needle}`)");
            }
        }
    }
    Ok("no embedding API, model file or pretrained artifact on disk or in the dependency graph".into())
}

fn collect_model_files(dir: &Path, hits: &mut Vec<String>) -> Result<()> {
    const MODEL_EXTENSIONS: [&str; 9] =
        ["onnx", "pt", "pth", "safetensors", "gguf", "npy", "npz", "tflite", "mlmodel"];
    for entry in fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if matches!(name.as_str(), "target" | ".git" | "runs" | "gates") {
            continue;
        }
        if entry.file_type()?.is_dir() {
            collect_model_files(&path, hits)?;
        } else if let Some(ext) = path.extension() {
            if MODEL_EXTENSIONS.contains(&ext.to_string_lossy().as_ref()) {
                hits.push(path.display().to_string());
            }
        }
    }
    Ok(())
}

fn check_no_json_readback(ws: &Workspace) -> Result<String> {
    let mut hits = Vec::new();
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) {
            continue;
        }
        if text.contains("\"gates/") || text.contains("gates\\") {
            hits.push(format!("{relative}: opens gates/"));
        }
        for line in text.lines() {
            if (line.contains("include_str!") || line.contains("include_bytes!"))
                && line.contains(".json")
            {
                hits.push(format!("{relative}: compiles JSON into the build"));
            }
            if line.contains("serde_json::from_reader") {
                hits.push(format!("{relative}: parses JSON from a reader"));
            }
        }
    }
    if !hits.is_empty() {
        bail!("JSON re-enters a build path: {}", hits.join("; "));
    }
    Ok("no run path opens gates/** or reads JSON back as a build input (R-33)".into())
}

fn check_record_layout(_ws: &Workspace) -> Result<String> {
    use si_core::hash::{CoreBlock, CORE_BLOCK_BYTES};
    use si_core::{aligned, EDGE_RECORD_BYTES, GENOME_REC_BYTES, RECORD_FIXED_BYTES, STAMP_BYTES};

    if !si_core::layout_is_valid() {
        bail!("si-core's layout constants are inconsistent");
    }
    let lengths = [
        RECORD_FIXED_BYTES,
        EDGE_RECORD_BYTES,
        STAMP_BYTES,
        CORE_BLOCK_BYTES,
        GENOME_REC_BYTES[0],
        GENOME_REC_BYTES[1],
        GENOME_REC_BYTES[2],
    ];
    for length in lengths {
        if !aligned(length) {
            bail!("record length {length} is not a multiple of four");
        }
    }
    if RECORD_FIXED_BYTES != 80 + 32 {
        bail!("the envelope+body pair is not 80 + 32");
    }
    if CORE_BLOCK_BYTES != 36 {
        bail!("the core block is not 36 bytes (01 §13)");
    }

    let block = CoreBlock::at_birth(0x51);
    let bytes = block.encode();
    if bytes[11] != 0 || bytes[33] != 0 || bytes[34] != 0 {
        bail!("the core block encoder wrote a non-zero reserved byte");
    }
    if bytes[35] != 0x01 {
        bail!("the core block version byte is not 0x01");
    }
    let decoded = CoreBlock::decode(&bytes)?;
    if decoded != block {
        bail!("the core block does not round-trip");
    }
    for offset in [11usize, 33, 34] {
        let mut mutated = bytes;
        mutated[offset] = 0x7f;
        if CoreBlock::decode(&mutated).is_ok() {
            bail!("reserved byte at offset {offset} was accepted; reserved is checked, not trusted");
        }
    }
    Ok("112/16/16/32/16/16/36 bytes, all % 4 == 0; the core block's pads and version are asserted on read".into())
}

fn check_registry_size(ws: &Workspace) -> Result<String> {
    use si_core::consts::{self, Class};
    use si_core::shelves::{self, Layer, Mutability};

    if shelves::SHELVES.len() != 11 {
        bail!("the shelf registry has {} entries, want 11", shelves::SHELVES.len());
    }
    if !shelves::registry_is_valid() {
        bail!("shelf ids are not 1..=11 with exactly {} reserved entries", shelves::RESERVED_SHELF_COUNT);
    }
    let live = shelves::SHELVES.iter().filter(|s| s.layer != Layer::Reserved).count();
    let reserved = shelves::SHELVES.iter().filter(|s| s.layer == Layer::Reserved).count();
    // 01 §2's stage table: shelf 9 opens with B17, so the registry reads
    // "eleven entries, two reserved-empty" from here on (05 §20's moved-checks
    // list). Before B17 the same two numbers were 8 and 3.
    if live != 9 || reserved != 2 {
        bail!("shelf layers are {live} live / {reserved} reserved, want 9 / 2");
    }
    if shelves::SHELVES.iter().any(|s| s.mutability == Mutability::Reserved && s.layer != Layer::Reserved) {
        bail!("a live shelf is marked reserved");
    }
    let mut names: Vec<&str> = shelves::SHELVES.iter().map(|s| s.shelf_name).collect();
    names.sort_unstable();
    names.dedup();
    if names.len() != 11 {
        bail!("shelf names are not unique");
    }

    if !consts::placeholders_match_registry() {
        bail!("the registry's swept-row sets and the placeholder table disagree (OD-86)");
    }
    let mut names: Vec<&str> = consts::CONST_ROWS.iter().map(|r| r.name).collect();
    let total = names.len();
    names.sort_unstable();
    names.dedup();
    if names.len() != total {
        bail!("the constants registry repeats a row name");
    }
    for row in consts::CONST_ROWS {
        if row.grid < 1 {
            bail!("{} has grid {}", row.name, row.grid);
        }
        if row.lo > row.hi {
            bail!("{} has an inverted range", row.name);
        }
        if let Some(value) = row.law_value {
            if row.scalar && !(row.lo..=row.hi).contains(&value) {
                bail!("{}'s LAW value {} is outside its own range", row.name, value);
            }
        }
    }
    if consts::temperament_count() != si_core::temperament::DERIV_ORDER_LEN {
        bail!("TEMPERAMENT rows and |DERIV_ORDER| disagree");
    }
    if consts::placeholders().iter().any(|p| p.tag != consts::PLACEHOLDER_TAG) {
        bail!("a placeholder lost its UNMEASURED_PLACEHOLDER tag");
    }
    // 11 §4 DoD 2: a published value is the artifact's, not a hand edit. Every
    // entry must name a swept scalar SPECIES row, sit inside the row's box on its
    // grid, and carry the sha256 of the artifact the row's own tag names.
    let mut published: Vec<&str> = Vec::new();
    for entry in consts::SWEPT {
        let row = consts::row(entry.name)
            .ok_or_else(|| anyhow::anyhow!("swept value names no registry row: {}", entry.name))?;
        if row.class != Class::Species || !row.scalar || row.sweep.is_none() {
            bail!("{} is {:?}/scalar={}/sweep={:?} but carries a swept value",
                row.name, row.class, row.scalar, row.sweep);
        }
        if entry.value < row.lo || entry.value > row.hi || (entry.value - row.lo) % row.grid != 0 {
            bail!("{} = {} is outside [{}, {}] on grid {}", row.name, entry.value, row.lo, row.hi, row.grid);
        }
        let artifact = row.sweep.expect("checked above");
        let expected_path = format!("gates/{artifact}");
        if entry.artifact != expected_path {
            bail!("{} cites `{}`, but its registry tag is `{expected_path}`",
                row.name, entry.artifact);
        }
        if entry.version_tag.is_empty() {
            bail!("{} carries no version tag", row.name);
        }
        let path = ws.path(&expected_path);
        if !path.is_file() {
            bail!("{} cites {expected_path}, which is not on disk; a value without its artifact is a hand edit",
                row.name);
        }
        let digest = si_core::sha256::sha256_hex(&fs::read(&path)?);
        if digest != entry.sha256 {
            bail!("{} cites {expected_path} at sha256 {}, but the file hashes to {digest}",
                row.name, entry.sha256);
        }
        if published.contains(&row.name) {
            bail!("{} carries two swept values", row.name);
        }
        published.push(row.name);
    }
    if !si_core::patterns::table_is_closed() {
        bail!("the pattern table is not the closed ten-entry table");
    }
    if si_core::CONF_MAX != 99
        || si_core::SIGNAL_KIND_COUNT != 6
        || si_core::LABEL_COUNT != 6
        || si_core::DOOR_COUNT != 6
        || si_core::ORGAN_COUNT != 6
    {
        bail!("a closed count is off (conf 99, signals/labels/doors/organs 6)");
    }
    Ok(format!(
        "11 shelves (9 live, 2 reserved); {total} registry rows; {} placeholders + {} published = {} swept species rows; 18 TEMPERAMENT draws; ten patterns; closed counts 6/6/6/6",
        consts::placeholder_count(),
        consts::published_swept_count(),
        consts::swept_species_count()
    ))
}

fn check_string_constants(ws: &Workspace) -> Result<String> {
    const FUNCTION_WORDS: [&str; 15] = [
        "the", "a", "an", "is", "are", "was", "were", "what", "who", "when", "where", "of", "means",
        "yes", "no",
    ];
    const CORRECTION_MARKERS: [&str; 6] =
        ["no wait", "wait", "actually", "instead", "rather", "no,"];
    const FRAME_NAMES: [&str; 9] = [
        "P_ATTRIB", "P_DEF", "P_QUEST_A", "P_QUEST_D", "P_QUEST_W", "P_OF_REL", "P_CORRECT",
        "P_CMD_C", "P_TEACH",
    ];
    const SOURCE_LITERALS: [&str; 2] = ["owner", "grimm"];
    const TEACH_LITERALS: [&str; 3] = ["/teach", "=>", "owner-teach"];
    const SHELF_NAMES: [&str; 11] = [
        "dictionary",
        "relations",
        "rules",
        "s_r_pairs",
        "facts",
        "links",
        "questions",
        "context",
        "skills",
        "strategies",
        "events",
    ];

    if si_core::strings::FUNCTION_WORDS != FUNCTION_WORDS {
        bail!("FUNCTION_WORDS differ from docs/03 §3.2");
    }
    if si_core::strings::CORRECTION_MARKERS != CORRECTION_MARKERS {
        bail!("CORRECTION_MARKERS differ from docs/03 §3.2");
    }
    if si_core::strings::PATTERN_FRAME_NAMES != FRAME_NAMES {
        bail!("the frame names differ from docs/03 §3.2");
    }
    if si_core::SOURCE_LITERALS != SOURCE_LITERALS {
        bail!("the source enum literals are not owner/grimm");
    }
    if si_core::TEACH_LITERALS != TEACH_LITERALS {
        bail!("the TEACH literals are not /teach, =>, owner-teach");
    }
    let shelves: Vec<&str> = si_core::shelves::SHELVES.iter().map(|s| s.shelf_name).collect();
    if shelves != SHELF_NAMES {
        bail!("the shelf-registry defaults differ from 01 §2");
    }

    // The vocabulary declarations live in one module. A second copy elsewhere is
    // a second vocabulary, which is the drift this check exists to catch.
    let mut declarations = 0usize;
    let mut stray = Vec::new();
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) {
            continue;
        }
        if text.contains("pub const FUNCTION_WORDS") || text.contains("pub const CORRECTION_MARKERS")
        {
            declarations += 1;
        }
        if relative.starts_with("crates/si-audit/") {
            continue;
        }
        for literal in string_literals(&text) {
            let exact_word = FUNCTION_WORDS.contains(&literal.as_str())
                || CORRECTION_MARKERS.contains(&literal.as_str());
            if exact_word && !relative.ends_with("si-core/src/strings.rs") {
                stray.push(format!("{relative}: \"{literal}\""));
            }
        }
    }
    if declarations != 1 {
        bail!("the vocabulary tables are declared {declarations} times; want exactly one home");
    }
    if !stray.is_empty() {
        bail!("vocabulary literals outside si-core::strings: {}", stray.join("; "));
    }
    Ok("allowlist is byte-exact: 15 function words, 6 correction markers, 9 frames, 2 source literals, 3 TEACH literals, 11 shelf names; no stray word literal".into())
}

fn check_one_clock(ws: &Workspace) -> Result<String> {
    let sources = ws.source_text()?;
    let mut definitions = Vec::new();
    for (relative, text) in &sources {
        if Workspace::is_auditor(relative) {
            continue;
        }
        for line in text.lines() {
            if line.contains("fn recency(") {
                definitions.push(relative.clone());
            }
        }
    }
    if definitions.len() != 1 || !definitions[0].ends_with("si-core/src/clock.rs") {
        bail!("recency is defined at {definitions:?}; the one curve lives in si-core::clock");
    }
    let mut decay_consts = Vec::new();
    for (relative, text) in &sources {
        if !relative.starts_with("crates/si-affect/") {
            continue;
        }
        for line in text.lines() {
            let line = line.trim();
            if !line.starts_with("const ") && !line.starts_with("pub const ") {
                continue;
            }
            let name = line
                .trim_start_matches("pub ")
                .trim_start_matches("const ")
                .split(':')
                .next()
                .unwrap_or("")
                .trim();
            let decay_like = ["DECAY", "LAMBDA", "HALFLIFE", "RECENCY", "DIM"].iter().any(|n| name.contains(n));
            if decay_like && name != "LAMBDA_A" {
                decay_consts.push(format!("{relative}: {name}"));
            }
        }
    }
    if !decay_consts.is_empty() {
        bail!("si-affect defines a second decay: {}", decay_consts.join("; "));
    }
    // G13(k.1): the sleep is a *caller* of the one curve, not an owner of its
    // own. A night that defined a private decay would read as a faster clock
    // while sharing none of the arithmetic the day's median weights use (R-29).
    let mut sleep_callers = 0usize;
    let mut sleep_decays = Vec::new();
    for (relative, text) in &sources {
        if !relative.starts_with("crates/si-sleep/src/") {
            continue;
        }
        for line in text.lines() {
            let code = line.split("//").next().unwrap_or("");
            if code.contains("clock::recency(") || code.contains("Clock::recency(") {
                sleep_callers += 1;
                continue;
            }
            let trimmed = code.trim();
            if !trimmed.starts_with("const ") && !trimmed.starts_with("pub const ") {
                continue;
            }
            let name = trimmed
                .trim_start_matches("pub ")
                .trim_start_matches("const ")
                .split([':', '='])
                .next()
                .unwrap_or("")
                .trim();
            if ["DECAY", "LAMBDA", "HALFLIFE", "RECENCY", "DIM", "FLOOR"]
                .iter()
                .any(|needle| name.contains(needle))
            {
                sleep_decays.push(format!("{relative}: {name}"));
            }
        }
    }
    if sleep_callers == 0 {
        bail!("si-sleep never calls clock::recency, so its dimming is a curve of its own");
    }
    if !sleep_decays.is_empty() {
        bail!("si-sleep defines its own decay constants: {}", sleep_decays.join("; "));
    }
    // G16 / 12 §6.3: silt's decay is a *share* of the one curve (LAMBDA_SILT is
    // the same treatment DIM_BASE gets), so si-silt must call `clock::recency`
    // and may not define a constant that names a decay of its own. The check that
    // would have caught a second τ is this one, widened — not a new check.
    let mut silt_callers = 0usize;
    let mut silt_decays = Vec::new();
    for (relative, text) in &sources {
        if !relative.starts_with("crates/si-silt/src/") {
            continue;
        }
        for line in text.lines() {
            let code = line.split("//").next().unwrap_or("");
            if code.contains("clock::recency(") || code.contains("Clock::recency(") {
                silt_callers += 1;
                continue;
            }
            let trimmed = code.trim();
            if !trimmed.starts_with("const ") && !trimmed.starts_with("pub const ") {
                continue;
            }
            let name = trimmed
                .trim_start_matches("pub ")
                .trim_start_matches("const ")
                .split([':', '='])
                .next()
                .unwrap_or("")
                .trim();
            if ["DECAY", "LAMBDA", "HALFLIFE", "RECENCY", "DIM", "FLOOR"]
                .iter()
                .any(|needle| name.contains(needle))
            {
                silt_decays.push(format!("{relative}: {name}"));
            }
        }
    }
    if silt_callers == 0 {
        bail!("si-silt never calls clock::recency, so its decay is a curve of its own (04 §30 AB.1)");
    }
    if !silt_decays.is_empty() {
        bail!("si-silt defines its own decay constants: {}", silt_decays.join("; "));
    }
    Ok(format!(
        "one recency function, in si-core::clock; si-affect owns LAMBDA_A only and no other decay; si-sleep calls the curve {sleep_callers} times and owns no constant of its own; si-silt calls it {silt_callers} time(s) and owns none either"
    ))
}

/// The 1-based line span of `fn NAME(` in a source text, closed by the brace
/// balance. A write-partition claim is per-pass (10 §7 fixes the pass set), so a
/// scan that cannot say *which* function a call sits in proves nothing.
///
/// The opening brace is not on the `fn` line — the parameter list of these
/// signatures runs over its own lines — so the balance only counts once it has
/// seen a positive depth; before that a `where` clause or an empty parameter line
/// would close the span on the function's own header.
pub(crate) fn fn_span(text: &str, name: &str) -> Option<(usize, usize)> {
    let needle = format!("fn {name}(");
    let start = text.lines().position(|line| line.contains(&needle))? + 1;
    let mut depth = 0i32;
    let mut opened = false;
    for (offset, line) in text.lines().enumerate().skip(start - 1) {
        for ch in line.chars() {
            match ch {
                '{' => {
                    depth += 1;
                    opened = true;
                }
                '}' => depth -= 1,
                _ => {}
            }
        }
        if opened && depth <= 0 {
            return Some((start, offset + 1));
        }
    }
    None
}

/// The 1-based line on which the block opened at `start` closes, by brace balance.
/// A declaration's body is the unit these field-partition scans need: a token in
/// the file is not a token in the type.
pub(crate) fn block_end(text: &str, start: usize) -> usize {
    let mut depth = 0i32;
    let mut opened = false;
    for (offset, line) in text.lines().enumerate().skip(start.saturating_sub(1)) {
        for ch in line.chars() {
            match ch {
                '{' => {
                    depth += 1;
                    opened = true;
                }
                '}' => depth -= 1,
                _ => {}
            }
        }
        if opened && depth <= 0 {
            return offset + 1;
        }
    }
    text.lines().count()
}

/// The kind and name of a declaration line, so a scan can say whether the line it
/// is looking at sits in a **struct body** (a field, which persists), an enum body
/// (a variant payload, which is a value the code returns) or a function (a
/// parameter or a local).
pub(crate) fn decl_of(line: &str) -> Option<(&'static str, String)> {
    let rest = line.strip_prefix("pub ").unwrap_or(line);
    for (kind, word) in [
        ("struct", "struct "),
        ("enum", "enum "),
        ("fn", "fn "),
        ("other", "impl "),
        ("other", "trait "),
        ("other", "use "),
        ("other", "mod "),
        ("other", "const "),
        ("other", "static "),
        ("other", "type "),
    ] {
        if let Some(name) = rest.strip_prefix(word) {
            let name = name
                .split(['<', '{', ' ', '(', ':', '='])
                .next()
                .unwrap_or_default()
                .to_string();
            return Some((kind, name));
        }
    }
    None
}

/// A struct field line: `name: Type` with no call syntax and no match arm. AE.1's
/// licence is "everywhere except as a return value", so a function's parameter and
/// an enum's variant payload pass and a persisted field does not.
fn is_field_line(line: &str) -> bool {
    let rest = line.strip_prefix("pub ").unwrap_or(line);
    if rest.contains('(') || rest.contains("=>") || rest.starts_with('&') {
        return false;
    }
    let mut halves = rest.splitn(2, ':');
    let name = halves.next().unwrap_or("");
    let ty = match halves.next() {
        Some(ty) => ty.trim(),
        None => return false,
    };
    !name.is_empty()
        && !ty.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// The full text of the type whose declaration line contains `header`, up to and
/// including the brace that closes it. A field-partition claim ("`World` carries
/// the identity and no class") is a claim about one declaration's body, so it needs
/// the body's boundaries rather than a window of lines around a guess.
pub(crate) fn type_block(text: &str, header: &str) -> Option<String> {
    let start = text.lines().position(|line| line.contains(header))? + 1;
    let end = block_end(text, start);
    Some(
        text.lines()
            .skip(start - 1)
            .take(end - start + 1)
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// 05 §20's C8 leg for G13(i)/(j): the sleep has no route to a row. The strong
/// form is structural — `si-sleep` does not depend on `si-learn`, so `commit` is
/// not a symbol it could name — and the verb scan is the belt.
fn check_sleep_writes_nothing(ws: &Workspace) -> Result<String> {
    let manifest = ws.read("crates/si-sleep/Cargo.toml")?;
    for forbidden in ["si-learn", "si-audit", "si-front"] {
        if manifest.contains(forbidden) {
            bail!(
                "si-sleep's manifest declares `{forbidden}`; a sleep that can commit is SI \
                 teaching itself while nobody is watching (G13(i))"
            );
        }
    }
    const BANNED: [&str; 12] = [
        "si_learn",
        "si_audit",
        "append_row",
        "commit(",
        "supersede(",
        ".age(",
        "confirm(",
        "promote_gain",
        "adjust_gain",
        "bake_q12(",
        "interner_mut",
        "points_mut",
    ];
    let mut hits = Vec::new();
    let mut scanned = 0usize;
    for (relative, text) in ws.source_text()? {
        if !relative.starts_with("crates/si-sleep/src/") {
            continue;
        }
        scanned += 1;
        for (index, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            for needle in BANNED {
                if code.contains(needle) {
                    hits.push(format!("{relative}:{}: `{needle}`", index + 1));
                }
            }
        }
    }
    if scanned == 0 {
        bail!("si-sleep has no source module to scan");
    }
    if !hits.is_empty() {
        bail!("si-sleep reaches the write path: {}", hits.join("; "));
    }
    // The floor is a registry constant, never a column: an edge that carried its
    // own floor would let one night's `d` be authored by the row it dims.
    let record = ws.read("crates/si-core/src/record.rs")?;
    for struct_name in ["Edge", "Envelope"] {
        let needle = format!("pub struct {struct_name}");
        let start = record
            .lines()
            .position(|line| line.contains(&needle))
            .with_context(|| format!("`{struct_name}` is gone from 01 §5.1's layout"))?;
        let body: String = record
            .lines()
            .skip(start)
            .take_while(|line| !line.trim().ends_with('}'))
            .chain(std::iter::once("}"))
            .collect::<Vec<_>>()
            .join("\n");
        if body.to_lowercase().contains("floor") {
            bail!("`{struct_name}` carries a floor column; the floor is a registry constant (03 §6)");
        }
    }
    Ok(format!(
        "{scanned} si-sleep modules name no row-writing verb, the crate has no si-learn/si-audit/si-front dependency, and neither Edge nor Envelope has a floor column"
    ))
}

/// 05 §20's C8 leg for G13(j.2): one writer of `weight`, and it cannot brighten
/// or zero. Checked where the call sits, because 10 §7's pass set is the unit.
fn check_sleep_never_brightens(ws: &Workspace) -> Result<String> {
    let sleep = ws.read("crates/si-sleep/src/lib.rs")?;
    let store = ws.read("crates/si-store/src/store.rs")?;
    let mut writers: Vec<(usize, String)> = Vec::new();
    for (index, line) in sleep.lines().enumerate() {
        let code = line.split("//").next().unwrap_or("");
        if code.contains("dim_edge_weight(") {
            writers.push((index + 1, "dim_edge_weight".to_string()));
        }
        // Any other route to an edge byte — a decoded `Edge` reassigned and
        // re-encoded, or an arena write — is a second weight writer.
        for needle in ["edges.write(", "edges_arena", ".encode(", "Arena::write"] {
            if code.contains(needle) {
                writers.push((index + 1, format!("second write route `{needle}`")));
            }
        }
    }
    if writers.len() != 1 {
        bail!("si-sleep's weight writers: {writers:?}; exactly one `dim_edge_weight` call is allowed");
    }
    let (line, _) = writers[0];
    let p3 = fn_span(&sleep, "p3_rescale")
        .context("p3_rescale is gone, so the rescale pass has no body to audit")?;
    if line < p3.0 || line > p3.1 {
        bail!("the weight write is at si-sleep line {line}, outside p3_rescale ({}–{})", p3.0, p3.1);
    }
    for other in ["p1_replay", "p2_integration_scan", "p4_calm_reprocessing"] {
        let span = fn_span(&sleep, other)
            .with_context(|| format!("{other} is gone, so the pass set is not 10 §7's"))?;
        let body: Vec<&str> = sleep.lines().skip(span.0 - 1).take(span.1 - span.0 + 1).collect();
        for needle in ["dim_edge_weight", "touch_edge_curve"] {
            if body.iter().any(|l| l.split("//").next().unwrap_or("").contains(needle)) {
                bail!("{other} calls `{needle}`; only P3 writes `weight` while quiet (law 25)");
            }
        }
    }
    // P3's own arithmetic is monotone by two panics, and the store refuses the
    // same two outcomes as an error. A clamp instead of a refusal would make the
    // brightening survivable, which is the failure this leg is for.
    let p3_body: String = sleep
        .lines()
        .skip(p3.0 - 1)
        .take(p3.1 - p3.0 + 1)
        .collect::<Vec<_>>()
        .join("\n");
    for guard in ["would brighten", "would zero"] {
        if !p3_body.contains(guard) {
            bail!("p3_rescale has no `{guard}` refusal");
        }
    }
    let dim = fn_span(&store, "dim_edge_weight")
        .context("the store has no dim_edge_weight")?;
    let dim_body: String = store
        .lines()
        .skip(dim.0 - 1)
        .take(dim.1 - dim.0 + 1)
        .collect::<Vec<_>>()
        .join("\n");
    for guard in ["would brighten", "would zero", "outside 6..8"] {
        if !dim_body.contains(guard) {
            bail!("the store's weight write has no `{guard}` refusal");
        }
    }
    Ok(format!(
        "one weight writer: si-sleep line {line} inside p3_rescale ({}–{}); P1/P2/P4 name neither the write nor the RAM pair, and both P3 and the store refuse a brightening, a zeroing and a byte outside 6..8",
        p3.0, p3.1
    ))
}

/// 05 §20's C8 leg for G13(k.2): the calm view is a readout. `walks` is excluded
/// on purpose — 04 §28 makes it the sleep's tick cost, so the session *must*
/// consume it; the four counts below are the ones with no legal consumer.
fn check_calm_view_is_terminal(ws: &Workspace) -> Result<String> {
    const READOUT: [&str; 6] = [
        "CalmView",
        "edges_relit",
        "rows_relit",
        "rows_read_calm_not_settleable",
        "points_swept",
        "stamps_written",
    ];
    // The organs that could act on a count: a relight that fed `ignite`, a
    // sweep that fed a floor or `d`, a labeler that fed a name. The harness is
    // deliberately absent — grading a budget from a reported count is what the
    // harness is for; an organ changing its behavior because of one is not.
    const CONSUMER_CRATES: [&str; 9] = [
        "si-core", "si-store", "si-affect", "si-walk", "si-learn", "si-read", "si-front",
        "si-audit", "si-session",
    ];
    let mut leaks = Vec::new();
    let mut seen = 0usize;
    for (relative, text) in ws.source_text()? {
        let crate_name = match relative
            .strip_prefix("crates/")
            .and_then(|rest| rest.split('/').next())
        {
            Some(name) => name.to_string(),
            None => continue,
        };
        if !CONSUMER_CRATES.contains(&crate_name.as_str()) {
            continue;
        }
        for (index, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            for needle in READOUT {
                if code.contains(needle) {
                    leaks.push(format!("{relative}:{}: `{needle}`", index + 1));
                }
            }
        }
    }
    for (relative, text) in ws.source_text()? {
        if relative.starts_with("crates/si-sleep/src/") {
            seen += text
                .lines()
                .filter(|line| READOUT.iter().any(|needle| line.contains(needle)))
                .count();
        }
    }
    if !leaks.is_empty() {
        bail!(
            "a calm-view count reaches an organ: {}",
            leaks.join("; ")
        );
    }
    if seen == 0 {
        bail!("the calm view has no report line at all, so this check proved nothing");
    }
    // Inside the sleep: the rescale must not read the calm pass's counts, or the
    // readout becomes a second curve.
    let sleep = ws.read("crates/si-sleep/src/lib.rs")?;
    if let Some((start, end)) = fn_span(&sleep, "p3_rescale") {
        for line in sleep.lines().skip(start - 1).take(end - start + 1) {
            let code = line.split("//").next().unwrap_or("");
            for needle in READOUT {
                if code.contains(needle) {
                    bail!("p3_rescale reads the calm-view count `{needle}`: the readout is feeding `d`");
                }
            }
        }
    }
    Ok(format!(
        "{seen} calm-view lines live only in si-sleep's report and the harness; no organ names a readout field, and P3 reads none of them"
    ))
}

fn check_teach_literals(_ws: &Workspace) -> Result<String> {
    use si_core::patterns::{PatternId, PATTERN_TABLE, PATTERN_TABLE_SIZE};
    if PATTERN_TABLE.len() != PATTERN_TABLE_SIZE || PATTERN_TABLE_SIZE != 10 {
        bail!("the pattern table is not ten entries");
    }
    let frames: Vec<PatternId> = PATTERN_TABLE
        .iter()
        .copied()
        .filter(|p| *p != PatternId::Undecoded)
        .collect();
    let names: Vec<&str> = frames.iter().map(|p| p.name()).collect();
    if names != si_core::strings::PATTERN_FRAME_NAMES {
        bail!("the table's named frames differ from docs/03 §3.2's order");
    }
    if !PATTERN_TABLE.contains(&PatternId::PTeach) {
        bail!("P_TEACH is missing from the table");
    }
    if PATTERN_TABLE.last() != Some(&PatternId::Undecoded) {
        bail!("Undecoded is not the tenth, trailing entry");
    }
    if si_core::TEACH_LITERALS != ["/teach", "=>", "owner-teach"] {
        bail!("the TEACH literals are not the three metadata literals");
    }
    Ok("ten patterns: the nine §3.2 frames in order plus Undecoded, with P_TEACH present and exactly three TEACH literals".into())
}

fn check_temperament_closed(ws: &Workspace) -> Result<String> {
    use si_core::consts::{Class, CONST_ROWS};
    use si_core::hash::CoreBlock;
    use si_core::temperament::{DerivId, Temperament, DERIV_ORDER, DERIV_ORDER_LEN};

    if !si_core::seed::order_is_closed() {
        bail!("DERIV_ORDER does not match the TEMPERAMENT rows");
    }
    if std::mem::size_of::<Temperament>() != DERIV_ORDER_LEN * 2 {
        bail!("the temperament vector is not 18 u16 values");
    }
    let drawn: Vec<&str> = DERIV_ORDER.iter().map(|id| id.registry_name()).collect();
    for row in CONST_ROWS {
        if matches!(row.class, Class::Law | Class::Ruling) && drawn.contains(&row.name) {
            bail!("{} is a LAW/ruling row and must not be drawn", row.name);
        }
        if row.class == Class::Temperament && !drawn.contains(&row.name) {
            bail!("{} is TEMPERAMENT but DERIV_ORDER never fills it", row.name);
        }
    }
    for id in DERIV_ORDER {
        let _ = DerivId::registry_name(id);
    }

    // The derivation reads no entropy, no file and no clock. 13 §6.3: the list gains
    // `tongue.rs`, because the voice's draw is the same seed-only law one class
    // over and the scan must not quietly stop covering it.
    for relative in [
        "crates/si-core/src/seed.rs",
        "crates/si-core/src/temperament.rs",
        "crates/si-core/src/tongue.rs",
        "crates/si-core/src/rng.rs",
    ] {
        let text = ws.read(relative)?;
        for needle in [
            "getrandom",
            "rand::",
            "SystemTime",
            "Instant::now",
            "env::var",
            "std::fs",
            "File::open",
        ] {
            if text.contains(needle) {
                bail!("{relative} reaches `{needle}`; the derivation is seed-only");
            }
        }
    }

    // The vector has no address in a schema, container, export or cache.
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) {
            continue;
        }
        if relative.contains("si-core/src/temperament.rs") || relative.contains("si-core/src/seed.rs") {
            continue;
        }
        if text.contains("temperament_vector") || text.contains("temperament_values") {
            bail!("{relative} stores the derived vector");
        }
    }

    // G14(c): the digest is the byte-comparison target, and it is seed-bound.
    for seed in [0u64, 1, 0x51, u64::MAX] {
        let block = CoreBlock::at_birth(seed);
        if !block.digest_matches_seed() {
            bail!("core block for seed {seed:#x} does not re-derive to its digest");
        }
        let bytes = block.encode();
        let decoded = CoreBlock::decode(&bytes)?;
        if decoded.temperament_digest != si_core::seed::temperament_digest(seed) {
            bail!("the core block's digest is not the derived-vector digest");
        }
    }
    Ok("18 values across the registry's TEMPERAMENT rows, no LAW row drawn, derivation seed-only, digest re-derived and byte-compared (G14(c))".into())
}

/// OD-87 (owner ruling 2026-10-23, B7 batch OD-A): the decay-deficit DRIVE is SPECIES — the mind
/// drifts toward what it is about to forget — and is mechanism, so it has no row
/// of its own; the four `IGN_WEIGHT_*` multipliers across the ignition sources are
/// TEMPERAMENT draws. "No sweep may ever touch a temperament row", so this checks
/// the law from each side a sweep can reach: the placeholder rule, the axis
/// enumeration, the value channel the enumeration writes through, and the
/// artifacts already published.
fn check_sweep_never_touches_temperament(ws: &Workspace) -> Result<String> {
    use si_core::consts::{Class, CONST_ROWS};

    let drawn: Vec<&str> = CONST_ROWS
        .iter()
        .filter(|row| row.class == Class::Temperament)
        .map(|row| row.name)
        .collect();
    if drawn.is_empty() {
        bail!("the registry has no TEMPERAMENT rows for the law to protect");
    }

    // (1) The one mechanical placeholder rule (OD-86) gives a draw no value.
    for row in CONST_ROWS.iter().filter(|row| row.class == Class::Temperament) {
        if let Some(value) = si_core::consts::placeholder_value(row) {
            bail!("{} is a TEMPERAMENT draw but the placeholder rule hands it {value}", row.name);
        }
    }

    // (2) Every axis any sweep enumerates is a SPECIES row. The prefixes are read
    // off the registry's own tags, so a gate added later is covered here too.
    let mut prefixes: Vec<&str> = Vec::new();
    for row in CONST_ROWS {
        if let Some(tag) = row.sweep {
            let prefix = tag.split('/').next().unwrap_or(tag);
            if !prefixes.contains(&prefix) {
                prefixes.push(prefix);
            }
        }
    }
    let mut axes = 0usize;
    for prefix in &prefixes {
        for axis in super::sweep::axes(prefix) {
            axes += 1;
            if axis.class != Class::Species {
                bail!(
                    "sweep {prefix} owns {} as a {:?} axis; OD-87: a draw is never an axis",
                    axis.name,
                    axis.class
                );
            }
            if drawn.contains(&axis.name) {
                bail!("sweep {prefix} enumerates the TEMPERAMENT row {}", axis.name);
            }
        }
    }

    // (3) The value channel. A sweep installs constants through
    // `launch_constants`, which refuses any name that is neither a swept SPECIES
    // row nor one of engineering/12 §3.2's RULING rows; that guard is what makes
    // the gate tags the TEMPERAMENT rows still carry inert metadata — they name
    // the artifact that publishes the row's drawn distribution, not a grid, and
    // `check_registry_size` counts them.
    let runner = ws.read("crates/si-cli/src/harness/runner.rs")?;
    // The guard below is a multi-line text, so it is matched on the *code*, not on
    // the platform's line endings: a CRLF worktree and an LF one hold the same
    // refusal, and an auditor that only passes on one of them audits its own
    // checkout. (`code_text` is the same law for the single-line scans.)
    let runner = runner.replace("\r\n", "\n");
    let start = runner
        .find("fn launch_constants")
        .context("runner.rs has no `launch_constants` for the channel to guard")?;
    let body = &runner[start..];
    let body = match body.find("\nfn ") {
        Some(end) => &body[..end],
        None => body,
    };
    if !body.contains(
        "!matches!(\n            row.class,\n            si_core::consts::Class::Species | si_core::consts::Class::Ruling\n        )",
    ) {
        bail!(
            "`launch_constants`' class guard is not the Species-or-Ruling refusal, so a sweep could \
             override a draw"
        );
    }

    // (4) What has already been published lists no draw among its axes. A gate
    // may own more than one artifact (05 §4 gives G13 two), and each one's axes
    // are read from its own file, not from the gate's union.
    let mut artifacts = 0usize;
    for prefix in &prefixes {
        for path in super::sweep::artifact_paths(prefix) {
            if !ws.exists(path) {
                continue;
            }
            let document: serde_json::Value = serde_json::from_str(&ws.read(path)?)
                .with_context(|| format!("parse {path}"))?;
            for axis in document["axes"].as_array().into_iter().flatten() {
                let name = axis["name"].as_str().unwrap_or_default();
                if drawn.contains(&name) {
                    bail!("{path} publishes the TEMPERAMENT row {name} as a swept axis");
                }
                if axis["class"].as_str() != Some("Species") {
                    bail!("{path} publishes {name} as a {} axis", axis["class"]);
                }
            }
            artifacts += 1;
        }
    }

    Ok(format!(
        "{axes} swept axes across {} enumeration prefix(es), every one SPECIES; the {} TEMPERAMENT draws \
         (the four IGN_WEIGHT_* among them) keep their gate tag as inert metadata only: the placeholder rule \
         gives them no value, `launch_constants` refuses them by class, and {artifacts} published artifact(s) \
         name none of them",
        prefixes.len(),
        drawn.len()
    ))
}

// ------------------------------------------------------------ B1 stage checks

/// 00 §6 / 05 §20 `check_config_guard`: the loader cannot run in CONFIG SI, and
/// the shipped build has no loader body to run anyway.
fn check_config_guard(ws: &Workspace) -> Result<String> {
    // (1) The build guard: `engine` is off in every default build.
    let store_manifest = ws.crate_manifest("si-store")?;
    if !store_manifest.contains("engine = []") {
        bail!("si-store has no `engine` feature; the loader body has no build guard");
    }
    if !store_manifest.lines().any(|line| line.trim() == "default = []") {
        bail!("si-store's default features are not empty");
    }
    for name in ws.crate_names()? {
        let manifest = ws.crate_manifest(&name)?;
        for line in manifest.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("default =") {
                if rest.contains("engine") {
                    bail!(
                        "crates/{name} enables `engine` by default: the shipped binary would carry the loader body"
                    );
                }
            }
        }
    }

    // (2) The runtime guard: the entry refuses before its engine branch, and the
    // file holds no I/O at all.
    let source = ws.read("crates/si-store/src/layers_frozen.rs")?;
    for needle in ["fs::", "File::", "OpenOptions", "mmap", "read_to_string", "read_dir"] {
        if source.contains(needle) {
            bail!("layers_frozen.rs reaches `{needle}`; the loader entry must not touch a handle");
        }
    }
    let guard = source
        .find("if config == Config::Si")
        .ok_or_else(|| anyhow::anyhow!("the loader entry has no CONFIG SI guard"))?;
    let after_guard = &source[guard..];
    let refusal = after_guard
        .find("ConfigRefusal::SiHasNoFrozenLayer")
        .ok_or_else(|| anyhow::anyhow!("the CONFIG SI guard returns no refusal"))?;
    let engine = after_guard
        .find("load_engine(")
        .ok_or_else(|| anyhow::anyhow!("the loader entry has no engine branch"))?;
    if refusal > engine {
        bail!("the CONFIG SI refusal is not the first branch of the loader entry");
    }
    if !source.contains("#[cfg(not(feature = \"engine\"))]") {
        bail!("no engine-off stub is compiled: the guard has no build half");
    }

    // (3) One file may name a frozen artifact; the loop crates name none.
    //
    // The scan reads code text, not prose: B11's viewer explains in its module doc
    // that `SIgen` is *not* its file to map (it takes bytes, **OD-148**), and a law
    // that forbids an open handle cannot also forbid saying so. A test that quotes the
    // magic to assert the header is the same category — the G5(c) subject is a handle
    // a build path can reach.
    let mut holders = Vec::new();
    let mut genome_mentions = Vec::new();
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) || relative.contains("/tests/") {
            continue;
        }
        let code = code_text(text);
        if code.contains("FrozenManifest") {
            holders.push(relative.clone());
        }
        if code.contains("SIgen") {
            genome_mentions.push(relative);
        }
    }
    holders.sort();
    if holders != ["crates/si-store/src/layers_frozen.rs"] {
        bail!(
            "`FrozenManifest` is constructed or named in {}; only the loader entry may hold a frozen path",
            holders.join(", ")
        );
    }
    genome_mentions.sort();
    if genome_mentions != ["crates/si-core/src/container.rs"] {
        bail!("the `SIgen` magic is named in {}", genome_mentions.join(", "));
    }
    Ok("engine off by default everywhere, the loader entry performs no I/O and refuses CONFIG SI before its engine branch, and only `layers_frozen` can name a frozen artifact (G5c)".into())
}

/// 05 §20: the inverted scan. `bake` is deliberately stored, so a no-mood sweep
/// that also banned `bake` would pass while deleting the plasticity state.
fn check_bake_is_the_exception(ws: &Workspace) -> Result<String> {
    use si_core::record::{Body, Edge, Envelope, EDGE_BYTES};
    use si_store::store::{EdgeDraft, Lattice};

    let schema = ws.read("crates/si-core/src/record.rs")?;
    if !schema.contains("pub bake: u16") {
        bail!("the edge schema has no stored `bake`: the no-mood scan would pass by banning both");
    }

    let edge = Edge { to_point: 1, kind: 0, flags: 0, weight: 4096, bake: 7, charge_off: 0 };
    let bytes = edge.encode();
    if bytes.len() != EDGE_BYTES || u16::from_le_bytes([bytes[8], bytes[9]]) != 7 {
        bail!("bake is not stored at edge offset 8 of {EDGE_BYTES} bytes");
    }
    if bytes[10] != 0 || bytes[11] != 0 {
        bail!("the edge pad after bake is not zero");
    }
    if Edge::decode(&bytes)?.bake != 7 {
        bail!("bake does not survive the edge round-trip");
    }
    let frozen = Edge { kind: 7, bake: 1, ..edge };
    if !frozen.is_owned() && Edge::decode(&frozen.encode()).is_ok() {
        bail!("a frozen-genome edge carrying a bake was accepted (07 §9)");
    }

    // The one write path stores the draft's bake into the edge arena unchanged.
    let mut lattice = Lattice::new();
    let _row_id = lattice.append_row(
        Envelope::live(4, 0, 1, 0),
        Body::default(),
        vec![EdgeDraft::owned(1, 4096, 9)],
        &[],
        &[],
    )?;
    let stored = lattice.edges_arena().read(0)?;
    if u16::from_le_bytes([stored[8], stored[9]]) != 9 {
        bail!("append_row did not store the draft's bake at edge offset 8");
    }
    Ok("`bake` is a stored u16 at edge offset 8, carried through append_row, and refused on frozen-genome edges; it is the exception the no-mood scan must not catch (G9)".into())
}

/// 05 §20 / 02 §12 (iii): no stored mood. Field shapes and JSON keys only — a
/// comment about the read-side median is not a field.
// ------------------------------------------------------------ B2 stage checks

/// The signature text of the first function whose line starts with `prefix`.
fn function_signature(source: &str, prefix: &str) -> Option<String> {
    let start = source.find(prefix)?;
    let rest = &source[start..];
    let end = rest.find('{')?;
    Some(rest[..end].trim().to_string())
}

fn function_body<'a>(source: &'a str, prefix: &str) -> Option<&'a str> {
    let start = source.find(prefix)?;
    let open = start + source[start..].find('{')?;
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    for index in open..bytes.len() {
        match bytes[index] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&source[open + 1..index]);
                }
            }
            _ => {}
        }
    }
    None
}

fn argument_list(signature: &str) -> Vec<String> {
    let open = match signature.find('(') {
        Some(index) => index,
        None => return Vec::new(),
    };
    let close = match signature[open..].find(')') {
        Some(index) => open + index,
        None => return Vec::new(),
    };
    signature[open + 1..close]
        .split(',')
        .map(str::trim)
        .filter(|arg| !arg.is_empty())
        .map(str::to_string)
        .collect()
}

/// 05 §20 `check_signal_kinds`: six kinds in 01 §5.3's order, closed to a
/// seventh, and no module outside 04 §6's birth table constructs an event.
fn check_signal_kinds(ws: &Workspace) -> Result<String> {
    use si_affect::signal::SignalKind;

    const SCHEMA: [&str; 6] = [
        "novelty-spike",
        "realized-gain",
        "contradiction",
        "settle-failure",
        "open-question-age",
        "idle-cascade-contradiction",
    ];
    let names: Vec<&str> = SignalKind::ALL.iter().map(|kind| kind.name()).collect();
    if names != SCHEMA {
        bail!("signal kinds are {names:?}, want 01 §5.3's {SCHEMA:?}");
    }
    for (index, kind) in SignalKind::ALL.iter().enumerate() {
        if kind.index() != index {
            bail!("{} has index {}, want {index}", kind.name(), kind.index());
        }
    }

    let source = ws.read("crates/si-affect/src/signal.rs")?;
    let body = source
        .split_once("pub enum SignalKind {")
        .and_then(|(_, rest)| rest.split_once('}'))
        .map(|(body, _)| body)
        .ok_or_else(|| anyhow::anyhow!("signal.rs has no `pub enum SignalKind`"))?;
    let mut variants: Vec<(String, u8)> = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        let (name, value) = line
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("enum line `{line}` carries no index"))?;
        let index: u8 = value
            .trim()
            .trim_end_matches(',')
            .parse()
            .with_context(|| format!("parse enum line `{line}`"))?;
        variants.push((name.trim().to_string(), index));
    }
    let indices: Vec<u8> = variants.iter().map(|(_, index)| *index).collect();
    let wanted: Vec<u8> = (0..6).collect();
    if indices != wanted {
        bail!("the signal enum's indices are {indices:?}, want 0..=5");
    }
    let arms = source.matches("=> SignalKind::").count();
    if arms != 6 {
        bail!("`from_u8` has {arms} kind arms, want six");
    }
    if !source.contains("other => panic!") {
        bail!("`from_u8` has no panic arm; a seventh number would be accepted (I8)");
    }

    // 04 §6's birth table: si-walk (lighting, settle, idle), si-learn (write
    // events), si-session (tick-start age), si-audit (the labeler). Tests may
    // build fixtures; no other module may construct an event.
    const BIRTH_CRATES: [&str; 4] =
        ["crates/si-walk/", "crates/si-learn/", "crates/si-session/", "crates/si-audit/"];
    let mut hits = Vec::new();
    let mut src_sites = 0usize;
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) {
            continue;
        }
        let constructed = text.contains("Signal::new(") || text.contains("Signal { kind:");
        if !constructed {
            continue;
        }
        let allowed = relative.contains("/tests/")
            || relative == "crates/si-affect/src/signal.rs"
            || BIRTH_CRATES.iter().any(|prefix| relative.starts_with(prefix));
        if allowed {
            if !relative.contains("/tests/") {
                src_sites += 1;
            }
        } else {
            hits.push(relative);
        }
    }
    if !hits.is_empty() {
        bail!("signal events constructed outside 04 §6's birth table: {}", hits.join(", "));
    }
    Ok(format!(
        "six kinds in 01 §5.3 order, indices 0..=5, six arms plus a panic for a seventh; construction lives in the birth crates only ({src_sites} src module(s) name the constructor)"
    ))
}

/// 05 §20 `check_bake_no_event_kind`: the costume test. At the formation
/// instant the bake law reads one scalar; the firing kind and its magnitude go
/// into the stamp and nowhere near the curve (G9, OD-33).
fn check_bake_no_event_kind(ws: &Workspace) -> Result<String> {
    let drive = ws.read("crates/si-affect/src/drive.rs")?;
    let signature = function_signature(&drive, "pub fn salience")
        .ok_or_else(|| anyhow::anyhow!("drive.rs defines no `pub fn salience`"))?;
    let args = argument_list(&signature);
    let want = ["a_formation: u8", "formation: &Formation", "params: &Params"];
    if args != want {
        bail!("salience takes {args:?}, want {want:?}");
    }
    let scalars: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|arg| arg.contains(": u8") || arg.contains(": u16") || arg.contains(": i"))
        .collect();
    if scalars != ["a_formation: u8"] {
        bail!("salience's scalar arguments are {scalars:?}; `a_formation` is the only one");
    }

    let body = function_body(&drive, "pub fn salience")
        .ok_or_else(|| anyhow::anyhow!("salience has no body"))?;
    let bake_line = body
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("let bake ="))
        .ok_or_else(|| anyhow::anyhow!("salience assigns no bake"))?;
    if bake_line != "let bake = bake_q12(a_formation, params.b_max, params.k_tanh);" {
        bail!(
            "the bake expression is `{bake_line}`; it may read `a_formation` and the two scale constants, nothing else"
        );
    }

    let bake = ws.read("crates/si-affect/src/bake.rs")?;
    let bake_signature = function_signature(&bake, "pub fn bake_q12")
        .ok_or_else(|| anyhow::anyhow!("bake.rs defines no `pub fn bake_q12`"))?;
    if argument_list(&bake_signature) != ["a_formation: u8", "b_max: u16", "k_tanh: u16"] {
        bail!("bake_q12's arguments are not the scalar and its two scale constants");
    }
    for (index, line) in bake.lines().enumerate() {
        let code = line.split("//").next().unwrap_or("");
        for banned in ["Signal", "SignalKind", "kind", "source", "referent", "magnitude"] {
            if contains_word(code, banned) {
                bail!("bake.rs:{} names `{banned}`; the bake may not consult one", index + 1);
            }
        }
    }

    // One assignment site: salience. A second call would let a later tick
    // re-price a stored bake (OD-33).
    let mut call_sites = Vec::new();
    for (relative, text) in ws.source_text()? {
        if !relative.starts_with("crates/si-affect/src/") || relative.ends_with("bake.rs") {
            continue;
        }
        for _ in 0..text.matches("bake_q12(").count() {
            call_sites.push(relative.clone());
        }
    }
    if call_sites != ["crates/si-affect/src/drive.rs"] {
        bail!("bake_q12 is called from {call_sites:?}; `salience` is the only assignment site (OD-33)");
    }
    Ok("salience takes one scalar (`a_formation`); the bake law names no kind, magnitude, source or referent, and bake_q12 has one call site".into())
}

/// 05 §20 `check_drive_doors`: exactly six doors on one home, `budget` level
/// free, and — once `si-walk` exists — the walk's call sites are exactly
/// `update` and `conduction` (OD-20, OD-22).
fn check_drive_doors(ws: &Workspace) -> Result<String> {
    use si_affect::drive::DOORS;

    const SPEC: [&str; 6] = ["prioritize", "emit", "budget", "salience", "update", "conduction"];
    if DOORS != SPEC {
        bail!("DOORS is {DOORS:?}, want 02 §3's {SPEC:?}");
    }
    let drive = ws.read("crates/si-affect/src/drive.rs")?;
    let mut defined: Vec<&str> = drive
        .lines()
        .filter_map(|line| line.strip_prefix("pub fn "))
        .filter_map(|rest| rest.split('(').next())
        .collect();
    let mut want = DOORS.to_vec();
    defined.sort_unstable();
    want.sort_unstable();
    if defined != want {
        bail!("drive.rs's public functions are {defined:?}, want exactly the six doors {want:?}");
    }

    let sources = ws.source_text()?;
    for (relative, text) in &sources {
        if !relative.starts_with("crates/si-affect/src/") || relative.ends_with("drive.rs") {
            continue;
        }
        // A door is a free function. `Integrator::update` is an indented method
        // behind the door, not a second door; only a column-0 `pub fn` counts.
        for door in DOORS {
            if text.lines().any(|line| line.starts_with(&format!("pub fn {door}("))) {
                bail!("{relative} defines a second `{door}`; a door has one home");
            }
        }
    }

    let signature = function_signature(&drive, "pub fn budget")
        .ok_or_else(|| anyhow::anyhow!("drive.rs defines no `pub fn budget`"))?;
    let args = argument_list(&signature);
    if args != ["params: &Params"] {
        bail!("budget takes {args:?}; queue arithmetic reads no level (OD-22)");
    }
    let body = function_body(&drive, "pub fn budget")
        .ok_or_else(|| anyhow::anyhow!("budget has no body"))?;
    // `salience_floor` is a queue threshold the door legitimately carries; the
    // level itself (a_now / a_formation), a signal and the conduction term are
    // what must not appear.
    for needle in ["a_now", "a_formation", "Signal", "conduction"] {
        if body.contains(needle) {
            bail!("budget's body reaches `{needle}`; it is queue arithmetic only (OD-22)");
        }
    }

    let mut calls: Vec<&str> = Vec::new();
    for (relative, text) in &sources {
        if !relative.starts_with("crates/si-walk/src/") {
            continue;
        }
        for door in DOORS {
            if text.contains(&format!("drive::{door}("))
                || text.contains(&format!("si_affect::{door}("))
            {
                calls.push(door);
            }
        }
    }
    calls.sort_unstable();
    calls.dedup();
    if calls.is_empty() {
        return Ok("six doors, one home each, budget level-free; the walk half is pending B3 (si-walk has no drive call site yet)".into());
    }
    if calls != ["conduction", "update"] {
        bail!("si-walk calls the drive at {calls:?}; the walk may call exactly conduction and update");
    }
    Ok("six doors, one home each, budget level-free; si-walk calls exactly update and conduction".into())
}

/// 05 §20 `check_idle_no_commit`: symbol reachability from `si-walk`. The walk
/// and its idle cascade may read the web and the session's integrator; the
/// write path (learn, audit, sleep, store mutation) must be unreachable.
fn check_idle_no_commit(ws: &Workspace) -> Result<String> {
    let manifest = ws.read("crates/si-walk/Cargo.toml")?;
    for forbidden in ["si-learn", "si-audit", "si-sleep"] {
        if manifest.contains(forbidden) {
            bail!(
                "si-walk's manifest declares `{forbidden}`; the idle path may not reach the write path (G10(b))"
            );
        }
    }
    const BANNED: [&str; 9] = [
        "si_learn",
        "si_audit",
        "si_sleep",
        "append_row",
        ".age(",
        "commit(",
        "supersede(",
        "bake_q12(",
        "&mut Lattice",
    ];
    let mut scanned = 0usize;
    let mut hits = Vec::new();
    for (relative, text) in ws.source_text()? {
        if !relative.starts_with("crates/si-walk/src/") {
            continue;
        }
        scanned += 1;
        for (index, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            for needle in BANNED {
                if code.contains(needle) {
                    hits.push(format!("{relative}:{}: `{needle}`", index + 1));
                }
            }
        }
    }
    if hits.is_empty() && scanned == 0 {
        bail!("si-walk has no source module to scan");
    }
    if !hits.is_empty() {
        bail!("si-walk reaches the write path: {}", hits.join("; "));
    }
    Ok(format!(
        "si-walk's {scanned} source module(s) name no write-path or commit symbol, and the manifest has no write-side edge (G10(b))"
    ))
}

/// 05 §20 / 02 §9 / 04 §21 U.3 `check_audit_edges`: the audit crate is a leaf
/// with no causal path into control. It imports only the shared kernel plus its
/// two serialization helpers, and it is imported by none of the loop crates — the
/// label stream joins only in the exporter/viewer (si-cli, si-vulkan), never in
/// the session, so no label can steer a later step (R-24, G9(c.1)).
fn check_audit_edges(ws: &Workspace) -> Result<String> {
    let audit_manifest = ws.crate_manifest("si-audit")?;
    let allowed_outgoing = ["si-core", "anyhow", "serde"];
    for key in dependency_keys(&audit_manifest) {
        if !allowed_outgoing.contains(&key.as_str()) {
            bail!(
                "si-audit depends on `{key}`: the audit crate may import only the kernel and its \
                 serialization helpers, never a control organ (R-24)"
            );
        }
    }

    // The permitted join points are the exporter and the viewer; every loop crate
    // must import nothing named si-audit.
    const JOIN_POINTS: [&str; 2] = ["si-cli", "si-vulkan"];
    let mut importers = Vec::new();
    for name in ws.crate_names()? {
        if name == "si-audit" || JOIN_POINTS.contains(&name.as_str()) {
            continue;
        }
        let manifest = ws.crate_manifest(&name)?;
        if dependency_keys(&manifest).iter().any(|key| key == "si-audit") {
            importers.push(name);
        }
    }
    if !importers.is_empty() {
        bail!(
            "a control crate imports si-audit: {}; labels are produced downstream, over the written log, \
             so none of them may steer the loop (R-24)",
            importers.join(", ")
        );
    }
    Ok("si-audit imports only si-core/anyhow/serde and is imported by no loop crate; the label stream \
        joins only in the exporter/viewer (G9(c.1), R-24)"
        .into())
}

fn check_no_mood_field(ws: &Workspace) -> Result<String> {
    const NAMES: [&str; 4] = ["median", "mood", "bias", "bias_state"];
    let mut scanned = 0usize;
    let mut hits = Vec::new();
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) {
            continue;
        }
        scanned += 1;
        for (index, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            let trimmed = code.trim_start();
            for name in NAMES {
                let field = trimmed.starts_with(&format!("pub {name}:"))
                    || trimmed.starts_with(&format!("{name}:"));
                let key = code.contains(&format!("\"{name}\""));
                if field || key {
                    hits.push(format!("{relative}:{}: `{name}`", index + 1));
                }
            }
        }
    }
    if !hits.is_empty() {
        bail!("a stored mood field exists: {}", hits.join("; "));
    }
    Ok(format!(
        "no stored field or export key named median/mood/bias across {scanned} source files (R-28); the read-side median is a return value, never a field"
    ))
}

// ------------------------------------------------------------ B4 stage checks

/// 05 §20 `check_teach_left_side`: the left side is accepted only when it
/// decodes to one of the four question frames (09 §2.1), and there is no
/// widening machinery — no POS guesser, template list or synonym table.
fn check_teach_left_side(ws: &Workspace) -> Result<String> {
    use si_core::patterns::PatternId;
    use si_core::strings::Source;
    use si_front::{decide, TeachOutcome, LEFT_ACCEPT_FRAMES};

    const ACCEPTED: [PatternId; 4] =
        [PatternId::PQuestA, PatternId::PQuestD, PatternId::PQuestW, PatternId::POfRel];
    if LEFT_ACCEPT_FRAMES != ACCEPTED {
        bail!("the teach accept set is {:?}, want the four question frames", LEFT_ACCEPT_FRAMES);
    }

    const LINES: [(&str, bool); 6] = [
        ("what color is apple", true),
        ("what is apple", true),
        ("when was apple", true),
        ("red of apple", true),
        ("apple is red", false),
        ("znark means floop", false),
    ];
    for (left, accepted) in LINES {
        let line = format!("/teach {left} => content");
        let decision = decide(&line, Source::Owner)
            .ok_or_else(|| anyhow::anyhow!("`{line}` was not read as a teach line"))?;
        if accepted {
            if decision.outcome != TeachOutcome::Row || decision.rows != 1 {
                bail!(
                    "`{left}` decoded to {:?} with {} rows; want one row",
                    decision.outcome,
                    decision.rows
                );
            }
        } else {
            if decision.outcome != TeachOutcome::LeftNotAQuestion || decision.rows != 0 {
                bail!(
                    "`{left}` decoded to {:?} with {} rows; want left_not_a_question and zero",
                    decision.outcome,
                    decision.rows
                );
            }
            if decision.outcome.reason() != "left_not_a_question" {
                bail!("the rejection's reason is `{}`", decision.outcome.reason());
            }
        }
    }

    const WIDENING: [&str; 5] = ["synonym", "thesaurus", "pos_tag", "part_of_speech", "lemmat"];
    let mut hits = Vec::new();
    let mut scanned = 0usize;
    for (relative, text) in ws.source_text()? {
        if !relative.starts_with("crates/si-front/src/") || Workspace::is_auditor(&relative) {
            continue;
        }
        scanned += 1;
        let lower = text.to_ascii_lowercase();
        for needle in WIDENING {
            if lower.contains(needle) {
                hits.push(format!("{relative}: {needle}"));
            }
        }
    }
    if !hits.is_empty() {
        bail!("the teach channel carries a widening table or a tagger: {}", hits.join("; "));
    }
    Ok(format!(
        "the accept set is the four question frames; a statement left side is `left_not_a_question` with zero rows; {scanned} si-front source module(s) name no synonym/tagger machinery"
    ))
}

/// 05 §20 `check_no_teach_vocabulary`: beyond `/teach`, `=>` and `owner-teach`
/// (03 §3.2's allowlist) the channel owns no literal, and no other crate
/// redeclares those three markers.
fn check_no_teach_vocabulary(ws: &Workspace) -> Result<String> {
    use si_core::TEACH_LITERALS;

    const REASONS: [&str; 5] = [
        "",
        "grimm_source_inert",
        "left_undecoded",
        "empty_response",
        "left_not_a_question",
    ];
    let source = ws.read("crates/si-front/src/teach.rs")?;
    let mut stray = Vec::new();
    for literal in string_literals(&source) {
        if !REASONS.contains(&literal.as_str()) {
            stray.push(literal);
        }
    }
    if !stray.is_empty() {
        bail!("the teach channel owns literals beyond 03 §3.2's allowlist: {stray:?}");
    }

    let mut redeclared = Vec::new();
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) || relative.starts_with("crates/si-core/") {
            continue;
        }
        for literal in string_literals(&text) {
            if TEACH_LITERALS.contains(&literal.as_str()) {
                redeclared.push(format!("{relative}: `{literal}`"));
            }
        }
    }
    if !redeclared.is_empty() {
        bail!("a teach marker is redeclared outside the allowlist's home: {}", redeclared.join("; "));
    }
    Ok("teach.rs carries only its five outcome codes; /teach, => and owner-teach exist once, in si-core (03 §3.2)".into())
}

/// 05 §20 `check_hop_cap_is_claim_side`: the cap appears only in the registry,
/// the version-tag hasher and the labeler's citation check — never inside
/// `walk`/`spread`/`settle_or_stop` — and an over-cap route downgrades rather
/// than truncates (04 §5.4, G11(i)).
fn check_hop_cap_is_claim_side(ws: &Workspace) -> Result<String> {
    use si_core::consts::{self, Class};
    use si_front::{label, Caps, ClaimLabel, Flavor, LabelInput, Reason, Rendered, HOP_CAP_ROW};
    use si_read::{Activation, Cited, RouteCounts, StopReason, Thought};

    let row = consts::row(HOP_CAP_ROW)
        .ok_or_else(|| anyhow::anyhow!("{HOP_CAP_ROW} is not a registry row"))?;
    if row.class != Class::Species || row.home != "si-walk" || row.gate != "G11" {
        bail!("{HOP_CAP_ROW} is class {:?}, home `{}`, gate `{}`", row.class, row.home, row.gate);
    }
    let registry = consts::registry_value(row)
        .ok_or_else(|| anyhow::anyhow!("{HOP_CAP_ROW} carries no registry value"))?;
    if Caps::registry().max_owned_hops as i64 != registry {
        bail!("the labeler's cap is not the registry value {registry}");
    }

    const PERMITTED: [&str; 3] = [
        "crates/si-core/src/consts/mod.rs",
        "crates/si-core/src/hash.rs",
        "crates/si-front/src/label.rs",
    ];
    let mut outside = Vec::new();
    let mut in_walk = Vec::new();
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) || !text.contains(HOP_CAP_ROW) {
            continue;
        }
        if relative.starts_with("crates/si-front/tests/") || PERMITTED.contains(&relative.as_str()) {
            continue;
        }
        if relative.starts_with("crates/si-walk/") || relative.starts_with("crates/si-read/") {
            in_walk.push(relative.clone());
        }
        outside.push(relative);
    }
    if !in_walk.is_empty() {
        bail!("the walk/read path reads {HOP_CAP_ROW}: {in_walk:?}; the cap is claim-side (G11(i))");
    }
    if !outside.is_empty() {
        bail!("{HOP_CAP_ROW} is read outside the registry/hasher/labeler: {outside:?}");
    }

    let thought = Thought {
        settle_set: vec![Activation { point: 20, activation: 4096, first_lit: 0 }],
        rounds: Vec::new(),
        a_trajectory: Vec::new(),
        stop_reason: StopReason::Settled,
        lit_round0: Vec::new(),
        relit: Vec::new(),
        oscillated: false,
        frame_log: Default::default(),
    };
    let cited = vec![
        Cited {
            row_id: 1,
            kind: 0,
            confidence: 95,
            active_at_anchor: true,
            key_point: 10,
            val_point: 20,
            surface: "red".to_string(),
            nameless: false,
        },
        Cited {
            row_id: 2,
            kind: 0,
            confidence: 95,
            active_at_anchor: true,
            key_point: 11,
            val_point: 21,
            surface: "green".to_string(),
            nameless: false,
        },
    ];
    let rendered = Rendered {
        surface: "red green".to_string(),
        content: vec!["red".to_string(), "green".to_string()],
        citations: Vec::new(),
        leaked: Vec::new(),
        containment: true,
    };
    let judge = |hops: u32| {
        label(&LabelInput {
            thought: &thought,
            caps: &Caps { max_owned_hops: 2, engine: false },
            cited: &cited,
            rendered: &rendered,
            question_shaped: true,
            named_dark: false,
            route: RouteCounts { owned: hops, base: 0, hops },
        })
    };
    let inside = judge(2);
    if inside.label != ClaimLabel::Inference || inside.reason != Reason::Composed {
        bail!("a route at the cap labeled {:?}/{:?}, want INFERENCE", inside.label, inside.reason);
    }
    let over = judge(3);
    if over.label != ClaimLabel::Speculation
        || over.flavor != Flavor::Uncertainty
        || over.reason != Reason::Depth
    {
        bail!(
            "an over-cap route labeled {:?}/{:?}, want the uncertainty surface",
            over.label,
            over.reason
        );
    }
    if over.citation.len() != cited.len() {
        bail!(
            "the over-cap route cites {} of {} reached rows; the full route is the evidence",
            over.citation.len(),
            cited.len()
        );
    }
    Ok(format!(
        "the cap is the registry value {registry}; its only reads are the registry, the version-tag hasher and si-front's labeler; at the cap a composed route is INFERENCE, one hop over it is UNCERTAINTY citing the full route"
    ))
}

/// 05 §20 `check_candidate_unanswerable`: the FACT predicate and retrieve's
/// cited-row set share one gate, and no `6 cooccur-weak` row or
/// `confidence = 0` row can source an answer (G11(p), R-01).
fn check_candidate_unanswerable(ws: &Workspace) -> Result<String> {
    use std::collections::BTreeMap;

    use si_affect::params::LagUnit;
    use si_affect::{Integrator, Params};
    use si_core::addr::SHELF_S_R_PAIRS;
    use si_core::patterns::PatternId;
    use si_core::record::{answerable_meta, Body, Envelope};
    use si_read::light::Resolve;
    use si_read::{retrieve, ReadRequest};
    use si_store::store::{EdgeDraft, Lattice};
    use si_walk::{Curve, CurvePoint, OscillationRule, WalkConfig};

    const TRUTH: [(u16, u8, bool, bool); 6] = [
        (95, 0, true, true),
        (95, 5, true, true),
        (95, 6, true, false),
        (0, 0, true, false),
        (95, 0, false, false),
        (1, 5, true, true),
    ];
    for (confidence, kind, active, expected) in TRUTH {
        if answerable_meta(confidence, kind, active) != expected {
            bail!("answerable_meta({confidence}, {kind}, {active}) is not {expected}");
        }
    }

    let retrieve_source = ws.read("crates/si-read/src/retrieve.rs")?;
    if !retrieve_source.contains("answerable_meta(") {
        bail!("retrieve does not consult answerable_meta");
    }
    let label_source = ws.read("crates/si-front/src/label.rs")?;
    if !label_source.contains("answerable_meta(") {
        bail!("the FACT/INFERENCE predicate does not consult answerable_meta");
    }
    let mut second = Vec::new();
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) {
            continue;
        }
        if !(relative.starts_with("crates/si-read/src/")
            || relative.starts_with("crates/si-front/src/"))
        {
            continue;
        }
        if text.contains("confidence > 0") || text.contains("confidence != 0") {
            second.push(relative);
        }
    }
    if !second.is_empty() {
        bail!("a second answerability predicate exists in {second:?}");
    }

    struct Map(BTreeMap<String, Vec<u32>>);
    impl Resolve for Map {
        fn aliases(&self, key: &str) -> Vec<u32> {
            self.0.get(key).cloned().unwrap_or_default()
        }
    }
    fn seed_row(
        lattice: &mut Lattice,
        key_point: u32,
        val_point: u32,
        kind: u8,
        confidence: u16,
        surface: &str,
    ) -> Result<u32> {
        let resp_surface = lattice.interner_mut().intern(surface);
        let mut body = Body::default();
        body.pattern_id = PatternId::PQuestA.index() as u16;
        body.key_point = key_point;
        body.val_point = val_point;
        body.resp_surface = resp_surface;
        let mut edge = EdgeDraft::owned(val_point, 4096, 0);
        edge.kind = kind;
        let mut envelope = Envelope::live(SHELF_S_R_PAIRS, 1, 0, 1);
        envelope.confidence = confidence;
        lattice.append_row(envelope, body, vec![edge], &[], &[])
    }

    let mut lattice = Lattice::new();
    let owned = seed_row(&mut lattice, 10, 20, 0, 95, "it is red")?;
    seed_row(&mut lattice, 11, 21, 6, 95, "it is maybe")?;
    seed_row(&mut lattice, 12, 22, 0, 0, "it is cold")?;
    let resolve = Map(
        [
            ("apple".to_string(), vec![10u32]),
            ("pear".to_string(), vec![11u32]),
            ("plum".to_string(), vec![12u32]),
        ]
        .into_iter()
        .collect(),
    );

    let params = Params {
        lambda_a: 128,
        gain: [0; 6],
        beta: 4096,
        gamma: 4096,
        ask_cap_per_input: 1,
        priority: [4096, 4096, 4096],
        gap_salience_floor: 0,
        dedupe_window: 0,
        bias_cap_share: 4096,
        b_max: 4096,
        k_tanh: 1,
        median_min_n: 1,
        median_shape: 0,
        lag_rounds: 1,
        lag_unit: LagUnit::PerRound,
    };
    let curve = Curve::fixture(CurvePoint { reach: 1, decay_q12: 4096, settle_shift_q12: 0 });
    let config = WalkConfig {
        params: &params,
        curve: &curve,
        k_max: 1,
        tau_settle: 1,
        decay_base: 128,
        hub_exponent_q12: 4096,
        frontier_cap: 16,
        oscillation: OscillationRule::MaxKOnly,
        ghost_max: 0,
        shape_max: 0,
        // 04 §31 AC.1: a caller with no index passes the shipped off state.
        tugs: &[],
        relight_max: 0,
        composition_hint: &[],
    };
    let mut integrator = Integrator::at_birth();
    let read = retrieve(
        &lattice,
        lattice.interner(),
        &resolve,
        &mut integrator,
        &config,
        &ReadRequest {
            keys: vec!["apple".to_string(), "pear".to_string(), "plum".to_string()],
            frame_point: None,
            slot: None,
            anchor: 10,
            thought_id: 1,
        },
    )?;
    let seen: Vec<(u32, u8, u16)> =
        read.cited.iter().map(|row| (row.row_id, row.kind, row.confidence)).collect();
    if seen != vec![(owned, 0, 95)] {
        bail!("retrieve cited {seen:?}; a cooccur-weak row and a zero-confidence row sourced a claim");
    }

    // 12 §6.3's grown arm (**B18**): **a nomination candidate can never be an
    // answer's source.** The row is a real step-2 append — gap kind 6
    // (`move-nomination`) on the nomination lane's own origin 2 (OD-128) — written
    // at the *same settled handle* the cited belief row sits on. That placement is
    // the whole of the arm: nothing about it is unreachable, so the only thing that
    // can keep it out of `cited` is the shared predicate. Whichever way OD-58/98
    // land on a candidate's answerability, an open ask is a gap, and a gap does not
    // answer itself.
    let nomination = si_learn::gate::append_question(
        &mut lattice,
        &si_learn::gate::QuestionSpec {
            pattern_id: PatternId::PQuestA.index() as u16,
            gap_kind: si_learn::gate::GAP_MOVE_NOMINATION,
            origin: si_learn::gate::ASK_ORIGIN_NOMINATION,
            key_point: 10,
            target_slot: 0,
            text_ref: 0,
            prov_step: 1,
            arousal_at_ask: 0,
        },
        &si_learn::gate::WriteContext {
            t: 1,
            prov_session: 0,
            source: si_core::strings::Source::Owner,
            class: si_core::world::SpeakerClass::Owner,
            stage: 0,
            a_formation: 0,
            thought_id: 1,
            signal_ref: 1,
        },
    )?;
    let open = si_learn::gate::open_asks(&lattice, 10)?;
    let ask = open
        .iter()
        .find(|row| row.row_id == nomination)
        .with_context(|| "the nomination append is not an open ask".to_string())?;
    if ask.key_point != 10
        || ask.status != si_learn::gate::Q_OPEN
        || ask.gap_kind() != si_learn::gate::GAP_MOVE_NOMINATION
        || ask.origin() != si_learn::gate::ASK_ORIGIN_NOMINATION
    {
        bail!("the planted nomination is not the row the owner lane reads: {ask:?}");
    }
    let ask_kind = si_read::retrieve::primary_kind(&lattice, &ask.record)?;
    if ask.record.envelope.confidence != 0 || ask_kind != si_core::addr::EDGE_KIND_COOCCUR_WEAK {
        bail!(
            "the nomination ask is answerable by construction (confidence {}, kind {ask_kind}); \
             the grown arm would grade a row the predicate was never asked about",
            ask.record.envelope.confidence
        );
    }
    let mut second_integrator = Integrator::at_birth();
    let with_ask = retrieve(
        &lattice,
        lattice.interner(),
        &resolve,
        &mut second_integrator,
        &config,
        &ReadRequest {
            keys: vec!["apple".to_string(), "pear".to_string(), "plum".to_string()],
            frame_point: None,
            slot: None,
            anchor: 10,
            thought_id: 2,
        },
    )?;
    let cited_again: Vec<u32> = with_ask.cited.iter().map(|row| row.row_id).collect();
    if cited_again != vec![owned] {
        bail!(
            "with the nomination ask open on the settled handle, retrieve cited {cited_again:?}; \
             a gap row sourced an answer"
        );
    }
    Ok(format!(
        "one shared predicate gates both the FACT/INFERENCE rows and retrieve's citations; the \
         truth table holds on six cases and on a lattice where the weak and cold rows were \
         reached but not cited (G11(p)), and B18's grown arm adds row {nomination}: a kind-6 \
         `move-nomination` ask on origin 2, open at the cited row's own handle, excluded by the \
         same predicate (confidence 0, no owned edge ⇒ cooccur-weak) rather than by an \
         unreachable point"
    ))
}

// ------------------------------------------------------------ B5 stage checks

/// Drop a line's trailing `//` comment so a scan reads only code.
fn code_part(line: &str) -> &str {
    line.split_once("//").map_or(line, |(code, _)| code)
}

/// 05 §20 `check_bake_write_once`: every assignment to `bake` is on the formation
/// path and the stored field is write-once. Two structural facts — exactly one
/// production site computes a bake (`drive::salience`, from `a_formation`), and no
/// code mutates a stored record's `.bake` after formation.
fn check_bake_write_once(ws: &Workspace) -> Result<String> {
    let mut computes = Vec::new();
    let mut mutations = Vec::new();
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative)
            || !relative.starts_with("crates/")
            || !relative.contains("/src/")
        {
            continue;
        }
        for (index, raw) in text.lines().enumerate() {
            let line = code_part(raw);
            if line.contains("bake_q12(") && line.contains("let bake =") {
                computes.push(format!("{relative}:{}", index + 1));
            }
            if let Some(pos) = line.find(".bake") {
                let tail = line[pos + ".bake".len()..].trim_start();
                if tail.starts_with('=') && !tail.starts_with("==") {
                    mutations.push(format!("{relative}:{}", index + 1));
                }
            }
        }
    }
    if computes.len() != 1 {
        bail!(
            "expected exactly one bake computation site, found {}: {}",
            computes.len(),
            computes.join(", ")
        );
    }
    if !mutations.is_empty() {
        bail!("a stored `bake` is mutated after formation (not write-once): {}", mutations.join(", "));
    }
    Ok(format!(
        "`bake` is computed once ({}) from `a_formation`; no stored `.bake` is ever reassigned — write-once (G9, OD-33)",
        computes[0]
    ))
}

/// 05 §20 / 04 §5.4 law 20 (I9, R-25): the composition path — settle → label →
/// render — lives in `si-walk` and `si-read` and must reach a commit/write
/// primitive never. That is structural: neither crate depends on `si-learn`, and
/// neither names a learn/write symbol.
fn check_compose_writes_nothing(ws: &Workspace) -> Result<String> {
    for crate_name in ["si-walk", "si-read"] {
        let manifest = ws.crate_manifest(crate_name)?;
        if manifest.contains("si-learn") {
            bail!("{crate_name} depends on si-learn; the composition path could reach learn.commit (I9/R-25)");
        }
    }
    const NEEDLES: [&str; 7] = [
        "si_learn",
        "si-learn",
        "learn::write",
        "learn::commit",
        "gate::write",
        "append_row",
        ".commit(",
    ];
    let mut hits = Vec::new();
    for (relative, text) in ws.source_text()? {
        if !(relative.starts_with("crates/si-walk/src/") || relative.starts_with("crates/si-read/src/")) {
            continue;
        }
        // Symbol reachability, not prose: a `//!` doc-comment that names the
        // crate to assert this very invariant is not a reachable write symbol.
        for raw in text.lines() {
            let line = code_part(raw);
            for needle in NEEDLES {
                if line.contains(needle) {
                    hits.push(format!("{relative}: {needle}"));
                }
            }
        }
    }
    if !hits.is_empty() {
        bail!("a write primitive is reachable from the composition path: {}", hits.join("; "));
    }
    Ok("si-walk and si-read depend on neither si-learn nor a lattice write; the composed inference reaches commit never (I9, R-25, G11(h))".into())
}

/// 09 §10 / 05 §20: the bulk `/teach` feeder is an inbox writer and nothing else.
/// The CLI binary that hosts `feed` imports no learn, no walk and no lattice write,
/// so its lines cannot reach the write gate; it opens the curriculum file and
/// `sense_inbox.siro` and no store symbol.
///
/// The same row of the law has a **second half**, and B11.7 found it unwired:
/// `G1.u.2` licenses the feeder only when a fed inbox reaches the byte-identical state
/// of the same lines typed one by one, which needs a reader. Measured on this build
/// before the wiring: a session fed two `/teach` lines settled at
/// `c9654e12086696a0aae43a459833b5ec` — the hash of the same session fed nothing — while
/// the typed arm reached `b71cf552d1219f4cfb8caeff27eb40c8` and committed 2 rows. So this
/// leg now asks for both halves: a writer that writes nothing else, and a run path that
/// hears the lane through the loop's own reader.
fn check_curriculum_is_inbox_only(ws: &Workspace) -> Result<String> {
    let cli = ws.read("crates/si-cli/src/bin/si-cli.rs")?;
    const NEEDLES: [&str; 7] = [
        "si_learn",
        "si-learn",
        "si_walk",
        "si-walk",
        "Lattice",
        "learn::write",
        ".commit(",
    ];
    for needle in NEEDLES {
        if cli.contains(needle) {
            bail!("the feeder's binary names `{needle}`; the importer became a writer (R-11, R-12)");
        }
    }
    if !cli.contains("fn feed_inbox") || !cli.contains("sense_inbox.siro") {
        bail!("si-cli has no inbox-only feed path (feed_inbox → sense_inbox.siro)");
    }
    // The bytes the feeder writes are the loop crate's writer, not a second one: the
    // `G1.u.2` arms feed the lane this same function builds.
    if !cli.contains("encode_sense_inbox") {
        bail!(
            "the feeder builds its own lane bytes: si-cli.rs does not call \
             `Session::encode_sense_inbox`, so a fed arm and `si-cli feed` would be two \
             writers of one idea (09 §8)"
        );
    }
    if !cli.contains("hear_sense_inbox") {
        bail!(
            "the feeder has no reader in the run path: si-cli.rs never calls \
             `Session::hear_sense_inbox`, so a fed `sense_inbox.siro` is a lane nothing hears and \
             `G1.u.2`'s byte-identity is unmeasurable (09 §8's \"the ordinary loop\" half)"
        );
    }
    // The reader is the loop's, not the CLI's: it decodes the lane and applies each frame
    // through `tick_frame`, so a fed line takes the same steps 1–13 a keystroke takes.
    let session_src = ws.read("crates/si-session/src/lib.rs")?;
    for needle in ["fn hear_sense_inbox", "read_sense_inbox", "tick_frame", "fn encode_sense_inbox"] {
        if !session_src.contains(needle) {
            bail!(
                "si-session names no `{needle}`: 09 §8's reader half lives outside the loop, so \
                the feeder and a gate would be two writers of one idea (R-11)"
            );
        }
    }
    Ok("the feeder reads a curriculum file and writes sense_inbox.siro only, and the run path \
        hears that lane through Session::hear_sense_inbox — which lives in si-session and \
        applies every frame with tick_frame, the same steps 1–13 a keystroke takes. Both halves \
        of the lane are si-session's: Session::encode_sense_inbox is the writer `si-cli feed` \
        calls and the writer `G1.u.2`'s fed arm feeds, so the feeder and a gate leg are one \
        code path (G1.u.2, 09 §8); the CLI binary imports no learn, no walk, no lattice \
        (G1.u, R-11/R-12)"
        .into())
}

/// 05 §20 / 08 §9 / G14(j): no re-roll verb and no sleep or temperament control
/// over the seam in the interactive CLI, the Vulkan panel or the supporter. The
/// seed is required at creation and only *compared* at resume.
fn check_no_seed_control_surface(ws: &Workspace) -> Result<String> {
    const NEEDLES: [&str; 10] = [
        "reroll",
        "re_roll",
        "re-roll",
        "resample",
        "roll_seed",
        "--seed=",
        "sleep now",
        "sleep-now",
        "sleep_now",
        "new temperament",
    ];
    let mut hits = Vec::new();
    let mut refusal_tables = 0usize;
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative)
            || relative.starts_with("crates/si-audit/")
            || relative.contains("/tests/")
        {
            continue;
        }
        if !(relative.starts_with("crates/si-cli/src/")
            || relative.starts_with("crates/si-vulkan/src/")
            || relative.starts_with("crates/si-supporter/src/"))
        {
            continue;
        }
        // 08 §9's refusal table is the law *quoted inside* the viewer: each entry names
        // the widget it refuses ("sleep now", "re-roll the temperament"). The scan is of
        // the control surface, so the table's own lines are exempt — and because a table
        // has to be found for the exemption to apply at all, the exemption cannot cover
        // for a widget that was actually built.
        let text = match refusal_table_span(&text) {
            Some((start, end)) => {
                refusal_tables += 1;
                text.lines()
                    .enumerate()
                    .filter(|(number, _)| !(*number >= start && *number < end))
                    .map(|(_, line)| line)
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            None => text,
        };
        let lower = text.to_lowercase();
        for needle in NEEDLES {
            if lower.contains(needle) {
                hits.push(format!("{relative}: {needle}"));
            }
        }
    }
    if !hits.is_empty() {
        bail!("a seed/sleep control surface exists: {}", hits.join("; "));
    }
    if refusal_tables == 0 {
        bail!(
            "no 08 §9 refusal table is declared anywhere: the exempt span is the viewer naming the \
             widgets it will not build, and without it §9 is a review note"
        );
    }
    Ok(format!(
        "no re-roll, no `--seed=` assignment and no sleep or temperament verb in si-cli, si-vulkan or \
         si-supporter — {refusal_tables} §9 refusal table(s) exempted by span, and a §9 widget built \
         outside one still trips this scan; --seed is required at creation and only compared at \
         resume (G14(j))"
    ))
}

/// The line span of 08 §9's refusal table: the declaration through the closing
/// `];`. `None` when a file declares none.
fn refusal_table_span(text: &str) -> Option<(usize, usize)> {
    let start = text.lines().position(|line| line.contains("const FORBIDDEN_WIDGETS"))?;
    let end = text.lines().skip(start).position(|line| line.trim().ends_with("];"))?;
    Some((start, start + end + 1))
}

// ------------------------------------------------------------- B9 stage checks

/// One line with its `//` tail removed — the house idiom of these scans, since a
/// doc comment is allowed to name the quantity it forbids.
fn code_of(line: &str) -> &str {
    line.split("//").next().unwrap_or("")
}

/// A whole file with every comment line's code-only text joined back: the ban
/// scans are over what the code names, never over the prose that explains the ban.
fn code_text(text: String) -> String {
    text.lines()
        .map(|line| code_of(line).trim_start())
        .collect::<Vec<_>>()
        .join("\n")
}

/// 05 §20 `check_no_affective_setter`: rule (vi) in its mechanical form. The
/// supporter's outgoing edges are the kernel and the store, so no module it can
/// reach holds `a`, charge, `bake`, the median or a door — and no line of its own
/// code names one. `soothe()` is R-21's fourth shape and this is the wall it hits.
fn check_no_affective_setter(ws: &Workspace) -> Result<String> {
    let manifest = ws.crate_manifest("si-supporter")?;
    let allowed_outgoing = ["si-core", "si-store", "anyhow", "serde"];
    let mut edges = Vec::new();
    for key in dependency_keys(&manifest) {
        if !allowed_outgoing.contains(&key.as_str()) {
            bail!(
                "si-supporter depends on `{key}`: the seam and the mouth filter import the kernel and \
                 the store and nothing else, which is what makes rule (vi) a build-graph fact rather \
                 than a runtime check (00 §2, R-34)"
            );
        }
        if key.starts_with("si-") {
            edges.push(key);
        }
    }

    const NEEDLES: [&str; 8] = [
        "si_affect",
        "a_formation",
        "arousal",
        "charge",
        "bake",
        "median",
        "salience",
        "door",
    ];
    let mut hits = Vec::new();
    let mut scanned = 0usize;
    for (relative, text) in ws.source_text()? {
        if !relative.starts_with("crates/si-supporter/src/") {
            continue;
        }
        scanned += 1;
        for (index, line) in text.lines().enumerate() {
            let code = code_of(line);
            for needle in NEEDLES {
                if code.contains(needle) {
                    hits.push(format!("{relative}:{}: `{needle}`", index + 1));
                }
            }
        }
    }
    if !hits.is_empty() {
        bail!(
            "the mouth reads an affective quantity: {}; the speak gate sits upstream of sense and has no \
             route to `a` (G12(d), rule (vi))",
            hits.join("; ")
        );
    }
    Ok(format!(
        "si-supporter imports {} and none of its {scanned} source lines names a, charge, bake, the median \
         or a door (G12(d), rule (vi), R-34)",
        edges.join(" + ")
    ))
}

/// 05 §20 `check_no_stage_on_supporter`: §11.2's and §21.6's law in code form. The
/// stage is a function of the written rows and of nothing else — neither the
/// supporter's presence, nor Grimm's readiness, nor an affect quantity enters a
/// transition predicate.
fn check_no_stage_on_supporter(ws: &Workspace) -> Result<String> {
    const NEEDLES: [&str; 9] = [
        "source",
        "Source",
        "grimm",
        "Grimm",
        "readiness",
        "a_formation",
        "arousal",
        "charge",
        "salience",
    ];
    const TRANSITIONS: [(&str, &str); 2] = [
        ("crates/si-learn/src/gate.rs", "pub fn stage_of"),
        ("crates/si-session/src/lib.rs", "pub fn stage(session"),
    ];
    let sources = ws.source_text()?;
    let mut hits = Vec::new();
    let mut bodies = 0usize;
    for (relative, signature) in TRANSITIONS {
        let text = sources
            .iter()
            .find(|(path, _)| path == relative)
            .map(|(_, text)| text.clone())
            .ok_or_else(|| anyhow::anyhow!("{relative} is not in the workspace sources"))?;
        let body = function_body(&text, signature)
            .ok_or_else(|| anyhow::anyhow!("{relative} has no readable `{signature}` body"))?;
        bodies += 1;
        for (index, line) in body.lines().enumerate() {
            let code = code_of(line);
            for needle in NEEDLES {
                if code.contains(needle) {
                    hits.push(format!("{relative} {signature}:{}: `{needle}`", index + 1));
                }
            }
        }
    }
    if !hits.is_empty() {
        bail!(
            "a stage-transition predicate reads the speaker or an affect quantity: {}; the stage is \
             derived from the written rows alone (05 §20, §11.2, §21.6)",
            hits.join("; ")
        );
    }
    Ok(format!(
        "{bodies} transition bodies name no source, no Grimm readiness and no affect quantity; the stage \
         is a function of row counts alone (G12(d), 04 §11.2, docs/21 §21.6)"
    ))
}

// ----------------------------------------------------------- B14 stage checks

/// engineering/12 §6.3 / §22.0's litmus: the two reader-only leaf crates may read
/// the store and nothing else. Their `observe` entry points take `&Lattice`, so
/// the compiler already refuses a write; this check makes the property visible —
/// **zero** `&mut` anywhere in either crate, and neither manifest reaches a
/// writer, the affect organ or the audit labeler (OD-98, G15(a)(b)(d), G18(a)(b)(d)).
fn check_crystallize_no_commit(ws: &Workspace) -> Result<String> {
    const DIRS: [&str; 2] = ["crates/si-crystallize/src/", "crates/si-skills/src/"];
    const BANNED: [&str; 11] = [
        "si_learn",
        "si_affect",
        "si_audit",
        "si_sleep",
        "si_front",
        "append_row",
        "commit(",
        "supersede(",
        ".age(",
        "bake_q12(",
        "salience",
    ];
    let mut scanned = 0usize;
    let mut hits = Vec::new();
    for (relative, text) in ws.source_text()? {
        if !DIRS.iter().any(|dir| relative.starts_with(dir)) {
            continue;
        }
        scanned += 1;
        // The exemption `check_ghost_is_frame_scoped` already grants its own
        // scan, for the same reason: a `#[cfg(test)]` module compiles into no
        // shipped binary, so it cannot carry the write capability this check
        // exists to deny. The rules test needs a store fixture — `append_row`
        // and a `&mut Lattice` are how a member row and its edges get written
        // at all (02 §2) — and policing the *scenario* would leave the reader's
        // four rules graded by the gate alone. Every line above the marker
        // stays scanned. (OD-160.)
        let text = text.split("#[cfg(test)]").next().unwrap_or("").to_string();
        for (index, line) in text.lines().enumerate() {
            let code = code_of(line);
            for needle in BANNED {
                if code.contains(needle) {
                    hits.push(format!("{relative}:{}: `{needle}`", index + 1));
                }
            }
            // `&mut self` is the detector's own table and is the point of
            // `observe`; a `&mut` on the store is the write-capable edge this
            // check exists to deny (12 §1.1).
            for (offset, _) in code.match_indices("&mut ") {
                let rest = &code[offset + "&mut ".len()..];
                let target: String = rest
                    .chars()
                    .take_while(|ch| ch.is_alphanumeric() || *ch == '_' || *ch == ':')
                    .collect();
                if ["Lattice", "Store", "store", "lattice", "si_store"]
                    .iter()
                    .any(|name| target.starts_with(name))
                {
                    hits.push(format!("{relative}:{}: `&mut {target}`", index + 1));
                }
            }
        }
    }
    if scanned == 0 {
        bail!("neither si-crystallize nor si-skills has a source file to scan (12 §1.1)");
    }
    if !hits.is_empty() {
        bail!(
            "a reader crate reaches the write path: {}; si-crystallize and si-skills take `&Lattice` \
             and store nothing (12 §6.3, docs/22 §22.0)",
            hits.join("; ")
        );
    }
    let mut manifests = 0usize;
    for name in ["si-crystallize", "si-skills"] {
        let manifest = ws.crate_manifest(name)?;
        for key in dependency_keys(&manifest) {
            if ["si-learn", "si-affect", "si-audit", "si-sleep", "si-front", "si-walk"]
                .contains(&key.as_str())
            {
                bail!("{name} declares `{key}`; a reader crate may not reach a writer (12 §1.1)");
            }
        }
        manifests += 1;
    }
    Ok(format!(
        "{scanned} reader source file(s) across {manifests} crate(s): no `&mut` on the store, no \
         store write, no affect and no salience symbol; every store call is a read \
         (G15(a)(b)(d), G18(a)(b)(d))"
    ))
}

/// engineering/12 §4.3 (xv) / docs/16 §16.8: the frame scratch is pure data
/// declared once in `si-core::frame`, constructed only inside a `si-walk`
/// thought, read by the two detectors as a slice, and serialized by the session
/// only into `mindlog.json`'s `frame_log[]`. It is in no container, schema or
/// `SIst`/`SIck` write, and it carries no serde derive that could export it.
fn check_ghost_is_frame_scoped(ws: &Workspace) -> Result<String> {
    // Per type, the only places it may be *named*: its declaration, the walk
    // that builds it, and — for `FrameLog` alone — the session that renders the
    // observation and the detectors that receive it as an argument (`&FrameLog`).
    const SCOPES: [(&str, &[&str]); 1] = [(
        "FrameLog",
        &[
            "crates/si-core/src/frame.rs",
            "crates/si-walk/src/",
            "crates/si-crystallize/src/",
            "crates/si-skills/src/",
            "crates/si-session/src/lib.rs",
        ],
    )];
    // 12 §6.2's words are "appears in a container, schema, export, cache or
    // `SIst`/`SIck` write", not "is never named": `si-skills` folds a `&RoleGraph`
    // into a template string, and a name in an argument position keeps no value.
    // So the scratch types are banned where a value could be *kept* — a struct's
    // field list or a collection's element type — and construction stays in the
    // walk. `Role::Ghost` is a role label, not the `Ghost` value, and is removed
    // before the scan.
    const SCRATCH: [&str; 3] = ["CoLight", "Ghost", "RoleGraph"];
    const KEEPERS: [&str; 6] = [
        "Vec<CoLight",
        "Vec<Ghost",
        "Vec<RoleGraph",
        "[CoLight",
        "[Ghost",
        "[RoleGraph",
    ];
    const CONSTRUCTORS: [&str; 3] = ["CoLight {", "Ghost {", "RoleGraph {"];
    const CONSTRUCTED_IN: &str = "crates/si-walk/src/";
    let under = |relative: &str, path: &str| {
        relative == path || (path.ends_with('/') && relative.starts_with(path))
    };
    let mut scanned = 0usize;
    let mut escaped = Vec::new();
    let mut constructed = 0usize;
    for (relative, text) in ws.source_text()? {
        scanned += 1;
        if Workspace::is_auditor(&relative) {
            // 12 §6.3: the auditor is the exempt scope. This check and the gate
            // report name the four types in their own sentences, and a report
            // string is not a value that could keep one (the audit layer writes
            // only `gates/`).
            continue;
        }
        // A unit test of the crate that owns the scratch builds values of it on
        // the stack; that is the opposite of keeping one, and the scan is of
        // what the shipped code could persist.
        let text = text.split("#[cfg(test)]").next().unwrap_or("").to_string();
        let built_here = under(&relative, CONSTRUCTED_IN);
        let lines: Vec<&str> = text.lines().collect();
        // A struct's field list is the container this check exists to police.
        let mut in_struct = false;
        for (index, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ") {
                in_struct = true;
            } else if in_struct && trimmed.starts_with('}') {
                in_struct = false;
            }
            let code = code_of(line).replace("Role::Ghost", "");
            let code = code.as_str();
            if trimmed.starts_with("use ") || trimmed.starts_with("pub use ") {
                continue;
            }
            for (ty, paths) in SCOPES {
                if code.contains(ty) && !paths.iter().any(|path| under(&relative, path)) {
                    escaped.push(format!("{relative}:{}: `{ty}`", index + 1));
                }
            }
            let scratch_allowed = ["crates/si-core/src/frame.rs", "crates/si-walk/src/"]
                .iter()
                .any(|path| under(&relative, path));
            if !scratch_allowed {
                if in_struct && SCRATCH.iter().any(|ty| code.contains(ty)) {
                    escaped.push(format!(
                        "{relative}:{}: a scratch type named in a field list",
                        index + 1
                    ));
                }
                for keeper in KEEPERS {
                    if code.contains(keeper) {
                        escaped.push(format!("{relative}:{}: `{keeper}`", index + 1));
                    }
                }
            }
            for ctor in CONSTRUCTORS {
                if code.contains(ctor) {
                    if built_here && relative.starts_with("crates/si-walk/") {
                        constructed += 1;
                    } else {
                        escaped.push(format!("{relative}:{}: constructed `{ctor}`", index + 1));
                    }
                }
            }
        }
    }
    // `si-core/src/frame.rs`'s own struct declarations carry the constructor
    // spelling, so the construction set excludes that file's lines by name.
    escaped.retain(|hit| !hit.starts_with("crates/si-core/src/frame.rs"));
    let frame = ws.read("crates/si-core/src/frame.rs")?;
    if frame.contains("Serialize") || frame.contains("serialize") {
        bail!("si-core::frame derives or names serde; frame scratch is never exported (12 §2.6)");
    }
    if constructed == 0 {
        bail!("no `si-walk` file constructs the frame scratch; the detectors would read nothing");
    }
    if !escaped.is_empty() {
        bail!(
            "frame scratch escaped its scope: {}; a `Ghost`/`RoleGraph`/`CoLight` value may live only \
             inside one thought (12 §4.3 (xv), docs/16 §16.8)",
            escaped.join("; ")
        );
    }
    Ok(format!(
        "{scanned} file(s) scanned: the four scratch types are declared in si-core::frame, constructed \
         at {constructed} site(s) inside si-walk, and named nowhere a container, schema, cache or \
         SIst/SIck write could keep them (G15(b))"
    ))
}

// ----------------------------------------------------------- B15 stage checks

/// 05 §20 / 12 §6.3 / 02 §12 (xiv): silt writes nothing at all, not even an ask.
/// The strong form is the manifest — `si-core` is the crate's only edge, so no
/// store, write gate, affect organ or labeler is a name it could call — and the
/// belt is the source scan plus the mutator partition: the only `&mut self` in
/// the crate belongs to `deposit` and `tick`, so the table has no third way to
/// change (OD-103, G16(a)(b), docs/23 §23.0's litmus).
fn check_silt_writes_nothing(ws: &Workspace) -> Result<String> {
    let manifest = ws.crate_manifest("si-silt")?;
    for key in dependency_keys(&manifest) {
        if key != "si-core" {
            bail!(
                "si-silt depends on `{key}`; silt's only edge is the kernel, because a table that can \
                 reach a writer is provenance in disguise (12 §6.3, G16(a)(b))"
            );
        }
    }
    const BANNED: [&str; 13] = [
        "si_store",
        "si_learn",
        "si_affect",
        "si_audit",
        "si_sleep",
        "si_front",
        "si_walk",
        "si_session",
        "append_row",
        "commit(",
        "supersede(",
        "salience",
        "&mut Lattice",
    ];
    let mut scanned = 0usize;
    let mut hits = Vec::new();
    let mut mutators: Vec<String> = Vec::new();
    for (relative, text) in ws.source_text()? {
        if !relative.starts_with("crates/si-silt/src/") {
            continue;
        }
        scanned += 1;
        // Every `fn NAME(` declaration, in line order; a `&mut self` belongs to
        // the last declaration at or above it whose brace span contains it.
        let mut declared: Vec<(String, usize)> = Vec::new();
        for (index, line) in text.lines().enumerate() {
            if let Some(at) = line.find("fn ") {
                let name: String = line[at + 3..]
                    .chars()
                    .take_while(|ch| ch.is_alphanumeric() || *ch == '_')
                    .collect();
                if !name.is_empty() {
                    declared.push((name, index + 1));
                }
            }
        }
        for (index, line) in text.lines().enumerate() {
            let code = code_of(line);
            for needle in BANNED {
                if code.contains(needle) {
                    hits.push(format!("{relative}:{}: `{needle}`", index + 1));
                }
            }
            if contains_word(code, "ask") || contains_word(code, "commit") {
                hits.push(format!("{relative}:{}: an ask/commit word", index + 1));
            }
            if !code.contains("&mut self") {
                continue;
            }
            let line_no = index + 1;
            let owner = declared
                .iter()
                .filter(|(_, start)| *start <= line_no)
                .last()
                .map(|(name, _)| name.clone());
            let inside = owner
                .as_deref()
                .and_then(|name| fn_span(&text, name).map(|(a, b)| (name, a, b)))
                .filter(|(_, a, b)| line_no >= *a && line_no <= *b);
            match inside {
                Some((name, _, _)) => mutators.push(name.to_string()),
                None => hits.push(format!("{relative}:{line_no}: `&mut self` outside a function")),
            }
        }
    }
    if scanned == 0 {
        bail!("si-silt has no source file to scan (12 §6.3)");
    }
    if !hits.is_empty() {
        bail!("si-silt reaches the write path: {}", hits.join("; "));
    }
    mutators.sort();
    mutators.dedup();
    if mutators != ["deposit", "tick"] {
        bail!(
            "si-silt's `&mut self` mutators are {mutators:?}; the table may change only through \
             `deposit` and `tick` (04 §30 AB.1)"
        );
    }
    Ok(format!(
        "{scanned} source file(s): the manifest's only edge is si-core, no store/write/affect/salience \
         symbol appears, and the table's sole mutators are deposit and tick (G16(a)(b), OD-103)"
    ))
}

/// 05 §20 / 02 §15 / OD-104 option (a): `silt::view` has exactly one consumer —
/// `walk::ignite`'s seed choice, reached from the idle tick — and the per-round
/// spread, the settle test and the conduction never read a level. This is the
/// collision made mechanical: were the spread primed by silt, a per-tick RAM
/// table would be a second modulation input out of `a` (R-29's shape).
fn check_silt_ignition_only(ws: &Workspace) -> Result<String> {
    let mut read_sites: Vec<(String, usize)> = Vec::new();
    for (relative, text) in ws.source_text()? {
        // The crate that *owns* the accessor is not a caller of it: `si-silt`'s
        // own unit tests read a table they built, and a loop crate still may not.
        if Workspace::is_auditor(&relative) || relative.starts_with("crates/si-silt/") {
            continue;
        }
        for (index, line) in text.lines().enumerate() {
            let code = code_of(line);
            if code.contains("silt.view(") || code.contains("silt::view(") {
                read_sites.push((relative.clone(), index + 1));
            }
        }
    }
    if read_sites.len() != 1 {
        bail!(
            "silt::view has {} read site(s) {read_sites:?}; the fifth ignition term is its only outlet \
             (04 §30 AB.2, OD-104(a))",
            read_sites.len()
        );
    }
    let (file, line_no) = &read_sites[0];
    if !file.ends_with("si-session/src/lib.rs") {
        bail!("silt::view is read at {file}:{line_no}; the loop is its only caller");
    }
    let session = ws.read(file)?;
    let in_idle = fn_span(&session, "idle_tick")
        .map(|(a, b)| *line_no >= a && *line_no <= b)
        .unwrap_or(false);
    if !in_idle {
        bail!("silt::view is read at {file}:{line_no}, outside `idle_tick`; the sampler is the only place a table may bias a choice");
    }
    let mut banned_mentions = Vec::new();
    for (relative, text) in ws.source_text()? {
        if !relative.starts_with("crates/si-walk/src/") {
            continue;
        }
        for name in ["spread", "settle_or_stop", "conduction"] {
            let Some((start, end)) = fn_span(&text, name) else {
                continue;
            };
            for (index, line) in text.lines().enumerate() {
                let line_no = index + 1;
                if line_no < start || line_no > end {
                    continue;
                }
                if line.to_ascii_lowercase().contains("silt") {
                    banned_mentions.push(format!("{relative}:{line_no}: `{name}` reads silt"));
                }
            }
        }
    }
    if !banned_mentions.is_empty() {
        bail!(
            "the per-round spread reads silt: {}; OD-104's option (a) keeps the seed choice the only \
             read (G16(e))",
            banned_mentions.join("; ")
        );
    }
    Ok(format!(
        "exactly one `silt::view` read site ({file}:{line_no}, inside idle_tick); spread, \
         settle_or_stop and conduction name no silt (OD-104(a), G16(e))"
    ))
}

// ----------------------------------------------------------- B16 stage checks

/// 05 §20 / 04 §31 AC.1 / OD-110: the event view and the `prov-link` index are
/// derived at frame load and stored nowhere. The strong half is structural — the
/// reserved shelf's schema version stays `0` — and the other half is a scan: no
/// source appends a row to shelf 11, and the two types carry no store handle.
fn check_prov_link_not_stored(ws: &Workspace) -> Result<String> {
    let entry = si_core::shelves::entry(si_core::addr::SHELF_EVENTS)
        .context("shelf 11 is missing from the registry (01 §2)")?;
    if entry.row_schema_version != 0
        || entry.mutability != si_core::shelves::Mutability::Reserved
    {
        bail!(
            "shelf 11 (`{}`) reads row_schema_version = {} and mutability {:?}; a derived view \
             leaves it reserved at 0 (04 §31 AC.1, OD-110)",
            entry.shelf_name,
            entry.row_schema_version,
            entry.mutability
        );
    }
    let reserved = si_core::shelves::SHELVES
        .iter()
        .filter(|entry| entry.mutability == si_core::shelves::Mutability::Reserved)
        .count();
    if reserved != si_core::shelves::RESERVED_SHELF_COUNT {
        bail!("{reserved} reserved shelves; 01 §2 declares {}", si_core::shelves::RESERVED_SHELF_COUNT);
    }

    // The two types are this crate's only declaration, and they carry no serde
    // derive: a serializable view is one container write away from being a row.
    // The scan is of code, not prose — a doc comment is allowed to name the
    // container it stays out of, which is why AC.1's own wording would otherwise
    // trip its own check.
    let episodic = code_text(ws.read("crates/si-core/src/episodic.rs")?);
    for needle in ["Serialize", "Deserialize", "append_row", "si_store", "offset"] {
        if episodic.contains(needle) {
            bail!(
                "si-core::episodic names `{needle}`; the view and the tug are derived values that \
                 hold no handle and cross no container (04 §31 AC.1)"
            );
        }
    }

    // The index builder: a read signature, no mutation of the lattice, and the
    // event id is the context row's own `y` (no new id space).
    let event_src = ws.read("crates/si-read/src/event.rs")?;
    let event_code = code_text(event_src.clone());
    for signature in [
        "pub fn event_view_index(lattice: &Lattice, anchor: i32, tug_q12: u16) -> Result<Vec<OriginTug>>",
        "pub fn event_views(lattice: &Lattice, anchor: i32) -> Result<Vec<EventView>>",
    ] {
        if !event_code.contains(signature) {
            bail!("the index builder's signature moved: `{signature}` is not in si-read::event");
        }
    }
    for needle in ["&mut Lattice", "&mut Store", "append_row", "commit(", "supersede("] {
        if event_code.contains(needle) {
            bail!(
                "si-read::event names `{needle}`; the index is one pass over committed rows at \
                 frame load and writes nothing (04 §31 AC.1, G17(b))"
            );
        }
    }

    // And the whole workspace: nothing appends to shelf 11, and no `prov-link`
    // row class exists to append. The two declaration files are the exception by
    // name because they are where the shelf id and its comment live.
    let mut hits = Vec::new();
    let mut scanned = 0usize;
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) {
            continue;
        }
        scanned += 1;
        for (index, line) in text.lines().enumerate() {
            let code = code_of(line);
            if code.contains("SHELF_EVENTS")
                && !matches!(
                    relative.as_str(),
                    "crates/si-core/src/addr.rs" | "crates/si-core/src/episodic.rs"
                )
            {
                hits.push(format!("{relative}:{}: `SHELF_EVENTS` outside its declaration", index + 1));
            }
            if code.contains("append_row") && code.to_ascii_lowercase().contains("event") {
                hits.push(format!("{relative}:{}: an `append_row` naming an event", index + 1));
            }
        }
    }
    if scanned == 0 {
        bail!("no source file to scan for a stored prov-link (04 §31 AC.1)");
    }
    if !hits.is_empty() {
        bail!(
            "a stored `prov-link` reached the workspace: {}; the index is rebuilt at frame load and \
             shelf 11 stays reserved-empty (OD-110, G17(a)(b))",
            hits.join("; ")
        );
    }
    Ok(format!(
        "shelf 11 `{}` reserved at row_schema_version 0; {scanned} file(s): no append to shelf 11, \
         no `prov-link` row class, and `event_view_index` is a `&Lattice` read (OD-110, G17(a)(b))",
        entry.shelf_name
    ))
}

/// 05 §20 / 04 §31 AC.3 / OD-112: the re-light's only mutation is frame
/// activation. The block is located inside `si-walk::walk` and scanned as a
/// block, so an unrelated read of `Arc.bake` elsewhere in the walk cannot mask a
/// write here; the stamp source value `2` is absent from every control crate.
fn check_relight_writes_nothing(ws: &Workspace) -> Result<String> {
    let source = ws.read("crates/si-walk/src/walk.rs")?;
    let (span_start, span_end) =
        fn_span(&source, "walk").context("si-walk has no `walk` function to scan")?;
    let lines: Vec<&str> = source.lines().collect();
    let start = lines
        .iter()
        .enumerate()
        .find(|(index, line)| *index + 1 >= span_start && line.contains("let mut relit: Vec<Relight>"))
        .map(|(index, _)| index)
        .context("the re-light block is not in `walk` (04 §31 AC.2)")?;
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(index, line)| *index + 1 <= span_end && **line == "    }")
        .map(|(index, _)| index)
        .context("the re-light block has no close before `walk` ends")?;
    const BANNED: [&str; 11] = [
        "Stamp",
        "stamp",
        "bake",
        "confidence",
        "append_row",
        "commit(",
        "supersede(",
        "si_store",
        "si_learn",
        "EdgeDraft",
        "store",
    ];
    let mut hits = Vec::new();
    for (offset, line) in lines[start..=end].iter().enumerate() {
        let code = code_of(line);
        for needle in BANNED {
            if code.contains(needle) {
                hits.push(format!(
                    "crates/si-walk/src/walk.rs:{}: `{needle}`",
                    start + offset + 1
                ));
            }
        }
    }
    if !hits.is_empty() {
        bail!(
            "the re-light path reaches the write side: {}; it may only add frame activation \
             (04 §31 AC.3, OD-112)",
            hits.join("; ")
        );
    }
    let block = lines[start..=end].join("\n");
    for required in ["h.entry(", "first_lit.entry(", "tugged.insert(", "relit.push("] {
        if !block.contains(required) {
            bail!("the re-light block no longer mutates `{required}`; the leg would grade an absence");
        }
    }
    // `Relight` is the log record: an event id, a point and a grid value. A
    // fourth field spelled like a stamp would make the record a write target.
    let relight_decl = source
        .split("pub struct Relight {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .context("si-walk declares no `Relight`")?;
    for needle in ["stamp", "bake", "confidence", "src"] {
        if relight_decl.contains(needle) {
            bail!("`Relight` carries `{needle}`; a re-light's record is an observation, not a stamp");
        }
    }
    // Neither crate may take a writer edge, and the stamp source `2` may not be
    // emitted anywhere a control crate could reach.
    for name in ["si-walk", "si-read"] {
        let manifest = ws.crate_manifest(name)?;
        for key in dependency_keys(&manifest) {
            if ["si-learn", "si-audit", "si-sleep", "si-session", "si-front"]
                .contains(&key.as_str())
            {
                bail!("{name} declares `{key}`; the re-light path may not reach a writer (12 §1.1)");
            }
        }
    }
    let mut src_hits = Vec::new();
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) || relative.contains("/tests/") {
            continue;
        }
        for (index, line) in text.lines().enumerate() {
            let code = code_of(line);
            if code.contains("idle-re-light") || code.contains("IdleRelight") {
                src_hits.push(format!("{relative}:{}: an idle-re-light stamp source", index + 1));
            }
            if code.contains("src: 2") || code.contains("src:2") || code.contains("src = 2") {
                src_hits.push(format!("{relative}:{}: the OPEN-24 stamp source `2`", index + 1));
            }
        }
    }
    if !src_hits.is_empty() {
        bail!(
            "a re-light stamp source is emitted: {}; OD-112 rides OPEN-24, which is off (G17(d))",
            src_hits.join("; ")
        );
    }
    Ok(format!(
        "the re-light block (si-walk/src/walk.rs:{}-{}) writes frame activation only — no Stamp, no \
         bake, no confidence, no edge, no question — and no control crate emits `src = 2` (OD-112 \
         rides OPEN-24 off; G17(c)(d))",
        start + 1,
        end + 1
    ))
}

// ----------------------------------------------------------- B17 stage checks

/// 12 §6.2 / OD-120: `si-skills` is a leaf that cannot commit, and the only hand
/// that writes a skill row is `si-learn`'s ordinary commit reached from step 11 of
/// the loop. The third claim keeps the usage counters honest: a number the
/// detector could assign would be a byte, and a byte would make "recomputed, never
/// stored" a sentence instead of a fact.
fn check_skills_no_commit(ws: &Workspace) -> Result<String> {
    const DIR: &str = "crates/si-skills/src/";
    const BANNED: [&str; 12] = [
        "si_learn",
        "si_affect",
        "si_audit",
        "si_sleep",
        "si_front",
        "si_session",
        "append_row",
        "commit(",
        "supersede(",
        ".age(",
        "bake_q12(",
        "salience",
    ];
    let mut edges = Vec::new();
    for key in dependency_keys(&ws.crate_manifest("si-skills")?) {
        edges.push(key);
    }
    let mut hits: Vec<String> = edges
        .iter()
        .filter(|key| !["si-core", "si-store", "anyhow"].contains(&key.as_str()))
        .map(|key| format!("manifest declares `{key}`"))
        .collect();
    let mut scanned = 0usize;
    for (relative, text) in ws.source_text()? {
        if !relative.starts_with(DIR) {
            continue;
        }
        scanned += 1;
        for (index, line) in text.lines().enumerate() {
            let code = code_of(line);
            for needle in BANNED {
                if code.contains(needle) {
                    hits.push(format!("{relative}:{}: `{needle}`", index + 1));
                }
            }
            // `&mut self` is the table; a `&mut` on the store would be the write
            // edge this crate was deliberately not given (12 §1.1).
            for (offset, _) in code.match_indices("&mut ") {
                let target: String = code[offset + "&mut ".len()..]
                    .chars()
                    .take_while(|ch| ch.is_alphanumeric() || *ch == '_' || *ch == ':')
                    .collect();
                if ["Lattice", "Store", "store", "lattice", "si_store"]
                    .iter()
                    .any(|name| target.starts_with(name))
                {
                    hits.push(format!("{relative}:{}: `&mut {target}`", index + 1));
                }
            }
        }
    }
    if scanned == 0 {
        bail!("si-skills has no source file to scan (12 §6.2)");
    }

    // The row's writer, named by the shelf it writes. Shelf 9 is a read for the
    // preference and an exclusion for the walk and the retrieval; the only files
    // that may name it *and* hold a row-writing verb are the loop's own commit
    // path and si-learn's generic append.
    const WRITE_VERBS: [&str; 3] = ["append_row(", "commit(", "supersede("];
    const WRITERS: [&str; 2] = ["crates/si-session/src/lib.rs", "crates/si-learn/src/gate.rs"];
    let mut shelf_files = 0usize;
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) || relative.contains("/tests/") {
            continue;
        }
        if !text.contains("SHELF_SKILLS") {
            continue;
        }
        shelf_files += 1;
        let writes = text
            .lines()
            .any(|line| WRITE_VERBS.iter().any(|verb| code_of(line).contains(verb)));
        if writes && !WRITERS.contains(&relative.as_str()) {
            hits.push(format!("{relative} names shelf 9 and holds a row-writing verb"));
        }
    }

    // OD-120: a usage counter is derived per live row from the rows plus the log.
    // It appears as a JSON key in the log block and nowhere else — no schema
    // field, no struct member, no assignment in the detector.
    let mut counter_files = 0usize;
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) || !text.contains("times_consulted") {
            continue;
        }
        counter_files += 1;
        if relative != "crates/si-session/src/lib.rs" {
            hits.push(format!("{relative}: names `times_consulted` (OD-120: derived, never stored)"));
            continue;
        }
        for (index, line) in text.lines().enumerate() {
            let code = code_of(line);
            if code.contains("times_consulted") && !code.contains("\"times_consulted\":") {
                hits.push(format!(
                    "{relative}:{}: `times_consulted` outside the derived json block",
                    index + 1
                ));
            }
        }
    }

    if !hits.is_empty() {
        bail!(
            "the skill detector reaches the write path or stores a counter: {}; si-skills reads and \
             nominates only, and the mint is step 11's commit (12 §6.2, OD-120)",
            hits.join("; ")
        );
    }
    Ok(format!(
        "{scanned} si-skills source file(s), edges [{}] over {shelf_files} shelf-9 file(s): no \
         commit symbol, no `&mut` store, no writer outside si-session/si-learn, and \
         times_consulted is derived in {counter_files} file(s) (G18(a)(b)(f), OD-120)",
        edges.join(", "),
    ))
}

/// 12 §2.4 / OD-128: `origin = 2` is the lane, and the lane is the owner's. The
/// nomination ask is an ordinary `questions` row, so the assertion cannot be
/// "nothing else reads shelf 6" — it is that the only reader which accepts an
/// `origin = 2` row does so behind `Source::Owner`, and that the ordinary queue
/// actively excludes it. G19(f) cites the same fact.
fn check_nomination_owner_lane(ws: &Workspace) -> Result<String> {
    let gate = ws.read("crates/si-learn/src/gate.rs")?;
    let gate_lines: Vec<&str> = gate.lines().collect();
    let (open_start, open_end) =
        fn_span(&gate, "latest_open_ask").context("si-learn has no `latest_open_ask`")?;
    let (lane_start, lane_end) = fn_span(&gate, "latest_open_nomination")
        .context("si-learn has no `latest_open_nomination` (12 §2.4)")?;
    if !gate_lines[open_start - 1..open_end]
        .join("\n")
        .contains("row.origin() != ASK_ORIGIN_NOMINATION")
    {
        bail!(
            "`latest_open_ask` no longer filters the nomination origin: an ordinary answer would \
             close a detector's own ask (OD-128)"
        );
    }
    let lane = gate_lines[lane_start - 1..lane_end].join("\n");
    if !lane.contains("row.origin() == ASK_ORIGIN_NOMINATION") || !lane.contains("gap_kind()") {
        bail!(
            "`latest_open_nomination` no longer selects `origin = 2` by kind: the lane would be a \
             second ordinary queue (12 §2.4)"
        );
    }

    // Every call site of the lane lookup, and the function that wraps it.
    let mut sites = 0usize;
    let mut guards = 0usize;
    let mut routed = Vec::new();
    let mut files = 0usize;
    let mut matches = 0usize;
    for (relative, text) in ws.source_text()? {
        files += 1;
        if Workspace::is_auditor(&relative) || relative.contains("/tests/") {
            continue;
        }
        if relative == "crates/si-learn/src/gate.rs" {
            continue;
        }
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            if !code_of(line).contains("latest_open_nomination(") {
                continue;
            }
            matches += 1;
            let Some(back) = lines[..index].iter().rev().position(|earlier| {
                let earlier = earlier.trim_start();
                earlier.starts_with("fn ") || earlier.starts_with("pub fn ")
            }) else {
                routed.push(format!("{relative}:{}: no enclosing fn", index + 1));
                continue;
            };
            let header = lines[index - back - 1];
            let name = header
                .trim_start()
                .trim_start_matches("pub ")
                .trim_start_matches("fn ")
                .split('(')
                .next()
                .unwrap_or_default();
            let Some((start, end)) = fn_span(&text, name) else {
                routed.push(format!("{relative}:{}: `{name}` has no span", index + 1));
                continue;
            };
            let body = lines[start - 1..end].join("\n");
            sites += 1;
            if !body.contains("Source::Owner") {
                routed.push(format!(
                    "{relative}:{}: `{name}` reads the lane with no owner guard",
                    index + 1
                ));
            }
            if body.contains("source == Source::Owner") {
                guards += 1;
            }
        }
    }
    if sites == 0 {
        bail!(
            "no crate calls `latest_open_nomination`: the nomination lane has no reader (OD-128) \
             [{matches} needle match(es) across {files} file(s), unrouted: {routed:?}]"
        );
    }
    if !routed.is_empty() {
        bail!(
            "an `origin = 2` row is read outside the owner lane: {}; the lane byte is the only \
             routing field a nomination has (12 §2.4)",
            routed.join("; ")
        );
    }
    if guards == 0 {
        bail!(
            "no lane reader is guarded on `source == Source::Owner`: a non-owner line could mint \
             a skill (G18(d))"
        );
    }

    // The lane byte may not name a *sink* anywhere else: a reader, a render or a
    // supporter route that could answer a nomination is a second lane. The two
    // detectors are allowed to *stamp* it — OD-119 gives them no edge to each
    // other, so one value is two declarations — and si-learn filters on it.
    const LANE_READERS: [&str; 4] = [
        "crates/si-learn/src/",
        "crates/si-session/src/",
        "crates/si-crystallize/src/",
        "crates/si-skills/src/",
    ];
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) || relative.contains("/tests/") {
            continue;
        }
        if text.contains("ASK_ORIGIN_NOMINATION")
            && !LANE_READERS.iter().any(|dir| relative.starts_with(dir))
        {
            bail!(
                "{relative} names the nomination origin: the lane is decided at the reader, and \
                 only the reader and the stamping detector are allowed to know the byte (OD-128)"
            );
        }
    }

    Ok(format!(
        "the lane has {sites} call site(s), every one behind `Source::Owner` ({guards} guarded on \
         the arriving line's source); `latest_open_ask` excludes `origin = 2`, and the origin byte \
         is named only in si-learn, si-skills and si-session (G18(d), G19(f), OD-128)",
    ))
}

// ---------------------------------------------------------------------------
// B18 — the world (05 §20, 12 §5.2.2, 04 §33 AE.1/AE.4, docs/12 rule (ix)).
// ---------------------------------------------------------------------------

/// The files whose every byte is a **stored** byte: a row's envelope or body, a
/// hash item, a container, an event-log record or a snapshot. A speaker class in
/// any of them is a class in a file, which is docs/12 rule (ix)'s one forbidden
/// shape, and an identity in any of them (beside the framed sense body, whose own
/// home is `eventlog.rs`) is OD-123's second forbidden shape.
const STORAGE_HOMES: [&str; 7] = [
    "crates/si-core/src/record.rs",
    "crates/si-core/src/hash.rs",
    "crates/si-core/src/shelves.rs",
    "crates/si-core/src/eventlog.rs",
    "crates/si-store/src/store.rs",
    "crates/si-store/src/snapshot.rs",
    "crates/si-store/src/lane.rs",
];

/// The licensed homes of the code token `speaker_identity`: the framed `SIro`
/// sense body itself, the derived event view **12 §2.6's own table** gives the
/// field to (`{…, t: i32, speaker_identity: u64}`, "assembled at frame load,
/// never stored", logged to `mindlog.json event_views{}`), that view's builder
/// and its one writer in the session, and the harness's `transcript.txt` plus
/// `G19/server_transcript.txt`, which is the artifact 05 §20 names.
///
/// B11 adds the viewer's two files: 08 §1 gives process B exactly one write, a kind-0
/// sense frame, and a sense frame is the one body whose first field *is* the identity
/// (01 §9). `si-vulkan/src/frames.rs` decodes that field and `surface.rs` mints the
/// owner's byte for it (00 §7's privilege table, `SOURCE_OWNER`), which is the framed
/// sense body and the transcript this list already licenses — not a row, an envelope or
/// a hash item.
const IDENTITY_HOMES: [&str; 8] = [
    "crates/si-core/src/eventlog.rs",
    "crates/si-core/src/episodic.rs",
    "crates/si-read/src/event.rs",
    "crates/si-session/src/lib.rs",
    "crates/si-vulkan/src/frames.rs",
    "crates/si-vulkan/src/surface.rs",
    "crates/si-cli/src/harness/runner.rs",
    "crates/si-cli/src/harness/gates.rs",
];

/// 05 §20 / 12 §5.2.2 / **G19(b)**: the class is a return value and nothing else.
///
/// The scan has four halves. (1) A field *typed* `SpeakerClass` exists exactly
/// once in the build — `WriteContext.class`, which is 04 §12 step 0b's
/// **argument**, RAM the gate never keeps. (2) No serializable type carries one,
/// because a `Serialize` derive is one container write away from a stored class.
/// (3) The session's `World` — the only place a class could be cached — holds the
/// identity and the grant view and no class, and `world::class` has one caller.
/// (4) `speaker_identity` rides the sense stream and the transcript and appears
/// in no row, envelope, hash item or snapshot, while the stream container's
/// version really is the bumped one and the reader really does refuse.
///
/// OD-123's shipped half is that a grant row is **session RAM installed by the
/// scenario's own owner line**: it is in no shelf and no container, because 12 §11
/// adds no shelf and no row kind and 11 §B18 adds no `/trust` parser. The
/// `class:`-typed field of the auditor's `StepRecord` is an `&'static str` printed
/// beside the step that derived it — the observation **G19(b)** demands — not a
/// class any reader can read back.
fn check_speaker_class_derived(ws: &Workspace) -> Result<String> {
    // (1) A field typed SpeakerClass, and the one place the law licenses it.
    let mut typed_fields = Vec::new();
    let mut serializable = Vec::new();
    let mut class_callers = Vec::new();
    let mut identity_lines = Vec::new();
    let mut scanned = 0usize;
    for (relative, text) in ws.source_text()? {
        if relative.contains("/tests/") || relative == "crates/si-cli/src/harness/checks.rs" {
            // This file names every token it forbids, in the messages that report the
            // ban — the exemption `check_prov_link_not_stored` gives the declaration
            // files it quotes, and the one `check_windows_suppress_only` gives itself.
            continue;
        }
        scanned += 1;
        let mut current_type = String::new();
        let mut current_kind = "";
        let mut block_end_line = 0usize;
        let mut pending_derive = false;
        for (number, raw) in text.lines().enumerate() {
            let line = code_of(raw).trim();
            let at = number + 1;
            if at > block_end_line {
                current_kind = "";
            }
            if let Some(rest) = line.strip_prefix("#[derive(") {
                pending_derive = rest.contains("Serialize") || rest.contains("Deserialize");
            } else if !line.starts_with("#") {
                if let Some((kind, name)) = decl_of(line) {
                    current_kind = kind;
                    current_type = name;
                    block_end_line = if line.contains('{') {
                        block_end(&text, at)
                    } else {
                        at
                    };
                    if pending_derive
                        && code_text(
                            text.lines()
                                .skip(number)
                                .take(block_end_line - number)
                                .collect::<Vec<_>>()
                                .join("\n"),
                        )
                        .contains("SpeakerClass")
                    {
                        serializable.push(format!(
                            "{relative}:{at}: `{current_type}` derives a serializer",
                        ));
                    }
                }
                pending_derive = false;
            }
            if line.contains("SpeakerClass") {
                // AE.1's ban is on the class as *stored state*: a struct field. A
                // function's parameter, an enum variant's payload and the type's own
                // declaration are the return-value half of the licence, and the two
                // arms below catch the places where such a value could persist.
                if current_kind == "struct"
                    && is_field_line(line)
                    && !matches!(
                        (relative.as_str(), current_type.as_str()),
                        ("crates/si-learn/src/gate.rs", "WriteContext")
                    )
                {
                    typed_fields.push(format!(
                        "{relative}:{at}: field of `{current_type}`",
                    ));
                }
                if STORAGE_HOMES.contains(&relative.as_str()) {
                    typed_fields.push(format!("{relative}:{at}: a class in a stored byte"));
                }
            }
            if code_of(raw).contains("world::class(") {
                class_callers.push(format!("{relative}:{at}"));
            }
            if code_of(raw).contains("speaker_identity") {
                identity_lines.push((relative.clone(), at));
            }
        }
    }
    if scanned == 0 {
        bail!("no source file to scan for a stored speaker class (04 §33 AE.1)");
    }
    if !typed_fields.is_empty() {
        bail!(
            "a speaker class is stored where the law says it is computed: {}; AE.1 keeps it a \
             return value and docs/12's rule (ix) makes a class in a row R-28 in the social \
             channel (**G19(b)**, OD-123/124)",
            typed_fields.join("; ")
        );
    }
    if !serializable.is_empty() {
        bail!(
            "{}; a serializable type naming a class is one container write away from a stored \
             class (05 §20, OD-123)",
            serializable.join("; ")
        );
    }

    // (3) The world holds the identity and the grants, never the class, and the
    // class function has one caller.
    let session = ws.read("crates/si-session/src/lib.rs")?;
    let world_block = type_block(&session, "pub struct World").context(
        "`pub struct World` is gone from si-session: 12 §2.6 names the session RAM this stage \
         declares (OD-123)",
    )?;
    for line in world_block.lines() {
        if code_of(line).contains("class") {
            bail!(
                "the session's World carries a class member (`{}`): AE.1 derives the class at every \
                 use, so caching it here is the stored class **G19(b)** forbids (OD-123)",
                code_of(line).trim()
            );
        }
    }
    for required in ["line_identity", "grants", "window"] {
        if !world_block.contains(required) {
            bail!(
                "the session's World no longer carries `{required}`: the identity, the grant view \
                 and the declared window are the three world values AE.1 keeps as RAM (04 §33)"
            );
        }
    }
    if !class_callers.is_empty()
        && !class_callers.iter().all(|site| site.starts_with("crates/si-session/src/lib.rs"))
    {
        bail!(
            "`world::class` is called outside `Session::class_of` ({class_callers:?}): the class has \
             one deriving site or it has a cache somewhere (04 §33 AE.1)"
        );
    }

    // (4) The identity's licensed homes, and the version law that reads it.
    let mut foreign = Vec::new();
    for (relative, number) in &identity_lines {
        if !IDENTITY_HOMES.contains(&relative.as_str()) {
            foreign.push(format!("{relative}:{number}"));
        }
    }
    if !foreign.is_empty() {
        bail!(
            "`speaker_identity` reached {foreign:?}: it rides the framed `SIro` sense body and the \
             transcript, and no envelope, row or other log carries it (01 §9, 12 §2.6, OD-123's \
             shipped half, **G19(b)**)"
        );
    }
    if identity_lines.is_empty() {
        bail!("no code line names `speaker_identity`: B18's transport pair was never written (01 §9)");
    }
    let eventlog = code_text(ws.read("crates/si-core/src/eventlog.rs")?);
    if !eventlog.contains("extend_from_slice(&speaker_identity.to_le_bytes())") {
        bail!(
            "the sense body no longer encodes `speaker_identity u64le` first: the frame's identity \
             has no byte (01 §9)"
        );
    }
    let container = code_text(ws.read("crates/si-core/src/container.rs")?);
    if !container.contains("STREAM_CONTAINER_VERSION: u32 = 2")
        || !container.contains("Format::StreamRecord => STREAM_CONTAINER_VERSION")
    {
        bail!(
            "the `SIro` stream container is not at version 2: a pre-B18 inbox would re-parse as a \
             B18 one, and 11 §B18 DoD 2's named refusal has nothing to refuse (01 §9)"
        );
    }
    if !code_text(session.clone()).contains("InboxRefusal::VersionMismatch") {
        bail!("the inbox reader no longer refuses a version mismatch as a branch (00 §5, 01 §9)");
    }

    Ok(format!(
        "{scanned} file(s) scanned: the only field typed `SpeakerClass` is `WriteContext.class` \
         (04 §12 step 0b's argument), no serializable type names a class, `World` holds \
         `line_identity` + `grants` and no class, `world::class` is called only from \
         `Session::class_of`, and `speaker_identity` appears in {number} code line(s), all inside \
         the sense body (`{}`), the 12 §2.6 event view, the transcript and B11's viewer — which \
         decodes the framed identity and mints the owner's byte for its one write — never in a \
         row, envelope or hash item (**G19(b)**, OD-123/124)",
        IDENTITY_HOMES[0],
        number = identity_lines.len(),
    ))
}

/// 05 §20 / 12 §5.2.2 / **G19(g)**: a window can only say "not now".
///
/// docs/26 §26.4's window is an interval of the **logical** axis in which a quiet
/// thread *may* ignite, and OD-126's law is the direction: opening one is never a
/// cause of a thought. Mechanically that is three facts. (1) The value is read in
/// exactly one place — `Session::window_denies`, whose body is a negation — and
/// that helper has exactly one caller, inside `idle_tick`. (2) The conjunct it
/// enters is a **deny** (`!denied`), so the ignition predicate holds only when the
/// window said nothing against it, and no line that names a window also names
/// `ignite`, the sleep's quiet counter, the clock's advance or a frame read.
/// (3) The type exists in `si-core::world` and in `si-session` (plus the harness's
/// scenario grammar); `si-walk`, `si-sleep`, `si-affect` and the store never see
/// it, so there is no path to run from a window to an ignition even in principle.
fn check_windows_suppress_only(ws: &Workspace) -> Result<String> {
    let window_files = [
        "crates/si-core/src/world.rs",
        "crates/si-session/src/lib.rs",
        "crates/si-cli/src/harness/scenario.rs",
        "crates/si-cli/src/harness/runner.rs",
        "crates/si-cli/src/harness/gates.rs",
    ];
    // The four destinations OD-126 names. A statement in an **organ** that reaches
    // one of them is not a suppression, it is a cause. The harness is excluded from
    // this scan because it drives both arms of every pair the gate grades and is not
    // a path a window value can travel; its own reach is the scan above.
    const CAUSAL: [&str; 6] = [
        "si_walk::ignite",
        "quiet_run",
        "due(",
        "advance(",
        "tick_frame",
        "read_sense_inbox",
    ];
    const ORGANS: [&str; 2] = ["crates/si-core/src/world.rs", "crates/si-session/src/lib.rs"];
    // The interval as the viewer could reach it: its module path, its type, its one
    // predicate and its setter.
    const INTERVAL_SPELLINGS: [&str; 5] = [
        "si_core::world",
        "world::Window",
        "window_denies",
        "suppressed_by_window",
        "set_window",
    ];
    let mut reads: Vec<String> = Vec::new();
    let mut causal = Vec::new();
    let mut stray = Vec::new();
    let mut scanned = 0usize;
    for (relative, text) in ws.source_text()? {
        if relative.contains("/tests/") || relative == "crates/si-cli/src/harness/checks.rs" {
            // The check's own file names the tokens it forbids, in the messages that
            // report the ban — the same exemption `check_prov_link_not_stored` gives
            // the two declaration files it quotes.
            continue;
        }
        scanned += 1;
        // B11's viewer owns an OS window; OD-126's `Window` is a session-RAM interval.
        // The two share a spelling and nothing else — `WindowEvent`, `WindowId`,
        // `Outcome::Windowed` are winit's and the viewer's own. So si-vulkan is read for
        // the *absence* of the interval rather than as a stray home for it: no line of the
        // viewer may name the interval's declaration site or its one predicate, which is a
        // stronger statement than a name match, because the viewer does link si-core.
        let viewer = relative.starts_with("crates/si-vulkan/");
        for (number, raw) in text.lines().enumerate() {
            let line = code_of(raw);
            if viewer {
                for forbidden in INTERVAL_SPELLINGS {
                    if line.contains(forbidden) {
                        stray.push(format!("{relative}:{}: `{forbidden}`", number + 1));
                    }
                }
                continue;
            }
            let names_window = line.contains("window_denies")
                || line.contains("world.window")
                || line.contains("set_window")
                || line.contains("Window")
                || line.contains("suppressed_by_window");
            if !names_window {
                continue;
            }
            if !window_files.contains(&relative.as_str()) {
                stray.push(format!("{relative}:{}", number + 1));
                continue;
            }
            if ORGANS.contains(&relative.as_str()) {
                for needle in CAUSAL {
                    if line.contains(needle) {
                        causal.push(format!("{relative}:{}: `{needle}`", number + 1));
                    }
                }
            }
            if line.contains("self.world.window") && !line.contains("self.world.window =") {
                reads.push(format!("{relative}:{}", number + 1));
            }
        }
    }
    if !stray.is_empty() {
        bail!(
            "a window value reached {stray:?}: OD-126 gives it one consultation, on the idle \
             gate's own predicate, and no other organ is allowed to see it — and the viewer, whose \
             `Window` is winit's and not this one, may not name the interval at all (**G19(g)**)",
        );
    }
    if !causal.is_empty() {
        bail!(
            "a window shares a statement with a cause ({causal:?}): a declared window can deny an \
             idle tick and cannot start one, advance the clock, feed the sleep trigger or read a \
             frame (04 §19 S.1, §33 AE.4, OD-126)",
        );
    }

    // The one consultation, the one caller, the deny position.
    let session = ws.read("crates/si-session/src/lib.rs")?;
    let (deny_start, deny_end) =
        fn_span(&session, "window_denies").context("si-session has no `window_denies`")?;
    let session_lines: Vec<&str> = session.lines().collect();
    if reads.len() != 1 || !reads[0].contains("si-session/src/lib.rs") {
        bail!(
            "the window value is read outside `window_denies` ({reads:?}): AE.4 gives it exactly \
             one consultation, so a second reader is a second predicate (OD-126)",
        );
    }
    let read_line: usize = reads[0]
        .rsplit(':')
        .next()
        .and_then(|text| text.parse().ok())
        .unwrap_or(0);
    if read_line < deny_start || read_line > deny_end {
        bail!(
            "the window's one read is at line {read_line}, outside `window_denies`'s span \
             ({deny_start}..{deny_end}): the suppression has one home (04 §33 AE.4)",
        );
    }
    let mut callers = 0usize;
    for (number, line) in session_lines.iter().enumerate() {
        let line = code_of(line);
        if number + 1 >= deny_start && number + 1 <= deny_end && line.contains("fn window_denies(")
        {
            continue;
        }
        if line.contains("window_denies(") {
            callers += 1;
        }
    }
    if callers != 1 {
        bail!(
            "`window_denies` has {callers} caller(s): the idle gate's ignition conjunct is the only \
             one (04 §33 AE.4)",
        );
    }
    let (idle_start, idle_end) =
        fn_span(&session, "idle_tick").context("si-session has no `idle_tick`")?;
    let idle = session_lines[idle_start - 1..idle_end].join("\n");
    if !idle.contains("window_denies(") {
        bail!("`idle_tick` no longer consults the window: the suppression leg has no subject");
    }
    for line in session_lines[idle_start - 1..idle_end].iter().filter(|line| {
        let line = code_of(line);
        line.contains("if ") && line.contains("denied")
    }) {
        if !line.contains("!denied") {
            bail!(
                "`{}` reads the window as a cause: the ignition conjunct must deny on it (`!denied`), \
                 never require it (OD-126, **G19(g)**)",
                code_of(line).trim()
            );
        }
    }
    // The quiet-and-empty-agenda law the window may only subtract from: the sleep
    // trigger's counter is `agenda_clear` alone, so a busy server cannot buy
    // thinking time and an open window grants none either.
    if !idle.contains("if agenda_clear {") {
        bail!(
            "the quiet-run counter no longer hangs on `agenda_clear` alone: OD-126's window is a \
             third *deny* conjunct on ignition and nothing else (04 §19 S.1, §33 AE.4)"
        );
    }

    // The type itself: one predicate, and it is the negation of "inside".
    let world = code_text(ws.read("crates/si-core/src/world.rs")?);
    if !world.contains("pub fn denies(&self, now: i32) -> bool") {
        bail!("`Window::denies` is gone: the window's whole interface is a denial (docs/26 §26.4)");
    }
    for forbidden in ["pub fn allows", "pub fn causes", "pub fn opens"] {
        if world.contains(forbidden) {
            bail!(
                "si-core::world declares `{forbidden}`: a window that can grant an ignition is not \
                 the suppress-only interval OD-126 rules (04 §19 S.1)"
            );
        }
    }
    if world.contains("Serialize") || world.contains("Deserialize") {
        bail!("si-core::world derives a serializer: the declared window is session RAM, never a byte");
    }

    Ok(format!(
        "{scanned} file(s) scanned: `Window` is named only in si-core::world, si-session and the \
         harness's scenario grammar — si-vulkan's `Window`/`WindowEvent` are winit's OS window and \
         name none of {INTERVAL_SPELLINGS:?} — `self.world.window` is read once (`window_denies`, whose one \
         caller is `idle_tick`), the ignition conjunct is `!denied`, the sleep counter hangs on \
         `agenda_clear` alone, and no window statement reaches `ignite`, `quiet_run`, the clock or \
         a frame read (**G19(g)**, OD-126)",
    ))
}

/// 05 §20 / 08 §1's OBSERVER-ONLY law, read as the row says it: `si-vulkan`'s
/// imports and open calls.
///
/// The row's read half lists five lane files the viewer may read tail-only. B11
/// narrowed that: the launcher decodes the lanes and hands the values to the panel
/// ([`si_vulkan::panels::Artifacts`], built in `si-cli view`), so process B reads no
/// artifact at all. This leg proves the **narrower** contract — one open call, for
/// the one writable file, and no reader anywhere in `src/` — and the detail names
/// the supersession instead of pretending the five reads happen here
/// (**OD-148**).
fn check_gui_observer_only(ws: &Workspace) -> Result<String> {
    let manifest = ws.crate_manifest("si-vulkan")?;
    const ALLOWED_EDGES: [&str; 7] = [
        "si-core",
        "anyhow",
        "serde_json",
        "bytemuck",
        "ash",
        "winit",
        "raw-window-handle",
    ];
    for key in dependency_keys(&manifest) {
        if !ALLOWED_EDGES.contains(&key.as_str()) {
            bail!(
                "si-vulkan depends on `{key}`: 00 §2 gives the viewer core ONLY, and a wider edge is \
                 how a display crate becomes a decider (08 §1)"
            );
        }
    }

    const WRITE_MARKERS: [&str; 5] = [
        "File::create",
        "OpenOptions",
        "File::options",
        "fs::write",
        ".write(true)",
    ];
    // The inbox is appended, never replaced: a rename, copy or delete over a run
    // directory *is* the second input path 08 §1 says does not exist.
    const NEVER_MARKERS: [&str; 6] = [
        "fs::remove_file",
        "fs::rename",
        "create_new",
        "fs::remove_dir",
        "fs::copy",
        "fs::hard_link",
    ];
    // The five tail-only reads as calls. Their absence is the narrowing above.
    const READ_MARKERS: [&str; 4] = ["read_to_string", "fs::read", "File::open", "fs::read_dir"];

    let mut opens = Vec::new();
    let mut reads = Vec::new();
    for (relative, text) in ws.source_text()? {
        if !relative.starts_with("crates/si-vulkan/src/") {
            continue;
        }
        let lines: Vec<&str> = text.lines().collect();
        for (index, raw) in lines.iter().enumerate() {
            let line = code_part(raw);
            for forbidden in NEVER_MARKERS {
                if line.contains(forbidden) {
                    bail!(
                        "{relative}:{} calls `{forbidden}`: process B deletes, copies and renames \
                         nothing (08 §1)",
                        index + 1
                    );
                }
            }
            for marker in READ_MARKERS {
                if line.contains(marker) {
                    reads.push(format!("{relative}:{} `{marker}`", index + 1));
                }
            }
            if WRITE_MARKERS.iter().any(|marker| line.contains(marker)) {
                let window = lines[index.saturating_sub(4)..(index + 5).min(lines.len())].join(" ");
                if !window.contains("WRITABLE_FILE") && !window.contains("sense_inbox") {
                    bail!(
                        "{relative}:{} opens a file this crate does not own: {}",
                        index + 1,
                        line.trim()
                    );
                }
                opens.push(format!("{relative}:{}", index + 1));
            }
        }
    }
    if !reads.is_empty() {
        bail!(
            "si-vulkan reads an artifact ({}) — under the handover the launcher decodes the lanes and \
             hands values over, so a second reader here is a second, unaudited copy of the schema",
            reads.join(", ")
        );
    }
    if opens.len() != 1 {
        bail!(
            "08 §1 gives process B exactly one writable file; si-vulkan opens {} ({})",
            opens.len(),
            opens.join(", ")
        );
    }
    Ok(format!(
        "si-vulkan's only SI edge is si-core; src/ opens exactly 1 file, {}, whose window names \
         `WRITABLE_FILE` = `sense_inbox.siro`, and reads 0 artifacts — the row's five tail-only lane \
         reads live with the launcher instead, so process B holds no path to a lattice (08 §1, \
         OD-148)",
        opens[0]
    ))
}

/// 05 §20 / 08 §9 + I8: the panel's write set — no widget, key binding or drag may
/// reach a state byte, a door or a sleep/seed trigger, and the sleep, seed and
/// fingerprint readouts are **labels over artifacts**.
///
/// Three mechanical facts, each one a scan of `si-vulkan/src` over code text only:
/// no state symbol is nameable from here, no setter-shaped function exists, and the
/// inbound event set is six named keys plus typed characters with no pointer or
/// scroll channel at all. A drag would be a second input path, which 08 §1 refuses
/// before §9 ever lists a widget.
fn check_no_viewer_state_write(ws: &Workspace) -> Result<String> {
    // A state byte, a door or a sleep/seed trigger is only reachable through a
    // symbol; the viewer names none of these.
    const STATE_NEEDLES: [&str; 16] = [
        "si_store",
        "si_learn",
        "si_session",
        "si_sleep",
        "si_affect",
        "si_walk",
        "si_read",
        "si_audit",
        "si_supporter",
        "Lattice",
        "append_row",
        ".commit(",
        "advance(",
        "ignite(",
        "set_a",
        "set_door",
    ];
    // §9's refused widgets are also verbs: a slider or a "sleep now" button is a
    // setter, and the crate declares none.
    const SETTER_NEEDLES: [&str; 5] = ["fn set_", "fn adjust", "fn trigger", "fn force", "fn apply"];
    // A drag, a scroll or a hover is an inbound event the sense channel does not
    // carry. 08 §9's refused widgets all need one.
    const POINTER_NEEDLES: [&str; 6] = [
        "PointerEvent",
        "CursorMoved",
        "MouseWheel",
        "MouseButton",
        "Touch",
        "CursorIcon",
    ];
    // The readouts are forwarded, never recomputed: no hashing or temperament
    // derivation primitive is named here, so a panel cannot mint the number it prints.
    const DERIVE_NEEDLES: [&str; 5] = ["sha256", "Md5", "md5", "hash::", "temperament::"];
    // The whole keyboard contract: Enter submits the owner's line to the inbox,
    // Backspace edits it, the arrows pan the camera. `Key::Character` is the line.
    const ALLOWED_KEYS: [&str; 6] = [
        "Enter",
        "Backspace",
        "ArrowLeft",
        "ArrowRight",
        "ArrowUp",
        "ArrowDown",
    ];

    let mut state_hits = Vec::new();
    let mut setter_hits = Vec::new();
    let mut pointer_hits = Vec::new();
    let mut derive_hits = Vec::new();
    let mut bound_keys = Vec::new();
    let mut scanned = 0usize;
    for (relative, text) in ws.source_text()? {
        if !relative.starts_with("crates/si-vulkan/src/") {
            continue;
        }
        scanned += 1;
        for (index, raw) in text.lines().enumerate() {
            let line = code_part(raw);
            for needle in STATE_NEEDLES {
                if line.contains(needle) {
                    state_hits.push(format!("{relative}:{} `{needle}`", index + 1));
                }
            }
            for needle in SETTER_NEEDLES {
                if line.contains(needle) {
                    setter_hits.push(format!("{relative}:{} `{needle}`", index + 1));
                }
            }
            for needle in POINTER_NEEDLES {
                if line.contains(needle) {
                    pointer_hits.push(format!("{relative}:{} `{needle}`", index + 1));
                }
            }
            for needle in DERIVE_NEEDLES {
                if line.contains(needle) {
                    derive_hits.push(format!("{relative}:{} `{needle}`", index + 1));
                }
            }
            if let Some((_, rest)) = line.split_once("NamedKey::") {
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_alphabetic())
                    .collect();
                if !ALLOWED_KEYS.contains(&name.as_str()) {
                    bail!(
                        "{relative}:{} binds `{name}`: the viewer's whole input contract is the six \
                         keys that type the owner's line or pan the camera, and a seventh binding is \
                         a control surface (08 §9, I8)",
                        index + 1
                    );
                }
                bound_keys.push(name);
            }
        }
    }
    for (label, hits) in [
        ("a state symbol", &state_hits),
        ("a setter-shaped widget", &setter_hits),
        ("a pointer or drag event", &pointer_hits),
        ("a derivation primitive", &derive_hits),
    ] {
        if !hits.is_empty() {
            bail!(
                "si-vulkan names {label} where the row forbids it: {}",
                hits.join("; ")
            );
        }
    }
    if bound_keys.is_empty() {
        bail!("si-vulkan binds no key: the owner's submit is the sense channel's only input path (08 §2)");
    }
    Ok(format!(
        "{scanned} src file(s): no state symbol, no setter-shaped fn, no pointer or drag event and no \
         hashing or temperament primitive — so the sleep, seed and fingerprint rows can only be labels \
         over handed-over artifacts — and the only key bindings are {bound} plus `Key::Character`, the \
         six that type the owner's line or pan the camera (08 §9, I8)",
        bound = {
            let mut seen: Vec<&str> = Vec::new();
            for key in &bound_keys {
                if !seen.contains(&key.as_str()) {
                    seen.push(key);
                }
            }
            format!("[{}]", seen.join(", "))
        }
    ))
}

/// 05 §20 / 08 §1's HEADLESS law, in its two halves.
///
/// The **structural** half is machine-checked here: the window lives in exactly one
/// crate, no crate that runs [02 §11]'s steps 1–13 (step 12b's four sleep passes
/// included) links it, and no source outside `si-vulkan` names an event loop or a
/// swapchain. The **measured** half — a build + run with process B never started
/// whose `state_hash.txt` is byte-identical to the viewer-present control
/// (**G5(a)**'s discipline) — is read from the artifact `si-cli view` writes when one
/// exists. `runs/` is gitignored, so a checkout with no viewer run reports that half
/// UNMEASURED in this leg rather than inventing a comparison; B13 DoD 5 runs it over
/// every gate.
fn check_headless_run(ws: &Workspace) -> Result<String> {
    // (1) and (2): one crate owns the window, and the loop's step order cannot link
    // it. `si-cli` is the launcher — it *starts* process B on request — and its edge
    // has to stay optional, or a headless build would link a window it never opens.
    let mut window_edges = Vec::new();
    for name in ws.crate_names()? {
        let manifest = ws.crate_manifest(&name)?;
        let edges = dependency_keys(&manifest);
        for dep in ["winit", "ash", "raw-window-handle"] {
            if edges.iter().any(|key| key == dep) {
                if name != "si-vulkan" {
                    bail!(
                        "{name} depends on `{dep}`: a second crate can open a window, so a loop step \
                         could wait on one (08 §1)"
                    );
                }
                window_edges.push(format!("{name}↗{dep}"));
            }
        }
        if edges.iter().any(|key| key == "si-vulkan") && name != "si-vulkan" {
            let optional = manifest
                .lines()
                .any(|line| line.contains("si-vulkan") && line.contains("optional = true"));
            if !(name == "si-cli" && optional) {
                bail!(
                    "{name} links si-vulkan (optional: {optional}): a crate in the step order would \
                     hold a handle to the window, and a session with process B never started would \
                     stop being a configuration"
                );
            }
        }
    }

    // (3) the loop never opens or waits on a window: the calls that would are named
    // in one crate only. The auditor holds these needles to assert this law, so it is
    // exempt by name, exactly as the other scans exempt it.
    const WINDOW_NEEDLES: [&str; 5] = [
        "EventLoop",
        "create_window",
        "swapchain",
        "winit::",
        "AppLifecycle",
    ];
    let mut leaks = Vec::new();
    let mut src_files = 0usize;
    for (relative, text) in ws.source_text()? {
        if !relative.contains("/src/") || Workspace::is_auditor(&relative) {
            continue;
        }
        src_files += 1;
        if relative.starts_with("crates/si-vulkan/src/") {
            continue;
        }
        for (index, raw) in text.lines().enumerate() {
            let line = code_part(raw);
            for needle in WINDOW_NEEDLES {
                if line.contains(needle) {
                    leaks.push(format!("{relative}:{} `{needle}`", index + 1));
                }
            }
        }
    }
    if !leaks.is_empty() {
        bail!(
            "a crate outside si-vulkan names window machinery ({}), so steps 1–13 are no longer \
             window-independent (08 §1)",
            leaks.join("; ")
        );
    }

    // (4) the measured half: the launcher stamps the session's `state_hash.txt` before
    // and after the window runs, and the headless value is the file the loop wrote with
    // no viewer in sight.
    let mut measured = Vec::new();
    let mut lanes = Vec::new();
    let runs = ws.path("runs");
    if runs.is_dir() {
        // `state_hash.txt` is one line per settled section (measured: a G13 run's hash is
        // three lines), and the log keeps one value per key, so both are folded to their
        // digest list before the comparison. The digests themselves are compared raw.
        let fold = |text: &str| {
            text.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        };
        for entry in fs::read_dir(&runs).with_context(|| format!("read {}", runs.display()))? {
            let dir = entry?.path();
            let hash_path = dir.join("state_hash.txt");
            let log_path = dir.join("viewer.log");
            if !(hash_path.is_file() && log_path.is_file()) {
                continue;
            }
            let on_disk = fold(&fs::read_to_string(&hash_path)?);
            let mut before = String::new();
            let mut after = String::new();
            let mut lane = String::new();
            for line in fs::read_to_string(&log_path)?.lines() {
                if let Some(value) = line.strip_prefix("state_hash before ") {
                    before = value.trim().to_string();
                } else if let Some(value) = line.strip_prefix("state_hash after ") {
                    after = value.trim().to_string();
                } else if let Some(value) = line.strip_prefix("lane ") {
                    lane = value.trim().to_string();
                }
            }
            if before != after || before != on_disk {
                bail!(
                    "{}: the viewer-present run hash differs from the headless one (before \
                     `{before}`, after `{after}`, on disk `{on_disk}`) — 08 §1's HEADLESS law and \
                     G5(a)'s byte-identity both fail here",
                    dir.display()
                );
            }
            if lane.is_empty() {
                bail!(
                    "{}: viewer.log names no lane, so a reader cannot tell a live \
                     `stream.siro` tail from a `mindlog.json` re-frame — the log states its \
                     witness or the view is not evidence (OD-150)",
                    dir.display()
                );
            }
            let name = dir.file_name().unwrap_or_default().to_string_lossy().into_owned();
            lanes.push(format!("{name} = {lane}"));
            measured.push(name);
        }
    }
    let evidence = if measured.is_empty() {
        "the hash half is UNMEASURED on this checkout (no runs/<id>/viewer.log present; runs/ is \
         gitignored) — B13 DoD 5 measures it over every gate"
            .to_string()
    } else {
        format!(
            "and {} viewer-present run(s) ({}) carry state_hash before == after == the headless \
             state_hash.txt digest-for-digest in file order, each naming its lane: {}",
            measured.len(),
            measured.join(", "),
            lanes.join("; ")
        )
    };
    Ok(format!(
        "the window edges are {} and no crate in the step order links si-vulkan; {src_files} src \
         file(s) scanned with `EventLoop`/`create_window`/`swapchain` named in si-vulkan alone, so \
         steps 1–13 (12b included) open and wait on nothing — {evidence} (**G5(a)**, 08 §1)",
        window_edges.join(", ")
    ))
}


// ----------------------------------------------------------- B20 stage checks

/// 13 §6.2 row 1 (**G20(a)(k)**): the voice's derivation order is closed, sealed,
/// seed-only, and a different stream from the 18 it sits beside. **OD-165**'s claim
/// is arithmetic — five rows, ten bytes, no `TEMPERAMENT` row drawn and no
/// non-`TONGUE` row reached — so it is graded here rather than in prose.
fn check_tongue_closed(ws: &Workspace) -> Result<String> {
    use si_core::consts::{self, Class, CONST_ROWS};
    use si_core::tongue::{Style, DERIV_ORDER_TONGUE, STYLE_ORDER_LEN};

    if !si_core::tongue::style_order_is_closed() {
        bail!("the sealed style order does not match the registry's TONGUE rows");
    }
    if DERIV_ORDER_TONGUE.len() != STYLE_ORDER_LEN {
        bail!("the style order is not its own sealed length");
    }
    if STYLE_ORDER_LEN != consts::tongue_count() {
        bail!(
            "the order draws {STYLE_ORDER_LEN} values but the registry has {} TONGUE rows",
            consts::tongue_count()
        );
    }
    if std::mem::size_of::<Style>() != STYLE_ORDER_LEN * 2 {
        bail!("the style vector is not 5 u16 values");
    }
    let drawn: Vec<&str> = DERIV_ORDER_TONGUE.iter().map(|id| id.registry_name()).collect();
    for name in &drawn {
        let row =
            consts::row(name).with_context(|| format!("{name} is drawn but has no registry row"))?;
        if row.class != Class::Tongue {
            bail!("{name} is drawn by the style order but is a {:?} row", row.class);
        }
    }
    for row in CONST_ROWS {
        if row.class == Class::Tongue && !drawn.contains(&row.name) {
            bail!("{} is a TONGUE row the style order never fills", row.name);
        }
        if matches!(row.class, Class::Law | Class::Ruling | Class::Species | Class::Temperament)
            && drawn.contains(&row.name)
        {
            bail!("{} is a {:?} row and must not be drawn by the voice", row.name, row.class);
        }
    }

    // The derivation reads no entropy, no file, no clock and no environment — the
    // same four files 13 §6.3 moves `check_temperament_closed` over to.
    for relative in [
        "crates/si-core/src/tongue.rs",
        "crates/si-core/src/temperament.rs",
        "crates/si-core/src/seed.rs",
        "crates/si-core/src/rng.rs",
    ] {
        let text = ws.read(relative)?;
        for needle in [
            "getrandom",
            "rand::",
            "SystemTime",
            "Instant::now",
            "env::var",
            "std::fs",
            "File::open",
        ] {
            if text.contains(needle) {
                bail!("{relative} reaches `{needle}`; the voice is seed-only");
            }
        }
    }

    // The drawn vector has no address outside its own module, exactly as the 18's
    // has none outside `temperament.rs` and `seed.rs`.
    for (relative, text) in ws.source_text()? {
        if Workspace::is_auditor(&relative) || relative.contains("si-core/src/tongue.rs") {
            continue;
        }
        if text.contains("style_vector") || text.contains("style_values") {
            bail!("{relative} names the style vector: the voice would be stored, not drawn");
        }
    }

    // Pure, and a stream of its own: the salt is the whole separation, so a style
    // vector that equalled the 18's first five values would mean an unsalted
    // continuation of stream T (**OD-165**, **OD-166**).
    for seed in [0u64, 1, 0x51, 0x5151, u64::MAX] {
        let style = si_core::tongue::derive_style(seed);
        if style != si_core::tongue::derive_style(seed) {
            bail!("seed {seed:#x} drew two different voices");
        }
        let temperament = si_core::seed::derive_temperament(seed);
        if style.canonical_bytes().as_slice() == &temperament.canonical_bytes()[..STYLE_ORDER_LEN * 2]
        {
            bail!("seed {seed:#x}'s voice is the temperament's first five values: TONGUE_DOMAIN does not separate the streams");
        }
    }
    Ok(format!(
        "{} values across the registry's {} TONGUE row(s), `Style` is {} bytes and no LAW, RULING, \
         SPECIES or TEMPERAMENT row is drawn; the four derivation files reach no entropy, file, clock \
         or environment; the vector is named nowhere outside `si-core/src/tongue.rs`; and the draw is a \
         stable function of the seed that differs from `derive_temperament`'s sequence at every seed \
         tried (**OD-165**, **G20(a)**)",
        STYLE_ORDER_LEN,
        consts::tongue_count(),
        std::mem::size_of::<Style>(),
    ))
}

/// 13 §6.2 row 2 (**G20(b)(c)(h)**, **OD-181**): the crate that walks the phrase
/// has no handle on the point table, in its manifest and in its own text. This is
/// the structural half of the citation-scoped vocabulary — **OD-181** was a hole
/// until a `License` argument existed, and a scan of the dependency edges is what
/// turns "it never looks a word up" back into a fact about the build.
fn check_tongue_is_leaf(ws: &Workspace) -> Result<String> {
    let manifest = ws.crate_manifest("si-tongue")?;
    let mut edges = dependency_keys(&manifest);
    edges.sort();
    edges.dedup();
    if edges != ["si-core", "si-walk"] {
        bail!(
            "si-tongue's dependency set is {edges:?}, not `si-core` + `si-walk`: with a store edge the walk \
             could reach the point table and **OD-181** would be a style rule again"
        );
    }
    let mut files = 0usize;
    for banned in ["si_store", "si_front", "si_learn", "si_affect", "si_read", "std::fs"] {
        for (relative, text) in ws.source_text()? {
            if !relative.starts_with("crates/si-tongue/") {
                continue;
            }
            if text.contains(banned) {
                bail!("si-tongue reaches `{banned}` in {relative}: the crate's whole removal surface is its return type");
            }
        }
    }
    for (relative, _) in ws.source_text()? {
        if relative.starts_with("crates/si-tongue/") {
            files += 1;
        }
    }
    if files == 0 {
        bail!("si-tongue has no scanned file");
    }

    // One caller. `si-cli` deliberately does not name the crate: a gate that graded
    // the license by calling the code that enforces it would grade nothing, so
    // `g20_license` re-derives it from the store instead.
    let mut callers = Vec::new();
    for name in ws.crate_names()? {
        if name == "si-tongue" {
            continue;
        }
        if dependency_keys(&ws.crate_manifest(&name)?).iter().any(|key| key == "si-tongue") {
            callers.push(name);
        }
    }
    if callers != ["si-session"] {
        bail!("the crates that depend on si-tongue are {callers:?}; 13 §1.1 gives the voice one caller");
    }
    let mut named = Vec::new();
    for (relative, text) in ws.source_text()? {
        if relative.starts_with("crates/si-tongue/") || Workspace::is_auditor(&relative) {
            continue;
        }
        if code_text(text).contains("si_tongue") {
            named.push(relative);
        }
    }
    if named.iter().any(|relative| !relative.starts_with("crates/si-session/")) {
        bail!("{named:?} name `si_tongue`; only the session may hold the phrase walk");
    }
    Ok(format!(
        "the manifest's edges are exactly si-core + si-walk, `si_store`/`si_front`/`si_learn`/`si_affect`/`si_read`/`std::fs` \
         appear in none of its {files} file(s), and the only crate that names `si_tongue` is si-session ({} file(s))",
        named.len()
    ))
}

/// 13 §6.2 row 3 (**G20(d)(f)**): the walk has no write capability, and the call
/// site uses its answer for one thing — the string the owner reads.
fn check_tongue_writes_nothing(ws: &Workspace) -> Result<String> {
    const BANNED: [&str; 5] = ["commit", "append", "put", "write", "save"];
    let mut files = 0usize;
    let mut mutable = Vec::new();
    for (relative, text) in ws.source_text()? {
        if !relative.starts_with("crates/si-tongue/src/") {
            continue;
        }
        files += 1;
        for (index, line) in text.lines().enumerate() {
            let code = code_of(line);
            for needle in BANNED {
                if contains_word(code, needle) {
                    bail!("{relative}:{} names `{needle}` in code: the phrase walk is a pure function and 13 §4.1's contract is `-> Option<String>`", index + 1);
                }
            }
            // The one `&mut` the crate may hold is the `Rng` it is handed: a draw
            // is the individual's own, and the walker advancing it is all it does.
            for (offset, _) in code.match_indices("&mut ") {
                let target: String = code[offset + "&mut ".len()..]
                    .chars()
                    .take_while(|ch| ch.is_alphanumeric() || *ch == '_' || *ch == ':')
                    .collect();
                if target != "Rng" {
                    bail!("{relative}:{} takes `&mut {target}` — the only thing the walk may mutate is the draw it was handed (**OD-183**)", index + 1);
                }
                mutable.push(format!("{}:{}", relative, index + 1));
            }
        }
    }
    if files == 0 {
        bail!("no si-tongue src file was scanned");
    }
    let lib = ws.read("crates/si-tongue/src/lib.rs")?;
    let signature = function_signature(&lib, "pub fn realize").context("si-tongue has no `pub fn realize`")?;
    if !signature.contains("-> Option<String>") {
        bail!("the phrase walk returns `{signature}`; a refusal must be the type, not a panic or a writer");
    }

    // The call site: one assignment, to the printed surface and to nothing else.
    let session = code_text(ws.read("crates/si-session/src/lib.rs")?);
    let calls: Vec<&str> =
        session.lines().filter(|line| line.contains("self.realized_surface(")).collect();
    if calls.len() != 1 {
        bail!(
            "{} call site(s) of `realized_surface`; 13 §4.1 gives tick step 9a one, and a second is a \
             second emission lane",
            calls.len()
        );
    }
    if !calls[0].trim_start().starts_with("let surface = ") {
        bail!(
            "the call site binds `{}` rather than `surface`: the voice may replace the printed string and \
             nothing else (**G20(b)**)",
            calls[0].trim()
        );
    }
    Ok(format!(
        "{files} src file(s) hold no `commit`/`append`/`put`/`write`/`save` symbol in code and their only \
         `&mut` is the drawn `Rng` ({}); `realize` returns `Option<String>`; and si-session's single call \
         site binds it to `surface` and to nothing else (**G20(d)**, **G20(f)**)",
        mutable.join(", ")
    ))
}

/// 13 §6.2 row 4 (**G20(e)**, **OD-168**, **OD-182**, **OD-183**): the voice reads
/// no arousal, directly or through a seed mixed from something that does. Item 3 of
/// the nine hashed items is the stamp arena, so the state hash is not an operand
/// either — which is why `render_seed` takes three named values and no `&Session`.
fn check_tongue_reads_no_a(ws: &Workspace) -> Result<String> {
    let core = ws.read("crates/si-core/src/tongue.rs")?;
    let mut scanned: Vec<(String, String)> =
        vec![("crates/si-core/src/tongue.rs".to_string(), core.clone())];
    for (relative, text) in ws.source_text()? {
        if relative.starts_with("crates/si-tongue/src/") {
            scanned.push((relative, text));
        }
    }
    if scanned.len() < 2 {
        bail!("the arousal scan found nothing to read");
    }
    for (relative, text) in &scanned {
        let code = code_text(text.clone());
        for needle in ["a_at", "a()", "arousal", "Integrator", "si_affect"] {
            if code.contains(needle) {
                bail!("{relative} reaches `{needle}` in code: the voice cannot know how loud the tick was (**OD-168**)");
            }
        }
    }

    // The three operands, and nothing that could carry a fourth in disguise.
    let signature =
        function_signature(&core, "pub fn render_seed").context("si-core has no `pub fn render_seed`")?;
    let arguments = argument_list(&signature);
    if arguments.len() != 3 {
        bail!(
            "render_seed takes {} operands ({}); 04 §34 AF.2 fixes three",
            arguments.len(),
            arguments.join(", ")
        );
    }
    for operand in ["birth_seed", "style_digest", "thought_id"] {
        if !arguments.iter().any(|argument| argument.starts_with(operand)) {
            bail!("render_seed has no `{operand}` operand: {}", arguments.join(", "));
        }
    }
    let body = function_body(&core, "pub fn render_seed").context("render_seed has no body")?;
    for banned in ["hash", "session", "State", "a_at"] {
        if body.contains(banned) {
            bail!("render_seed's body reaches `{banned}`: a state hash mixes arousal in through the stamp arena (**OD-183**)");
        }
    }
    Ok(format!(
        "`a_at`, `a()`, `arousal`, `Integrator` and `si_affect` appear in the code of none of the {} file(s) \
         the voice is built from; `render_seed({})` reads the birth seed, the style digest and the thought \
         id, and its body names no hash, session or state (**G20(e)**, **OD-168**, **OD-183**)",
        scanned.len(),
        arguments.join(", ")
    ))
}

/// 13 §6.2 row 5 (**G20(i)**, **OD-87**, **OD-177**): the fifth class is drawn,
/// never installed. The guard that refuses it is `launch_constants`' existing
/// `Species | Ruling` text, and 13 §6.3 requires that text byte-identical, so this
/// check reads it verbatim rather than paraphrasing it.
fn check_style_never_installed(ws: &Workspace) -> Result<String> {
    use si_core::consts::{Class, CONST_ROWS};
    use si_core::tongue::DERIV_ORDER_TONGUE;

    let drawn: Vec<&str> = DERIV_ORDER_TONGUE.iter().map(|id| id.registry_name()).collect();

    // (1) No fixture declares a style row. A scenario's `#` lines are prose and the
    // law is about the channel, so the scan reads the directive lines only: the one
    // way a fixture reaches a constant is the launch channel, and that channel
    // refuses by class (arm (3) below).
    let mut fixtures = 0usize;
    let scenario_dir = ws.path("harness/scenarios");
    for entry in
        fs::read_dir(&scenario_dir).with_context(|| format!("read {}", scenario_dir.display()))?
    {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext != "txt") {
            continue;
        }
        fixtures += 1;
        let text = fs::read_to_string(&path)?;
        let directives = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");
        if drawn.iter().any(|name| directives.contains(name)) {
            bail!(
                "{} names a style row in a step line: a scenario could install the voice, which is what \
                 `Class::Tongue` forbids (**OD-177**)",
                ws.relative(&path)
            );
        }
    }

    // (2) No published sweep artifact lists one, and no enumeration produces one.
    let mut prefixes: Vec<&str> = Vec::new();
    for row in CONST_ROWS {
        if let Some(tag) = row.sweep {
            let prefix = tag.split('/').next().unwrap_or(tag);
            if !prefixes.contains(&prefix) {
                prefixes.push(prefix);
            }
        }
    }
    let mut axes = 0usize;
    let mut artifacts = 0usize;
    for prefix in &prefixes {
        for axis in super::sweep::axes(prefix) {
            axes += 1;
            if axis.class == Class::Tongue {
                bail!("sweep {prefix} enumerates the style row {}", axis.name);
            }
        }
        for path in super::sweep::artifact_paths(prefix) {
            if !ws.exists(path) {
                continue;
            }
            let document: serde_json::Value = serde_json::from_str(&ws.read(path)?)
                .with_context(|| format!("parse {path}"))?;
            for axis in document["axes"].as_array().into_iter().flatten() {
                let name = axis["name"].as_str().unwrap_or_default();
                if drawn.contains(&name) {
                    bail!("{path} publishes the style row {name} as a swept axis");
                }
            }
            artifacts += 1;
        }
    }

    // (3) The channel, and the two arms B20 added to it.
    let runner = ws.read("crates/si-cli/src/harness/runner.rs")?.replace("\r\n", "\n");
    let start = runner
        .find("fn launch_constants")
        .context("runner.rs has no `launch_constants` for the channel to guard")?;
    let body = &runner[start..];
    let body = match body.find("\nfn ") {
        Some(end) => &body[..end],
        None => body,
    };
    if !body.contains(
        "!matches!(\n            row.class,\n            si_core::consts::Class::Species | si_core::consts::Class::Ruling\n        )",
    ) {
        bail!(
            "`launch_constants`' class guard is not the byte-identical `Species | Ruling` refusal, so the \
             fifth class lost its only wall (13 §6.3)"
        );
    }
    for name in ["TONGUE", "PHRASE_MAX_STEPS"] {
        if !body.contains(&format!("\"{name}\" =>")) {
            bail!("`launch_constants` has no installed arm for {name}; 13 §3.2 gives V14 two");
        }
    }
    // The ban is on a code arm, not on the prose that explains the refusal: the
    // function's own comment names the five rows to say why it rejects them.
    if code_text(body.to_string()).contains("STYLE_") {
        bail!("`launch_constants` names a STYLE_ row: the drawn voice became an installed constant");
    }
    Ok(format!(
        "{fixtures} fixture(s) name no style row, the {} sweep enumeration(s) own {axes} axis/axes of which \
         none is `Class::Tongue`, {artifacts} published artifact(s) list none in `axes`, `launch_constants`' \
         guard text is byte-identical to the `Species | Ruling` refusal and its B20 arms install only `TONGUE` \
         and `PHRASE_MAX_STEPS` (**G20(i)**, **OD-87**, **OD-177**)",
        prefixes.len()
    ))
}

/// 13 §6.2 row 6 (**G20(c)(h)(k)**, R-21): the crate holds no word table. **OD-181**
/// is why this is a static leg and not only a gate leg — with no handle on the point
/// table, the only way a word can enter a phrase is a literal compiled into the
/// walker, so "no table of strings, and every push comes from a `License`" is the
/// whole of the lexicon ban for this crate.
fn check_tongue_license_only(ws: &Workspace) -> Result<String> {
    use std::collections::BTreeSet;

    // The joiner, held here the way `check_emotion_words` holds the label words: the
    // auditor names the string it forbids any other shape of. `si-cli` has no
    // dependency edge on `si-tongue`, and 13 §6.2 row 6 is the check that would make
    // one necessary if it read the constant instead.
    const JOINER: &str = ", ";

    let mut lines: Vec<String> = Vec::new();
    for (relative, text) in ws.source_text()? {
        if !relative.starts_with("crates/si-tongue/src/") {
            continue;
        }
        lines.push(code_text(text));
    }
    if lines.is_empty() {
        bail!("si-tongue has no src text to scan");
    }
    let code = lines.join("\n");

    // (1) No string table of any size.
    for needle in ["[&str; ", "[&'static str; "] {
        for (index, line) in code.lines().enumerate() {
            let Some(at) = line.find(needle) else { continue };
            let rest = &line[at + needle.len()..];
            let digits: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
            let count: usize = digits.parse().unwrap_or(0);
            if count > 1 {
                bail!("si-tongue declares a {count}-entry string table at code line {}: a shipped lexicon, in the crate that may not own one (R-14, R-21)", index + 1);
            }
        }
    }

    // (2) The joiner is the only literal, and `tokenize` yields no token from it, so
    // it can never print as an unlicensed word (13 §0 fact 8).
    let literals: BTreeSet<String> =
        string_literals(&code).into_iter().filter(|text| text.chars().count() > 1).collect();
    let allowed: BTreeSet<String> = [JOINER.to_string()].into_iter().collect();
    if literals != allowed {
        bail!(
            "the crate's multi-character literals are {literals:?}, not only the joiner {allowed:?}: \
             every other string a walker can print is a word it was given outside its license"
        );
    }

    // (3) Every push into the phrase buffer takes its bytes from a `License`. The
    // walk keeps other buffers too — its candidate arcs and their weights — and a
    // weight is a number, not a phrase byte, so the claim is scoped to the one
    // vector whose contents reach the surface. **OD-189** moved the choice of
    // *which* licensed bytes into `utter`, so the push site must call `utter` and
    // (4) proves what `utter` may return.
    let mut pushes = 0usize;
    for (index, line) in code.lines().enumerate() {
        for (offset, _) in line.match_indices("parts.push(") {
            let source = &line[offset + "parts.push(".len()..];
            if !source.starts_with("utter(") {
                bail!("si-tongue pushes `{}` at code line {}: a phrase byte that does not come from `utter`, the one chooser over licensed surfaces (**G20(h)**, **OD-189**)", source.trim(), index + 1);
            }
            pushes += 1;
        }
        for (offset, _) in line.match_indices("parts.") {
            let rest = &line[offset + "parts.".len()..];
            if rest.starts_with("push(") || rest.starts_with("len()") || rest.starts_with("join(") {
                continue;
            }
            bail!("si-tongue reaches the phrase buffer with `{}`: the only ways it grows are a seeded license and a licensed push", rest.trim());
        }
    }
    let seeded = code.matches("vec![utter(anchor_license, rng)?").count();
    if pushes == 0 || seeded == 0 {
        bail!("the scan found no `utter` push and no seeded anchor surface: it read an empty walker");
    }

    // (4) `utter` is the one chooser, and its only byte sources are a license
    // surface and a taught variant's surface. Both call sites relate their
    // argument to the caller's `license` slice, so the crate still cannot spell a
    // word the citation did not hand it (**OD-181**, **OD-189**).
    let Some(utter_at) = code.find("fn utter") else {
        bail!("si-tongue has no `utter`: the phrase bytes have no single chooser to audit");
    };
    let after = &code[utter_at + "fn utter".len()..];
    let next_fn = ["\nfn ", "\npub fn "]
        .iter()
        .filter_map(|needle| after.find(needle))
        .min()
        .map(|pos| utter_at + "fn utter".len() + pos)
        .unwrap_or(code.len());
    let body = &code[utter_at..next_fn];
    for (offset, _) in body.match_indices(".surface") {
        let ident = body[..offset]
            .rsplit(|ch: char| !(ch.is_alphanumeric() || ch == '_'))
            .next()
            .unwrap_or("");
        if ident != "entry" && ident != "variant" {
            bail!("`utter` reads `{ident}.surface`: a phrase byte that is not a `License` surface or a `Variant` surface (**OD-189**)");
        }
    }
    if !body.contains("entry.surface.as_str()") || !body.contains("variant.surface.as_str()") {
        bail!("`utter` does not draw from both `License::surface` and `Variant::surface`: the bank or the canonical half is unreachable");
    }
    let returned = body.matches("Some(").count();
    let then_returned = body.matches("then_some(").count();
    if returned != 1 || !body.contains("Some(entry.surface.as_str())") {
        bail!("`utter` has {returned} `Some(…)` return(s): the canonical arm must be `Some(entry.surface.as_str())` and nothing else");
    }
    if then_returned != 1 || !body.contains("then_some(chosen)") {
        bail!("`utter` has {then_returned} `then_some(…)` return(s): the drawn arm must return `chosen`, the value (4) just audited");
    }
    if !code.contains("let anchor_license = license.iter().find(")
        || !code.contains("let entry = license.iter().find(")
    {
        bail!("a license lookup is missing: `anchor_license` and the push site's `entry` must both come out of the caller's `license` slice");
    }
    Ok(format!(
        "no string table, {pushes} licensed `parts.push(utter(…))` site(s), {seeded} seeded anchor \
         site(s), and the only multi-character literal is the joiner `{}` — `utter` chooses only among \
         a `License` surface and its taught variants, both looked up from the caller's citation \
         (**G20(c)**, **G20(h)**, **OD-181**, **OD-189**)",
        JOINER
    ))
}
