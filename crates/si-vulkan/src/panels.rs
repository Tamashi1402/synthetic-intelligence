//! 08 §8: the five panels, as one pure function of the lane.
//!
//! A panel is a **forward**. It takes frames and prints the numbers they carry;
//! it computes nothing over the lattice, because the viewer has no handle to a
//! lattice (§1's third law, and [02 §10]'s missing crate edges). Where a row the
//! table promises has no producer, the draw list says so in the field rather than
//! inventing a value — the same discipline §7 asks of an unmeasured cell.
//!
//! Live and replay are the same code (§2.2): both call [`Panels::build`], and only
//! the byte source differs.

use crate::frames::{Frame, Lane};
use crate::layout::{pos, shell_of, Segment};
use si_core::eventlog::kind;
use si_core::shelves::SHELVES;

/// §4/§6's rule for every drawn number: it is a **count or a grid int**, never a
/// level the owner can move. A row's `key` is chrome; its `value` is forwarded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub key: String,
    pub value: String,
}

impl Row {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self { key: key.into(), value: value.into() }
    }

    /// §7's rule for a cell whose instrument is not running: it prints this, and
    /// never a `0`, because "a blank that looks like a measurement is how an
    /// unmeasured number gets quoted".
    pub const INSTRUMENTATION_OFF: &'static str = "—  (instrumentation off)";

    pub fn unmeasured(key: impl Into<String>) -> Self {
        Row::new(key, Self::INSTRUMENTATION_OFF)
    }
}

/// One drawn node of the preview: a point, where [`crate::layout::pos`] puts it,
/// and the lit value the round actually produced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Node {
    pub handle: u32,
    pub position: [i32; 3],
    pub h_q12: i32,
    pub shelf: Option<u16>,
}

/// A panel's draw list plus the geometry a renderer needs, kept per panel so the
/// §8 table is checkable row by row.
#[derive(Clone, Debug, Default)]
pub struct Panel {
    pub title: &'static str,
    pub rows: Vec<Row>,
}

/// 08 §8's five, in the job's order.
#[derive(Clone, Debug, Default)]
pub struct Panels {
    pub chat: Panel,
    pub emotions: Panel,
    pub preview: Preview,
    pub organs: Panel,
    pub perf: Panel,
}

/// The preview is the one panel with geometry rather than only rows.
#[derive(Clone, Debug, Default)]
pub struct Preview {
    pub title: &'static str,
    pub nodes: Vec<Node>,
    pub edges: Vec<Segment>,
    pub rows: Vec<Row>,
}

/// What each panel reads, straight from §8's table. It is data, so `tests/laws.rs`
/// can assert a panel never draws a kind its row does not list. The 20 rows are
/// §2.1's sealed table minus row 18, which has a name and no producer (**OD-67**).
pub const READS: [(u8, &'static [&'static str]); 20] = [
    (kind::SENSE_ECHO, &["chat"]),
    (kind::DECODED, &["chat", "organs"]),
    (kind::LIT_ROUND0, &["preview"]),
    (kind::ROUND, &["chat", "emotions", "preview"]),
    (kind::SETTLE, &["preview", "organs"]),
    (kind::SIGNAL_EVENT, &["emotions"]),
    (kind::A_UPDATE, &["emotions"]),
    (kind::MEDIAN_USED, &["emotions", "preview"]),
    (kind::GAP, &["chat", "organs", "preview"]),
    (kind::DOOR_CALL, &["organs"]),
    (kind::ASK, &["chat", "organs"]),
    (kind::LABEL, &["chat", "emotions", "organs"]),
    (kind::OUTPUT, &["chat"]),
    (kind::COMMIT, &["chat", "emotions", "organs", "preview"]),
    (kind::SUPERSEDE, &["organs"]),
    (kind::STAGE, &["organs"]),
    (kind::TICK_END, &["organs", "perf"]),
    (kind::SEAM_REJECT, &["chat", "organs"]),
    (kind::SLEEP_PASS, &["organs", "perf"]),
    (kind::TEMPERAMENT_READOUT, &["organs"]),
];

/// §9's table, as the list the build is missing rather than a review note: there
/// is no widget in this crate that can move a state byte, because the only draw
/// primitive here is text and the only geometry is [`crate::layout::pos`].
pub const FORBIDDEN_WIDGETS: [(&str, &str); 10] = [
    ("a slider or setter over `a`, a gain, a threshold or the curve", "I8, R-21, R-34"),
    ("edit, delete, merge or forget of a row or edge", "I4, 08 §5.3 law (c)"),
    ("promote-this-inference-to-knowledge", "I2, 04 §21 U.4, OD-58"),
    ("send-this-line-as-supporter", "I3, I10, R-35, G12(e)"),
    ("live counterfactual probe", "R-03, 04 §15, OD-67"),
    ("a sweep artifact displayed as a setting", "R-33"),
    ("sleep now", "I3, I5, OD-78/79/81, 10 §7"),
    ("re-roll the temperament", "I5, G14(b)(c)(j), 10 §3/§5"),
    ("dim-this-edge or a weight field", "04 §26 law 25, G13(b)(d), 10 §6"),
    ("any write except the one inbox file", "OD-48, 08 §1's OBSERVER-ONLY"),
];

/// §2.2: a viewer that starts mid-session marks what it did not watch.
pub const RECONSTRUCTED: &str = "reconstructed from snapshot";

/// The one law the ORGANS panel prints next to its own number (§6 O4), because
/// the panel's whole value is that a contradiction shows up on screen.
pub const DOOR_INVARIANT: &str = "the walk makes exactly two door calls";

/// §6 E6: the bake is durable and bounded, and it is **not** a read-time
/// multiplier. The panel that shows a big bake number has to say which of the two
/// it is looking at, or it misleads the reviewer it is meant to inform.
pub const BAKE_NOTE: &str = "write-once at formation; not a read-time multiplier (20 §20.4a)";

/// §6/§8's non-lane sources, **already decoded and handed over**.
///
/// This is where the two file laws meet: [08 §2]'s table has process B reading
/// `SIst`, `mindlog.json`, `sleep.jsonl` and `fingerprint.txt`, while
/// [00 §8]'s sentence says the only files `si-vulkan` may open are B's own lane
/// tails. Both hold if the *launcher* (`si-cli view`, which is allowed the schema
/// crates) decodes them and passes numbers in, because what §8's rows ask of the
/// panel is a **count or a forwarded line**, never a lattice read (§1's third law).
/// A panel that opened `SIst` here would need `si-store`, and 02 §10's edge does
/// not exist.
#[derive(Clone, Debug, Default)]
pub struct Artifacts {
    /// The snapshot's row counts per shelf, **including the reserved seats** —
    /// §6 O2's rule that a reserved shelf renders rather than being skipped.
    pub shelf_rows: Vec<(u16, u64)>,
    /// `SIst`'s per-edge bake list (§6 E6): `bake`, `a_at`, `thought_id` and the
    /// write-once note, as rows the loop's own decoder produced.
    pub bakes: Vec<Row>,
    /// The core block's counters ([01 §13] item 9): the sleep counters, the
    /// quiet-tick and ignition counts, the derived-vector digest.
    pub core: Vec<Row>,
    /// `sleep.jsonl`'s lines, forwarded verbatim (§6 O11: one block per sleep).
    pub sleep_lines: Vec<String>,
    /// `fingerprint.txt`'s line, forwarded verbatim (§6 O12, §11's refusal to
    /// re-derive: "the panel does not re-derive; it forwards kind 20").
    pub fingerprint: Option<String>,
    /// §2.2: a viewer that opened mid-session marks everything before this point.
    pub reconstructed: bool,
}

/// Build all five from one lane tail plus the launcher's decoded artifacts.
///
/// `profile` is §7's switch: when the loop's sidecar is off, every timing cell
/// prints [`Row::INSTRUMENTATION_OFF`] instead of a number the run never took.
pub fn build(lane: &Lane, artifacts: &Artifacts, profile: bool) -> Panels {
    let mut panels = Panels {
        chat: Panel { title: "CHAT", rows: Vec::new() },
        emotions: Panel { title: "EMOTIONS", rows: Vec::new() },
        preview: Preview { title: "TESSERACT PREVIEW", ..Default::default() },
        organs: Panel { title: "ORGANS STATUS", rows: Vec::new() },
        perf: Panel { title: "VRAM/PERF", rows: Vec::new() },
    };
    for frame in &lane.frames {
        draw(&mut panels, frame, profile);
    }
    // §6 O4's invariant line is printed once, under the counts it belongs to.
    panels.organs.rows.push(Row::new("invariant", DOOR_INVARIANT));
    // §6 O10: the monologue pane exists and is blank until OD-38 rules.
    panels.organs.rows.push(Row::new("monologue", "present, unread (OD-38 pending)"));
    draw_artifacts(&mut panels, artifacts);
    panels
        .perf
        .rows
        .push(Row::new("frames_behind", lane.frames.len().to_string()));
    panels
}

/// The non-lane half of §8's read list: the snapshot's counts, the sleep lines,
/// the fingerprint. All of it **forwarded**, because a panel that computed a row
/// count by walking an arena would be the re-simulation §1's third law names.
fn draw_artifacts(panels: &mut Panels, artifacts: &Artifacts) {
    if artifacts.reconstructed {
        panels.chat.rows.push(Row::new("history", RECONSTRUCTED));
    }
    // O2 KNOW: every shelf seat listed, reserved ones named as reserved in the
    // key itself, so the count identity is on the screen and not in a footnote.
    for (shelf_x, count) in &artifacts.shelf_rows {
        panels.organs.rows.push(Row::new(
            format!("rows {}", crate::layout::shelf_label(*shelf_x)),
            count.to_string(),
        ));
    }
    for row in &artifacts.core {
        panels.organs.rows.push(Row::new(row.key.clone(), row.value.clone()));
    }
    // O11 SLEEP: the pass records, and the standing note that P4's counts are
    // terminal — the panel prints the law next to the number (§6's wording).
    for line in &artifacts.sleep_lines {
        panels.organs.rows.push(Row::new("sleep pass", line.clone()));
    }
    if !artifacts.sleep_lines.is_empty() {
        panels.organs.rows.push(Row::new(
            "calm view",
            "terminal: feeds no weight, floor, d or sampler (04 §26 law 26)",
        ));
    }
    // O12 TEMPERAMENT: the fingerprint line, never re-derived here.
    if let Some(line) = &artifacts.fingerprint {
        panels.organs.rows.push(Row::new("fingerprint line", line.clone()));
    }
    // E6 BAKE: write-once, and the panel says which of the two it is showing.
    for row in &artifacts.bakes {
        panels
            .emotions
            .rows
            .push(Row::new(format!("bake {}", row.key), row.value.clone()));
    }
    if !artifacts.bakes.is_empty() {
        panels.emotions.rows.push(Row::new("bake law", BAKE_NOTE));
    }
}

/// One frame into the panels that read its kind (§8's table is the dispatch).
fn draw(panels: &mut Panels, frame: &Frame, profile: bool) {
    let t = frame.t.to_string();
    match frame.kind {
        kind::SENSE_ECHO => panels.chat.rows.push(Row::new(
            format!("line {t} {}", source_tag(frame.source)),
            frame.text("line").unwrap_or_else(|| "(no line)".to_string()),
        )),
        kind::DECODED => {
            panels.chat.rows.push(Row::new(format!("decoded {t}"), frame.body.clone()));
            panels.organs.rows.push(Row::new("decoded", frame.body.clone()));
        }
        kind::LIT_ROUND0 => {
            for (handle, value) in pairs(frame, "lit") {
                panels.preview.nodes.push(Node {
                    handle,
                    position: pos(handle, None),
                    h_q12: value,
                    shelf: None,
                });
            }
        }
        kind::ROUND => {
            for (handle, value) in pairs(frame, "lit_set") {
                push_node(&mut panels.preview, handle, value);
            }
            for edge in edge_rows(frame) {
                panels.preview.edges.push(edge);
            }
            // E3: the modulation triple actually in force, next to the round's `a`.
            panels.emotions.rows.push(Row::new(
                format!("round {} a", number(frame, "k")),
                format!(
                    "a_u8={} reach={} decay={} tau_shift={}",
                    number(frame, "a_used_u8"),
                    nested(frame, "modulation", "reach"),
                    nested(frame, "modulation", "decay_q12"),
                    nested(frame, "modulation", "tau_shift_q12"),
                ),
            ));
        }
        kind::SETTLE => {
            panels.preview.rows.push(Row::new(
                format!("stop {}", number(frame, "thought")),
                frame.text("stop").unwrap_or_default(),
            ));
            panels.organs.rows.push(Row::new(
                format!("settle {}", number(frame, "thought")),
                pairs(frame, "settle").len().to_string(),
            ));
        }
        kind::SIGNAL_EVENT => {
            // E2: the six signals as **numbers**. No name reaches this crate.
            let rows = frame
                .json()
                .ok()
                .and_then(|v| v["rows"].as_array().cloned())
                .unwrap_or_default();
            if rows.is_empty() {
                panels.emotions.rows.push(Row::new(
                    format!("signals {t}"),
                    format!("count {} (no rows on the lane)", number(frame, "signals")),
                ));
            }
            for (index, row) in rows.iter().enumerate() {
                let citation = row
                    .get("citation")
                    .and_then(|node| node.as_i64())
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| format!("{t}/thought {}", number(frame, "thought")));
                panels.emotions.rows.push(Row::new(
                    format!("signal {} @ {index}", child(&row, "signal")),
                    format!(
                        "magnitude_q12={} cited {citation}",
                        child(&row, "magnitude_q12")
                    ),
                ));
            }
            // E1: the trajectory point, with the round it was *applied* in.
            panels.emotions.rows.push(Row::new(
                format!("a_after {t}"),
                format!("{} applies_at {}", number(frame, "a_after"), number(frame, "applies_at")),
            ));
        }
        kind::A_UPDATE => panels.emotions.rows.push(Row::new(
            format!("round {} a", number(frame, "round")),
            format!(
                "used={} after={} applies_at={}",
                number(frame, "a_used"),
                number(frame, "a_after"),
                number(frame, "applies_at")
            ),
        )),
        kind::MEDIAN_USED => {
            // E5: the median on the record, log only. `n_stamps` is forwarded as
            // the lane wrote it, and a `null` prints as absent — **OD-144** says
            // 04 §9's H has no producer, so a printed `0` would be a measurement
            // this build never made.
            let rows = frame
                .json()
                .ok()
                .and_then(|v| v["rows"].as_array().cloned())
                .unwrap_or_default();
            for row in rows {
                let stamps = row
                    .get("n_stamps")
                    .and_then(|node| node.as_i64())
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "absent".to_string());
                panels.emotions.rows.push(Row::new(
                    format!("median point {}", child(&row, "point")),
                    format!("q12={} n_stamps={stamps}", child(&row, "median_q12")),
                ));
            }
            if let Some(producer) = frame.text("producer") {
                panels.emotions.rows.push(Row::new("median producer", producer));
            }
        }
        kind::GAP => {
            panels.chat.rows.push(Row::new(format!("gap {t}"), frame.body.clone()));
            panels.organs.rows.push(Row::new("gap", frame.body.clone()));
        }
        kind::DOOR_CALL => {
            let door = frame.text("door").unwrap_or_default();
            let slot = format!("door {door}");
            let count = panels
                .organs
                .rows
                .iter()
                .position(|row| row.key == slot)
                .map(|index| {
                    let prior: i64 = panels.organs.rows[index]
                        .value
                        .parse()
                        .unwrap_or_default();
                    panels.organs.rows.remove(index);
                    (prior + 1).to_string()
                })
                .unwrap_or_else(|| "1".to_string());
            panels.organs.rows.push(Row::new(slot, count));
        }
        kind::ASK => {
            panels.chat.rows.push(Row::new(format!("ask {t}"), frame.body.clone()));
            panels.organs.rows.push(Row::new("ask", frame.body.clone()));
        }
        kind::LABEL => {
            // E4 / O7: the string arrives as a payload and is forwarded verbatim.
            let label = frame.text("label").unwrap_or_default();
            panels.chat.rows.push(Row::new(format!("label {t}"), label.clone()));
            panels.emotions.rows.push(Row::new(format!("label {t}"), label));
        }
        kind::OUTPUT => {
            let surface = frame.text("surface").unwrap_or_default();
            let tag = frame.text("tag").unwrap_or_default();
            panels.chat.rows.push(Row::new(
                format!("answer {t} [{tag}]"),
                format!("{surface} — {}", frame.text("label").unwrap_or_default()),
            ));
        }
        kind::COMMIT => {
            panels.chat.rows.push(Row::new(format!("commit {t}"), frame.body.clone()));
            panels.organs.rows.push(Row::new("commit", frame.body.clone()));
        }
        kind::SUPERSEDE => {
            panels.organs.rows.push(Row::new("supersede", frame.body.clone()));
        }
        kind::STAGE => {
            panels.organs.rows.push(Row::new("stage", frame.body.clone()));
        }
        kind::TICK_END => {
            panels.organs.rows.push(Row::new("tick", frame.body.clone()));
            panels.perf.rows.push(Row::new(
                "eventlog_len",
                number(frame, "eventlog_len"),
            ));
            if !profile {
                panels.perf.rows.push(Row::unmeasured("settle_latency"));
                panels.perf.rows.push(Row::unmeasured("sync_count"));
            }
        }
        kind::SEAM_REJECT => {
            // §3: a rejected emission is a visible line, because a silently
            // dropped voice would hide the whole of docs/21.
            panels.chat.rows.push(Row::new(format!("rejected {t}"), frame.body.clone()));
            panels.organs.rows.push(Row::new("seam_reject", frame.body.clone()));
        }
        kind::SLEEP_PASS => {
            panels.organs.rows.push(Row::new("sleep", frame.body.clone()));
            panels.perf.rows.push(Row::new("sleep_pass", frame.body.clone()));
        }
        kind::TEMPERAMENT_READOUT => {
            // O12: seed, box-relative value per row, digest — and the pane's own
            // two refusals, printed on the pane (§6's wording).
            panels.organs.rows.push(Row::new("birth_seed", frame.text("birth_seed").unwrap_or_default()));
            panels.organs.rows.push(Row::new("seed_format", frame.text("seed_format").unwrap_or_default()));
            panels.organs.rows.push(Row::new("fingerprint", frame.text("temperament_digest").unwrap_or_default()));
            for row in frame
                .json()
                .ok()
                .and_then(|v| v["rows"].as_array().cloned())
                .unwrap_or_default()
            {
                panels.organs.rows.push(Row::new("temperament row", row.to_string()));
            }
            panels.organs.rows.push(Row::new("refusal", "no nickname: a classification is si-audit's string"));
            panels.organs.rows.push(Row::new("refusal", "no explanation: the mindlog accounts for behavior"));
        }
        other => panels.organs.rows.push(Row::new("kind", format!("{other} (no panel row)"))),
    }
}

/// A node the preview has not seen yet, or the same node at its new intensity.
fn push_node(preview: &mut Preview, handle: u32, value: i32) {
    if let Some(node) = preview.nodes.iter_mut().find(|node| node.handle == handle) {
        node.h_q12 = value;
        return;
    }
    preview.nodes.push(Node { handle, position: pos(handle, None), h_q12: value, shelf: None });
}

/// `[(point, q12)]` out of a body's named array, as the loop wrote it.
fn pairs(frame: &Frame, field: &str) -> Vec<(u32, i32)> {
    let Ok(value) = frame.json() else { return Vec::new() };
    let Some(array) = value[field].as_array() else { return Vec::new() };
    array
        .iter()
        .filter_map(|entry| {
            let point = entry["point"].as_u64()? as u32;
            let q12 = entry["q12"].as_i64().or_else(|| entry["h_q12"].as_i64())? as i32;
            Some((point, q12))
        })
        .collect()
}

/// kind 3's `new_edges`, which carry OWNED vs BASE as the `base` bit the walk paid.
fn edge_rows(frame: &Frame) -> Vec<Segment> {
    let Ok(value) = frame.json() else { return Vec::new() };
    let Some(array) = value["new_edges"].as_array() else { return Vec::new() };
    array
        .iter()
        .filter_map(|entry| {
            Some(Segment {
                from: entry["from"].as_u64()? as u32,
                to: entry["to"].as_u64()? as u32,
                weight_q12: entry["weight_eff_q12"].as_i64()? as i32,
                owned: entry["base"].as_i64()? == 0,
            })
        })
        .collect()
}

/// A drawn number. `absent` prints rather than a `0`, because a key the lane does
/// not carry is not a key whose value is zero — §7's instrumentation rule applied
/// to payload fields.
fn number(frame: &Frame, field: &str) -> String {
    frame.int(field).map(|value| value.to_string()).unwrap_or_else(|| "absent".to_string())
}

fn nested(frame: &Frame, object: &str, field: &str) -> String {
    frame
        .json()
        .ok()
        .and_then(|v| v[object][field].as_i64())
        .map(|value| value.to_string())
        .unwrap_or_else(|| "absent".to_string())
}

fn child(value: &serde_json::Value, field: &str) -> String {
    match value.get(field) {
        Some(node) if node.is_null() => "null".to_string(),
        Some(node) => node.to_string(),
        None => "absent".to_string(),
    }
}

/// The source byte as the panel's own two marks — owner or supporter, minted by
/// the module that received the line ([00 §7]), never inferred from the text.
pub fn source_tag(source: u8) -> &'static str {
    match source {
        // 08 §3: "supporter lines visibly marked `grimm`". The mark is chrome, and
        // it names who minted the byte — [00 §7] gives that to the seam, never to
        // this process (§9's row: a "send this line as grimm" toggle is refused).
        crate::SOURCE_OWNER => "owner",
        crate::SOURCE_SUPPORTER => "grimm",
        _ => "unknown-source",
    }
}

/// The shell geometry the preview draws from: one band per registry shelf,
/// **including the reserved rows**, because §6 O2's rule is that a reserved shelf
/// renders as `reserved, v2` rather than being skipped — R-12's alarm may not be
/// patched around by a pane that lists eight shelves where the registry has eleven.
pub fn shells() -> Vec<(u16, String, i32)> {
    SHELVES
        .iter()
        .map(|entry| {
            (entry.shelf_id, crate::layout::shelf_label(entry.shelf_id), shell_of(entry.shelf_id))
        })
        .collect()
}
