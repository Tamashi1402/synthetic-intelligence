//! B11 DoD 1 ([11 §B11](../../../engineering/11-build-plan.md)): the three laws of
//! [08 §1](../../../engineering/08-gui-observability.md) are asserted against this
//! crate's **real sources and manifest**, not against a comment in them.
//!
//! A grep is the right instrument for these three specifically because each law
//! is a statement about what the crate *does not contain*: no second writable
//! path, no control-crate edge, no panel that draws a kind §2.1 reserves. 05 §20
//! runs the same scans without a session; this file is the crate-local half of
//! that deliberate redundancy — "the graph catches today's violation, the check
//! catches tomorrow's refactor".

use serde_json::json;
use si_core::container::{Container, Format};
use si_core::eventlog::{kind, Event};
use si_vulkan::frames::{frame_bytes, inbox_header, sense_frame, validate_kind, Lane, Outbound};
use si_vulkan::layout::{pos, shell_of, EXTENT};
use si_vulkan::panels::{build, Artifacts, FORBIDDEN_WIDGETS, READS, Row};
use std::path::Path;

/// Every `.rs` under `src/`, as `(crate-relative path, code)` where `code` has its
/// comments stripped — these scans ask about *code* literals, and a doc comment
/// quoting an emotion word is not the crate carrying a lexicon.
fn sources() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
        for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                walk(&path, root, out);
            } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                let raw = std::fs::read_to_string(&path).expect("read");
                let relative = path
                    .strip_prefix(root)
                    .expect("a source under src/")
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((format!("src/{relative}"), decomment(&raw)));
            }
        }
    }
    let mut out = Vec::new();
    walk(&root, &root, &mut out);
    out.sort();
    out
}

/// Line comments removed. No string in this crate contains `//`, so a plain
/// truncate is exact rather than approximate.
fn decomment(text: &str) -> String {
    text.lines()
        .map(|line| match line.find("//") {
            Some(cut) => &line[..cut],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The string literals of comment-free code, escapes skipped.
fn literals(code: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = code.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '"' {
            continue;
        }
        let mut current = String::new();
        while let Some(next) = chars.next() {
            match next {
                '\\' => {
                    chars.next();
                }
                '"' => break,
                other => current.push(other),
            }
        }
        out.push(current);
    }
    out
}

fn manifest() -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).expect("manifest")
}

/// The `si-*` packages named by `[dependencies]` — the manifest's own edge list,
/// which is what a refactor cannot hide behind a re-export.
fn si_dependencies() -> Vec<String> {
    let mut section = String::new();
    let mut found = Vec::new();
    for line in manifest().lines() {
        let line = line.trim();
        if line.starts_with('[') {
            section = line.trim_matches(|c| c == '[' || c == ']').to_string();
            continue;
        }
        if section != "dependencies" {
            continue;
        }
        let code = decomment(line);
        if let Some(name) = code
            .split_whitespace()
            .next()
            .and_then(|first| first.split(['=', '.']).next())
        {
            if name.starts_with("si-") {
                found.push(name.to_string());
            }
        }
    }
    found
}

/// DoD 1, assert (a): the crate's only writable path is `sense_inbox.siro`.
///
/// `WRITE_MARKERS` is the set of std calls that create or open a file for
/// writing; a write site has to name the inbox within its own window of lines.
/// `NEVER_MARKERS` are the calls that would let process B touch a file it does not
/// own — a rename or a delete over a run directory *is* the second input path
/// 08 §1 says does not exist.
#[test]
fn the_only_writable_path_is_the_sense_inbox() {
    assert_eq!(si_vulkan::WRITABLE_FILE, "sense_inbox.siro");
    const WRITE_MARKERS: [&str; 5] = [
        "File::create",
        "OpenOptions",
        "File::options",
        "fs::write",
        ".write(true)",
    ];
    const NEVER_MARKERS: [&str; 6] = [
        "fs::remove_file",
        "fs::rename",
        "create_new",
        "fs::remove_dir",
        "fs::copy",
        "fs::hard_link",
    ];
    let mut sites = 0usize;
    for (path, code) in sources() {
        let lines: Vec<&str> = code.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            for forbidden in NEVER_MARKERS {
                assert!(
                    !line.contains(forbidden),
                    "{path}:{} calls `{forbidden}` — process B deletes and renames nothing",
                    index + 1
                );
            }
            if WRITE_MARKERS.iter().any(|marker| line.contains(marker)) {
                let window = lines[index.saturating_sub(4)..(index + 5).min(lines.len())].join(" ");
                assert!(
                    window.contains("WRITABLE_FILE") || window.contains("sense_inbox"),
                    "{}:{} writes a file this crate does not own: {line}",
                    path,
                    index + 1
                );
                sites += 1;
            }
        }
    }
    // The pure half has no write site at all and `surface` adds one; either way
    // the count is at most one, because 08 §1 says process B has a single file.
    assert!(sites <= 1, "one writable file, found {sites} write sites");

    // And the name is not reconstructible from a pattern: the crate owns exactly
    // one `.siro` literal, and it is the inbox.
    let named: Vec<(String, String)> = sources()
        .into_iter()
        .flat_map(|(path, code)| {
            literals(&code)
                .into_iter()
                .filter(|literal| literal.ends_with(".siro"))
                .map(move |literal| (path.clone(), literal))
        })
        .collect();
    match named.as_slice() {
        [] => {}
        [(path, literal)] => assert_eq!(
            (path.as_str(), literal.as_str()),
            ("src/lib.rs", "sense_inbox.siro"),
            "08 §1's OBSERVER-ONLY law names one file"
        ),
        more => panic!("more than one .siro path literal: {more:?}"),
    }
}

/// DoD 1, assert (b): no control-crate import — in the sources and in the
/// manifest, which is where a `use` that got refactored away still lives.
#[test]
fn no_control_crate_edge_exists() {
    assert_eq!(si_vulkan::CONTROL_CRATES, ["si-learn", "si-read", "si-walk"]);
    assert_eq!(
        si_dependencies(),
        vec![si_vulkan::ALLOWED_SI_DEPENDENCY.to_string()],
        "02 §10: si-vulkan's one SI edge is si-core — displays, never decides (**OD-44**)"
    );
    for (path, code) in sources() {
        for banned in si_vulkan::CONTROL_CRATES {
            let module = banned.replace('-', "_");
            for needle in [format!("{module}::"), format!("use {module}"), format!("::{module}.")] {
                assert!(!code.contains(&needle), "{path} reaches {banned} through `{needle}`");
            }
        }
        // 05 §20's `check_gui_observer_only` wants the stronger half too: no
        // lattice handle, no door, no organ. The crates that own those types are
        // absent from the graph by the assert above, so this is the same law
        // spelled against a future dependency addition.
        for needle in ["si_store", "si_session", "si_affect", "si_front", "si_audit", "si_sleep"] {
            assert!(!code.contains(needle), "{path} holds a handle through {needle}");
        }
    }
}

/// A lane body of `{}` is enough for a parser test: every panel field reads
/// `absent` on a key it does not carry, so an unparseable kind fails in the parser
/// rather than silently in a draw.
fn lane_of(frames: &[(u8, &str)]) -> Vec<u8> {
    let mut payload = Vec::new();
    for (index, (kind_id, body)) in frames.iter().enumerate() {
        let event = Event {
            seq: index as u64,
            t: 1,
            source: 0,
            kind: *kind_id,
            body: body.as_bytes().to_vec(),
        };
        payload.extend_from_slice(&frame_bytes(&event));
    }
    Container::new(Format::StreamRecord, json!({ "lane": "stream" }), payload)
        .encode()
        .expect("encode")
}

/// DoD 1, assert (c): the frame parser handles kinds 0–20 and treats 18 as
/// reserved.
///
/// "Handles" is tested per kind so a failure names the row, then over the whole
/// lane to prove no two rows interact, then row 18 *inside* the full lane to prove
/// the refusal is of the lane and not of one frame.
#[test]
fn the_parser_covers_kinds_zero_to_twenty_and_refuses_eighteen() {
    for (value, name) in kind::SEALED {
        let bytes = lane_of(&[(value, "{}")]);
        if kind::is_reserved(value) {
            let err = Lane::decode(&bytes)
                .expect_err(&format!("kind {value} ({name}) must be refused"))
                .to_string();
            assert!(err.contains("reserved") && err.contains("OD-67"), "{err}");
            assert!(validate_kind(value).is_err());
            assert_eq!(name, "probe_result (reserved, never emitted)");
        } else {
            let lane = Lane::decode(&bytes)
                .unwrap_or_else(|e| panic!("kind {value} ({name}) must parse: {e}"));
            assert_eq!(lane.frames.len(), 1);
            assert_eq!(lane.frames[0].kind_name(), Some(name));
            assert!(validate_kind(value).is_ok());
        }
    }
    // 08 §2.1 is the table; **OD-143** is the one record above it with a name and
    // no row, and anything past that is the seventh sensor I8 forbids.
    assert!(validate_kind(kind::IGNITION).is_ok());
    assert!(validate_kind(kind::LAST_SEALED + 2).is_err());
    assert!(validate_kind(u8::MAX).is_err());

    let emitted: Vec<u8> = kind::SEALED
        .iter()
        .map(|(value, _)| *value)
        .filter(|value| !kind::is_reserved(*value))
        .chain(std::iter::once(kind::IGNITION))
        .collect();
    assert_eq!(emitted.len(), 21, "20 table rows the loop emits, plus OD-143");
    let all: Vec<(u8, &str)> = emitted.iter().map(|value| (*value, "{}")).collect();
    let lane = Lane::decode(&lane_of(&all)).expect("a lane of every emitted kind");
    assert_eq!(lane.frames.len(), 21);
    assert_eq!(lane.frames_behind(19), 1, "§2.2's lag counter counts what is undrawn");

    let mut with_probe = all.clone();
    with_probe.push((kind::PROBE_RESULT, "{}"));
    let err = Lane::decode(&lane_of(&with_probe))
        .expect_err("a reserved row refuses the whole lane")
        .to_string();
    assert!(err.contains("probe_result"), "{err}");
}

/// A lane is refused by **name**, not by a panic, when it is not a stream lane or
/// not the version this build writes: opening the state snapshot as a panel would
/// otherwise draw the store's bytes as text.
#[test]
fn a_lane_is_refused_by_name() {
    let lattice = Container::new(Format::StateSnapshot, json!({ "x": 1 }), Vec::new())
        .encode()
        .expect("encode");
    let err = Lane::decode(&lattice)
        .expect_err("a state container is not a lane")
        .to_string();
    assert!(err.contains("SIro") && err.contains("SIst"), "{err}");

    let stale = Container {
        format: Format::StreamRecord,
        version: Format::StreamRecord.container_version() - 1,
        header: json!({ "lane": "stream" }),
        payload: Vec::new(),
    }
    .encode()
    .expect("encode");
    assert!(Lane::decode(&stale)
        .expect_err("a pre-version lane is not this build's lane")
        .to_string()
        .contains("version"));
}

/// The one outbound shape round-trips through the shared codec: what process B
/// writes, process A reads with [01 §9]'s header, and the B18 prefix survives.
#[test]
fn a_keystroke_round_trips_as_the_owners_own_frame() {
    let event = sense_frame(
        7,
        42,
        &Outbound {
            text: "what is warm".to_string(),
            speaker_identity: 0,
            conversation: 3,
        },
    )
    .expect("sense frame");
    assert_eq!(event.source, si_vulkan::SOURCE_OWNER);
    assert_eq!(event.kind, kind::SENSE_ECHO);
    let lane = Lane::decode(
        &Container::new(Format::StreamRecord, inbox_header("run-7"), frame_bytes(&event))
            .encode()
            .expect("encode"),
    )
    .expect("decode");
    assert_eq!(lane.header["lane"], "sense_inbox");
    assert_eq!(lane.header["writer"], "viewer");
    assert_eq!(lane.header["run_id"], "run-7");
    let (identity, conversation, text) =
        si_core::eventlog::decode_sense_body(lane.frames[0].body.as_bytes()).expect("B18 body");
    assert_eq!((identity, conversation), (0, 3));
    assert_eq!(text, b"what is warm");

    // The supporter's byte is read here, never minted: an empty line is refused,
    // and §9's "send this line as grimm" has no code path to reach.
    assert!(sense_frame(
        8,
        42,
        &Outbound {
            text: String::new(),
            speaker_identity: 0,
            conversation: 3,
        }
    )
    .is_err());
    assert_eq!(si_vulkan::SOURCE_SUPPORTER, 1);
    assert_eq!(si_core::SOURCE_LITERALS, ["owner", "grimm"]);
}

/// 08 §5.1: positions are a deterministic pure function of identity, and the
/// arithmetic is integer-only — a float here would be a re-simulation, and a
/// learned or force-directed layout is forbidden outright.
#[test]
fn layout_is_a_pure_function_of_identity() {
    for handle in [0u32, 1, 7, 4096, u32::MAX] {
        let once = pos(handle, None);
        for _ in 0..3 {
            assert_eq!(pos(handle, None), once, "pos is derived, never memoized");
        }
        for axis in once {
            assert!(axis.abs() < EXTENT, "{axis} escapes the grid");
        }
        // A shelf owns a band, so the same point in two shelves is in two bands.
        assert_ne!(shell_of(0), shell_of(1));
        assert_ne!(pos(handle, Some(4)), pos(handle, Some(5)));
    }
    for (path, code) in sources() {
        if !path.contains("layout.rs") {
            continue;
        }
        for banned in ["f32", "f64", "sin", "cos", "tan", "sqrt", "pow", "float"] {
            assert!(
                !code.contains(banned),
                "layout.rs uses `{banned}` — §5.1 wants quantized grid ints, and a trig or \
                 force-directed layout is a re-simulation"
            );
        }
    }
    // A reserved shelf still renders, labelled rather than skipped (R-12).
    let shells = si_vulkan::panels::shells();
    assert_eq!(shells.len(), 11, "05 §20's check_registry_size counts eleven");
    assert_eq!(
        shells.iter().filter(|(_, label, _)| label.contains("reserved")).count(),
        2,
        "strategies and events stay reserved-empty until B17"
    );
}

/// 08 §3.1/§5.2: the atlas holds glyphs and no vocabulary. Every literal in the
/// file is a shape — the seven rows of five `#`/`.` marks separated by `/` — or a
/// single mark; a word anywhere in it would be a lexicon in the assets, which is
/// R-03's shape inside `src/`.
#[test]
fn the_atlas_holds_glyphs_and_no_vocabulary() {
    use si_vulkan::atlas::{bake, cells, glyph, FIRST, LAST, SHEET_BYTES, SUBSTITUTE_INDEX};
    assert_eq!(LAST - FIRST + 1, 95, "printable ASCII only, published as a limit");
    assert_eq!(bake().len(), SHEET_BYTES);
    assert_eq!(cells("a~").len(), 2);
    assert!(!cells("a~")[0].substituted);
    assert_eq!(cells("\u{7f}")[0].atlas_index, SUBSTITUTE_INDEX);
    assert!(cells("\u{7f}")[0].substituted);
    assert_eq!(cells("a~")[0].atlas_index, 97 - FIRST as u16);
    assert!(glyph('é').is_none(), "outside the band is a refusal, not a guess");
    assert_eq!(si_vulkan::atlas::width_of("abcd"), 4 * si_vulkan::atlas::CELL_W);

    for (path, code) in sources() {
        if !path.contains("atlas.rs") {
            continue;
        }
        for literal in literals(&code) {
            if literal.len() < 2 {
                continue;
            }
            assert!(
                literal.chars().all(|c| matches!(c, '#' | '.' | '/')),
                "atlas.rs carries the literal `{literal}` — glyphs, not words (08 §3.1)"
            );
            assert_eq!(
                literal.split('/').count(),
                si_vulkan::atlas::CELL_H,
                "`{literal}` is not a cell"
            );
            assert!(literal
                .split('/')
                .all(|row| row.len() == si_vulkan::atlas::CELL_W));
        }
    }
}

/// 08 §8's table as a law: a panel draws only kinds its row lists, and every kind
/// the loop emits has a row.
#[test]
fn no_panel_draws_a_kind_its_row_does_not_list() {
    let baseline = build(&Lane::decode(&lane_of(&[])).expect("empty lane"), &Artifacts::default(), false);
    let five = ["chat", "emotions", "preview", "organs", "perf"];
    assert_eq!(READS.len(), 20, "the sealed table minus the reserved row");
    for (value, panels) in READS {
        for panel in panels {
            assert!(five.contains(panel), "§8 names {panel}");
        }
        let body = sample_body(value);
        let lane = Lane::decode(&lane_of(&[(value, &body)])).expect("decode");
        let now = build(&lane, &Artifacts::default(), false);
        for panel in grew_panels(&baseline, &now) {
            assert!(
                panels.contains(&panel),
                "kind {value} drew into {panel}, which §8's row does not list"
            );
        }
    }
    for (value, _) in kind::SEALED {
        if kind::is_reserved(value) {
            assert!(!READS.iter().any(|(id, _)| *id == value), "a reserved row has no panel (**OD-67**)");
            continue;
        }
        assert!(READS.iter().any(|(id, _)| *id == value), "kind {value} has no §8 row");
    }
    assert_eq!(FORBIDDEN_WIDGETS.len(), 10, "08 §9's table is ten rows");
    for (widget, citation) in FORBIDDEN_WIDGETS {
        assert!(!widget.is_empty() && !citation.is_empty(), "§9's row {widget}");
    }
}

/// A body that gives each kind something to draw, so the subset check above is not
/// proven by drawing nothing.
fn sample_body(value: u8) -> String {
    match value {
        kind::SENSE_ECHO | kind::DECODED | kind::GAP | kind::ASK | kind::COMMIT | kind::SUPERSEDE
        | kind::STAGE | kind::SEAM_REJECT | kind::SLEEP_PASS => r#"{"line":"x"}"#.to_string(),
        kind::LIT_ROUND0 => r#"{"lit":[{"point":3,"q12":100}]}"#.to_string(),
        kind::ROUND => r#"{"k":1,"lit_set":[{"point":4,"q12":80}],"new_edges":[{"from":4,"to":5,"weight_eff_q12":64,"base":0}],"a_used_u8":10,"modulation":{"reach":2,"decay_q12":512,"tau_shift_q12":0}}"#.to_string(),
        kind::SETTLE => r#"{"thought":1,"stop":"settled","settle":[{"point":4,"q12":80}]}"#.to_string(),
        kind::SIGNAL_EVENT => r#"{"signals":1,"a_after":512,"applies_at":2,"thought":1,"rows":[]}"#.to_string(),
        kind::A_UPDATE => r#"{"round":1,"a_used":512,"a_after":520,"applies_at":2}"#.to_string(),
        kind::MEDIAN_USED => r#"{"thought":1,"round":1,"rows":[],"producer":"absent"}"#.to_string(),
        kind::DOOR_CALL => r#"{"door":"update","thought":1,"arg_digest":"0x00"}"#.to_string(),
        kind::LABEL => r#"{"label":"answer-shaped","thought":1}"#.to_string(),
        kind::OUTPUT => r#"{"surface":"s","tag":"FACT","label":"x"}"#.to_string(),
        kind::TICK_END => r#"{"eventlog_len":4}"#.to_string(),
        kind::TEMPERAMENT_READOUT => r#"{"birth_seed":"0x1","seed_format":"0x2","deriv_order_len":4,"rows":[],"temperament_digest":"0x3"}"#.to_string(),
        other => panic!("no sample body for kind {other}"),
    }
}

/// Which of the five grew since the baseline lane — rows, plus the preview's
/// geometry, which is the one panel that is not only rows.
fn grew_panels(baseline: &si_vulkan::panels::Panels, now: &si_vulkan::panels::Panels) -> Vec<&'static str> {
    let mut grew = Vec::new();
    let preview_len = |p: &si_vulkan::panels::Preview| p.nodes.len() + p.edges.len() + p.rows.len();
    if now.chat.rows.len() > baseline.chat.rows.len() {
        grew.push("chat");
    }
    if now.emotions.rows.len() > baseline.emotions.rows.len() {
        grew.push("emotions");
    }
    if preview_len(&now.preview) > preview_len(&baseline.preview) {
        grew.push("preview");
    }
    if now.organs.rows.len() > baseline.organs.rows.len() {
        grew.push("organs");
    }
    if now.perf.rows.len() > baseline.perf.rows.len() {
        grew.push("perf");
    }
    grew
}

/// §9's ten refusals as a build fact: the crate has no widget type at all, so
/// nothing in it can move a state byte, a door or a sleep/seed trigger — which is
/// also 05 §20's `check_no_seed_control_surface` row for this crate.
#[test]
fn the_crate_has_no_widget_that_could_move_a_state_byte() {
    for (path, code) in sources() {
        for banned in [
            "Slider",
            "Button",
            "Checkbox",
            "LineEdit",
            "TextBox",
            "set_a(",
            "reroll",
            "sleep_now",
            "forget(",
        ] {
            assert!(
                !code.contains(banned),
                "{path} carries `{banned}` — 08 §9 refuses the widget, not the button colour"
            );
        }
    }
}

/// §7's instrumentation rule: an unmeasured cell prints the em-dash sentence and
/// never a `0` that could be quoted as a measurement.
#[test]
fn an_uninstrumented_cell_is_blank_never_zero() {
    assert_eq!(
        si_vulkan::panels::Row::INSTRUMENTATION_OFF,
        "—  (instrumentation off)"
    );
    let body = r#"{"eventlog_len":4,"settle_latency_ms":null}"#;
    let lane = Lane::decode(&lane_of(&[(kind::TICK_END, body)])).expect("decode");
    let off = build(&lane, &Artifacts::default(), false);
    let values: Vec<&str> = off.perf.rows.iter().map(|row| row.value.as_str()).collect();
    assert!(
        values.contains(&si_vulkan::panels::Row::INSTRUMENTATION_OFF),
        "perf rows: {values:?}"
    );
    assert!(!values.contains(&"0"), "a timing cell printed 0 with no instrument");
    assert!(build(&lane, &Artifacts::default(), true).perf.rows.len() < off.perf.rows.len());
}

fn empty_lane() -> Lane {
    Lane::decode(&lane_of(&[])).expect("empty lane")
}

/// §8's "+ `SIst`, + `sleep.jsonl`, + `fingerprint.txt`" columns arrive as
/// [00 §8]'s hand-off: the launcher decodes them, the panel forwards. All eleven
/// registry seats appear, the reserved ones by name, so R-12's alarm cannot be
/// patched around by a pane that quietly lists eight rows.
#[test]
fn a_snapshot_arrives_forwarded_never_recomputed() {
    let artifacts = Artifacts {
        shelf_rows: (1u16..=11).map(|x| (x, x as u64)).collect(),
        bakes: vec![Row::new("z12", "bake=4096 a_at=512 thought=3")],
        core: vec![Row::new("sleeps_completed", "2")],
        sleep_lines: vec![r#"{"pass":1,"walks":4}"#.to_string()],
        fingerprint: Some("0x0000000000000051".to_string()),
        reconstructed: true,
    };
    let baseline = build(&empty_lane(), &Artifacts::default(), false);
    let now = build(&empty_lane(), &artifacts, false);
    assert_eq!(now.chat.rows.len(), baseline.chat.rows.len() + 1, "§2.2's mark");
    assert_eq!(now.emotions.rows.len(), baseline.emotions.rows.len() + 2, "E6 + its note");
    assert_eq!(
        now.organs.rows.len(),
        baseline.organs.rows.len() + artifacts.shelf_rows.len() + artifacts.core.len()
            + artifacts.sleep_lines.len()
            + 2, // the terminal-count note and the fingerprint line
        "every forwarded artifact is a row"
    );
    let keys: Vec<&str> = now.organs.rows.iter().map(|row| row.key.as_str()).collect();
    assert_eq!(keys.iter().filter(|k| k.starts_with("rows ")).count(), 11);
    assert!(keys.iter().any(|k| k.contains("reserved")), "a reserved seat renders by name");
    assert!(now
        .emotions
        .rows
        .iter()
        .any(|row| row.value == si_vulkan::panels::BAKE_NOTE));
}

/// The crate opens lane tails and the one inbox — nothing else. [08 §2]'s table
/// gives process B five more files; [00 §8]'s sentence gives `si-vulkan` the lanes
/// only, and the two hold together because the launcher hands the rest over as
/// [`Artifacts`]. A path literal outside the `.siro` set here would be the crate
/// reaching around that sentence for a lattice handle.
#[test]
fn the_crate_opens_only_lane_tails() {
    for (path, code) in sources() {
        for literal in literals(&code) {
            if literal.ends_with(".json")
                || literal.ends_with(".jsonl")
                || literal.ends_with(".txt")
                || literal.ends_with("SIst")
                || literal.ends_with("SIgen")
            {
                panic!("{path} opens `{literal}` — 00 §8: the crate's file set is B's lane tails");
            }
        }
    }
}

/// §6's O5/O11/O12 readouts forward a lane's numbers; a key the lane does not
/// carry prints `absent` rather than a `0` the run never produced.
#[test]
fn a_missing_key_prints_absent_not_zero() {
    let lane = Lane::decode(&lane_of(&[(kind::A_UPDATE, r#"{"round":1}"#)])).expect("decode");
    let panels = build(&lane, &Artifacts::default(), false);
    let values: Vec<&str> = panels.emotions.rows.iter().map(|row| row.value.as_str()).collect();
    assert!(values.iter().any(|v| v.contains("absent")), "{values:?}");
    assert!(!values.iter().any(|v| v.ends_with(" 0") || *v == "0"), "{values:?}");
}

/// The scans are not reading an empty tree: five modules, all of them real.
#[test]
fn the_scans_are_not_reading_an_empty_tree() {
    let files = sources();
    assert!(
        files.len() >= 4,
        "{:?}",
        files.iter().map(|(p, _)| p).collect::<Vec<_>>()
    );
    assert!(
        files.iter().any(|(path, _)| path.contains("panels.rs")),
        "the panel module the scan above reads"
    );
}
