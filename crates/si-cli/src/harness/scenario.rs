//! 05 §1/§22: a checked-in scenario is a literal transcript, hashed. The
//! `--- cold` marker starts a second fresh session inside the same gate run
//! (05 §6's teach block is one), and `idle` is a quiet tick. The seed, when the
//! file states one, must equal the launch seed: a scenario names its individual,
//! and the file's hash pins the name.
//!
//! From B18 the grammar grows the world's four declarations (12 §5 §33, 11 §B18
//! break-list row 10): an `identity <u64> owner|grimm` line names the two
//! registered ids, a `voice <id>:` prefix is the third sense form — the two
//! existing literals with a different identity — and `grant`/`revoke`/`window`
//! are steps a run carries out as it goes. They are built from the two literals
//! and the null identity, so a scenario that declares none of them parses to
//! exactly the steps it parsed to before.

use anyhow::{bail, Context, Result};
use md5::{Digest, Md5};
use si_core::strings::Source;
use si_core::world::{GrantRow, Identity, Right, Window, IDENTITY_OWNER};

use super::Workspace;

/// 11 §B18: one step. `identity` is `None` for the two literal forms — the
/// runner then uses the id registered for that literal, which is what keeps every
/// pre-B18 scenario's class derivation identical.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    Sense {
        source: Source,
        text: String,
        identity: Option<Identity>,
        conversation: u32,
    },
    Idle,
    /// The world's three in-run declarations. They mutate session RAM only: no
    /// step here writes a row, and `check_windows_suppress_only` reads this arm.
    World(WorldOp),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldOp {
    /// mint a grant (the owner's act; `granted_by` is the owner's registered id)
    Grant { subject: Identity, right: Right },
    Revoke { subject: Identity, right: Right },
    /// the declared idle window, and `None` removes it
    Window(Option<Window>),
}

/// The two registered ids as the file declares them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Identities {
    pub owner: Identity,
    pub grimm: Identity,
}

impl Default for Identities {
    fn default() -> Self {
        Self { owner: IDENTITY_OWNER, grimm: 1 }
    }
}

#[derive(Clone, Debug)]
pub struct Scenario {
    /// Workspace-relative path, for the report.
    pub path: String,
    pub digest: String,
    pub seed: Option<u64>,
    pub sections: Vec<Vec<Step>>,
    /// The `identity …` declarations, in file order. Empty means the birth
    /// registration (owner `0`, grimm `1`).
    pub identities: Vec<(Identity, Source)>,
}

impl Scenario {
    pub fn load(ws: &Workspace, relative: &str) -> Result<Self> {
        let bytes = std::fs::read(ws.path(relative))
            .with_context(|| format!("read scenario {relative}"))?;
        let digest = hex(Md5::digest(&bytes).as_slice());
        let text = String::from_utf8(bytes).with_context(|| format!("scenario {relative} is not UTF-8"))?;
        Self::parse(relative, &text, Some(digest))
    }

    /// The two registered ids a file declares, or the birth pair.
    pub fn registered(&self) -> Identities {
        let mut out = Identities::default();
        for (identity, source) in &self.identities {
            match source {
                Source::Owner => out.owner = *identity,
                Source::Grimm => out.grimm = *identity,
            }
        }
        out
    }

    pub fn parse(path: &str, text: &str, digest: Option<String>) -> Result<Self> {
        let mut seed = None;
        let mut identities: Vec<(Identity, Source)> = Vec::new();
        let mut sections: Vec<Vec<Step>> = vec![Vec::new()];
        for (number, raw) in text.lines().enumerate() {
            let line = raw.trim_end_matches('\r');
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("seed ") {
                if !sections[0].is_empty() || sections.len() > 1 {
                    bail!("{path}:{}: `seed` belongs at the top of the file", number + 1);
                }
                seed = Some(parse_seed(rest.trim())?);
                continue;
            }
            // 11 §B18: `identity <u64> owner|grimm`, a declaration, never a step.
            if let Some(rest) = trimmed.strip_prefix("identity ") {
                if !sections[0].is_empty() || sections.len() > 1 {
                    bail!(
                        "{path}:{}: `identity` belongs at the top of the file, beside `seed`",
                        number + 1
                    );
                }
                let (identity, source) = parse_identity(&format!("{path}:{}:", number + 1), rest)?;
                identities.push((identity, source));
                continue;
            }
            // The three in-run world declarations. `grant`/`revoke`/`window` are
            // steps because they take effect where the file puts them.
            if let Some(rest) = trimmed.strip_prefix("grant ") {
                let (subject, right) = parse_right(path, number + 1, rest)?;
                sections
                    .last_mut()
                    .expect("a section exists")
                    .push(Step::World(WorldOp::Grant { subject, right }));
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("revoke ") {
                let (subject, right) = parse_right(path, number + 1, rest)?;
                sections
                    .last_mut()
                    .expect("a section exists")
                    .push(Step::World(WorldOp::Revoke { subject, right }));
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("window ") {
                let window = parse_window(path, number + 1, rest)?;
                sections
                    .last_mut()
                    .expect("a section exists")
                    .push(Step::World(WorldOp::Window(window)));
                continue;
            }
            if trimmed == "idle" {
                sections.last_mut().expect("a section exists").push(Step::Idle);
                continue;
            }
            if trimmed == "--- cold" {
                if sections.last().is_some_and(|section| section.is_empty()) {
                    bail!("{path}:{}: empty section", number + 1);
                }
                sections.push(Vec::new());
                continue;
            }
            // `voice <id>: …` — the third sense form, built from the existing two
            // literals: it is a non-owner line (`Source::Grimm` reaches the gate as
            // an inert input) carrying a different registered identity.
            if let Some(rest) = trimmed.strip_prefix("voice ") {
                let (id_text, line_text) = rest.split_once(':').ok_or_else(|| {
                    anyhow::anyhow!("{path}:{}: `voice <id>:` needs a colon", number + 1)
                })?;
                let identity: Identity = id_text
                    .trim()
                    .parse()
                    .with_context(|| format!("{path}:{}: `{}` is not a u64 id", number + 1, id_text.trim()))?;
                let line_text = line_text.trim();
                if line_text.is_empty() {
                    bail!("{path}:{}: empty sense line", number + 1);
                }
                sections.last_mut().expect("a section exists").push(Step::Sense {
                    source: Source::Grimm,
                    text: line_text.to_string(),
                    identity: Some(identity),
                    conversation: 0,
                });
                continue;
            }
            let (source, text) = if let Some(rest) = trimmed.strip_prefix("owner:") {
                (Source::Owner, rest.trim())
            } else if let Some(rest) = trimmed.strip_prefix("grimm:") {
                (Source::Grimm, rest.trim())
            } else {
                bail!(
                    "{path}:{}: a step is `owner: …`, `grimm: …`, `voice <id>: …`, `idle` or \
                     `--- cold`",
                    number + 1
                );
            };
            if text.is_empty() {
                bail!("{path}:{}: empty sense line", number + 1);
            }
            sections.last_mut().expect("a section exists").push(Step::Sense {
                source,
                text: text.to_string(),
                identity: None,
                conversation: 0,
            });
        }
        let sections: Vec<Vec<Step>> =
            sections.into_iter().filter(|section| !section.is_empty()).collect();
        if sections.is_empty() {
            bail!("{path}: the scenario has no steps");
        }
        Ok(Self {
            path: path.to_string(),
            digest: digest.unwrap_or_default(),
            seed,
            sections,
            identities,
        })
    }

    pub fn steps(&self) -> usize {
        self.sections.iter().map(Vec::len).sum()
    }

    /// The grant rows the file's `grant`/`revoke` steps mint, in order, against
    /// the owner id the file registers. `gates/G19/grant_trace.json` is this
    /// reading plus the clock each step ran at.
    pub fn grant_rows(&self, owner: Identity) -> Vec<GrantRow> {
        let mut rows: Vec<GrantRow> = Vec::new();
        let mut t = 0i32;
        for section in &self.sections {
            for step in section {
                match step {
                    Step::World(WorldOp::Grant { subject, right }) => rows.push(GrantRow {
                        subject: *subject,
                        right: *right,
                        granted_by: owner,
                        granted_at_t: t,
                        revoked_at_t: None,
                    }),
                    Step::World(WorldOp::Revoke { subject, right }) => {
                        for row in rows.iter_mut().rev() {
                            if row.subject == *subject && row.right == *right && row.live_at(t) {
                                row.revoked_at_t = Some(t);
                                break;
                            }
                        }
                    }
                    Step::Sense { .. } | Step::Idle | Step::World(WorldOp::Window(_)) => t += 1,
                }
            }
        }
        rows
    }
}

/// 11 §B18: the `identity <u64> owner|grimm` declaration, read from the words
/// after the keyword. `Scenario::parse` calls it for a file, and the replay
/// calls it for a run's `#identity` header: a header is a declaration and by
/// design carries no steps, so it cannot go through the whole-file parser and
/// its `no steps` guard. One grammar for both, never a second dialect.
pub fn parse_identity(where_: &str, rest: &str) -> Result<(Identity, Source)> {
    let mut words = rest.split_whitespace();
    let digits = words
        .next()
        .ok_or_else(|| anyhow::anyhow!("{where_} `identity` names no id"))?;
    let identity: Identity = digits
        .parse()
        .with_context(|| format!("{where_} `{digits}` is not a u64 id"))?;
    let literal = words
        .next()
        .ok_or_else(|| anyhow::anyhow!("{where_} `identity` names no voice"))?;
    let source = Source::from_literal(literal)
        .ok_or_else(|| anyhow::anyhow!("{where_} `{literal}` is not one of the two voice literals"))?;
    if words.next().is_some() {
        bail!("{where_} `identity` takes an id and one voice literal");
    }
    Ok((identity, source))
}

fn parse_right(path: &str, line: usize, rest: &str) -> Result<(Identity, Right)> {
    let mut words = rest.split_whitespace();
    let digits = words
        .next()
        .ok_or_else(|| anyhow::anyhow!("{path}:{line}: `grant`/`revoke` names no subject"))?;
    let subject: Identity = digits
        .parse()
        .with_context(|| format!("{path}:{line}: `{digits}` is not a u64 id"))?;
    let word = words
        .next()
        .ok_or_else(|| anyhow::anyhow!("{path}:{line}: `grant`/`revoke` names no right"))?;
    // docs/26 §26.3(b)'s three-word menu, single-sourced from si-core::world.
    let right = Right::from_literal(word).ok_or_else(|| {
        anyhow::anyhow!("{path}:{line}: `{word}` is not one of teach|correct|answer")
    })?;
    if words.next().is_some() {
        bail!("{path}:{line}: `grant`/`revoke` takes a subject and one right");
    }
    Ok((subject, right))
}

fn parse_window(path: &str, line: usize, rest: &str) -> Result<Option<Window>> {
    let rest = rest.trim();
    if rest == "close" || rest == "off" {
        return Ok(None);
    }
    let mut words = rest.split_whitespace();
    let from: i32 = words
        .next()
        .ok_or_else(|| anyhow::anyhow!("{path}:{line}: `window` names no opening tick"))?
        .parse()
        .context("window opening tick")?;
    let through: i32 = words
        .next()
        .ok_or_else(|| anyhow::anyhow!("{path}:{line}: `window` names no closing tick"))?
        .parse()
        .context("window closing tick")?;
    if words.next().is_some() {
        bail!("{path}:{line}: `window` takes two logical ticks (OD-126: the axis is the tick, never a wall clock)");
    }
    Ok(Some(Window { open_from_t: from, open_through_t: through }))
}

pub fn parse_seed(text: &str) -> Result<u64> {
    let trimmed = text.trim();
    let (digits, radix) = match trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
        Some(hex) => (hex, 16),
        None => (trimmed, 10),
    };
    u64::from_str_radix(digits, radix)
        .map_err(|error| anyhow::anyhow!("`{text}` is not a u64 seed: {error}"))
}

pub fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}
