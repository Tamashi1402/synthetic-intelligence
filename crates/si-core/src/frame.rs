//! engineering/12 §2.6: the frame scratch the v2 arc's readers consume.
//!
//! These are pure data declarations. `si-walk` constructs them inside one
//! thought and drops them at settle; `si-crystallize`/`si-skills` receive a
//! slice **as an argument** — neither crate may import `si-walk` (12 §1.1), so
//! the shared vocabulary lives here, and `check_ghost_is_frame_scoped` asserts
//! that no container, schema, export or `SIst`/`SIck` write ever names them.
//!
//! Nothing here is hashed (01 §10's nine items do not include it) and nothing
//! here is stored: the log rides `Thought` and reaches the harness's
//! `mindlog.json` as an observation.

/// §32's role extraction, declared once for the three readers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    /// Lit by round 0's lighting.
    Seed,
    /// First lit later and first seen as an edge source.
    LitA,
    /// First lit later, seen as an edge source after another source round.
    LitB,
    /// First lit later as an edge target — the filler a member carries.
    SharedFiller,
    /// engineering/12 §2.6's ghost: frame-scoped, destroyed at settle.
    Ghost,
}

impl Role {
    pub const fn byte(self) -> u8 {
        match self {
            Role::Seed => 0,
            Role::LitA => 1,
            Role::LitB => 2,
            Role::SharedFiller => 3,
            Role::Ghost => 4,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Role::Seed => "seed",
            Role::LitA => "lit_a",
            Role::LitB => "lit_b",
            Role::SharedFiller => "shared_filler",
            Role::Ghost => "ghost",
        }
    }
}

/// One light in one round. `edge_kind` is the incoming edge's kind for a
/// filler and `0` for a source (04 §5's spread order is the log's order).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoLight {
    pub point: u32,
    pub role: Role,
    pub edge_kind: u16,
    pub round: u8,
    pub primed: bool,
}

/// docs/22 §22.2: ≤ `GHOST_MAX` per frame, never evicted, never exported.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ghost {
    pub members: Vec<u32>,
    pub formed_in_frame: u64,
    pub frame_seed: u64,
}

/// docs/25 §25.5's shape scratch: roles present, kinds traversed, filler sets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleGraph {
    pub roles: Vec<u8>,
    pub traversed_kinds: Vec<u16>,
    pub filler_sets: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FrameLog {
    pub lights: Vec<CoLight>,
    pub ghosts: Vec<Ghost>,
    pub role_graphs: Vec<RoleGraph>,
}

impl FrameLog {
    /// The inverse of `mindlog.json`'s `frame_log[]` block (12 §6.4 G15(c)): a
    /// leg that grades the detector in isolation feeds a recorded trace back
    /// through a fresh detector. An unknown role label is a hard error — a
    /// decoder that guessed would invent a role the walk never assigned.
    pub fn from_json(value: &serde_json::Value) -> Result<Self, String> {
        let mut log = FrameLog::default();
        let lights = value["lights"].as_array().ok_or("frame log has no `lights` array")?;
        for light in lights {
            let label = light["role"].as_str().ok_or("a light has no `role` label")?;
            let role = match label {
                "seed" => Role::Seed,
                "lit_a" => Role::LitA,
                "lit_b" => Role::LitB,
                "shared_filler" => Role::SharedFiller,
                "ghost" => Role::Ghost,
                other => return Err(format!("unknown frame role `{other}`")),
            };
            log.lights.push(CoLight {
                point: light["point"].as_u64().ok_or("a light has no `point`")? as u32,
                role,
                edge_kind: light["edge_kind"].as_u64().ok_or("a light has no `edge_kind`")? as u16,
                round: light["round"].as_u64().ok_or("a light has no `round`")? as u8,
                primed: light["primed"].as_bool().unwrap_or(false),
            });
        }
        for ghost in value["ghosts"].as_array().into_iter().flatten() {
            let members = ghost["members"]
                .as_array()
                .ok_or("a ghost has no `members` array")?
                .iter()
                .map(|point| point.as_u64().map(|point| point as u32).ok_or("a ghost member is not a point"))
                .collect::<Result<Vec<u32>, &str>>()?;
            log.ghosts.push(Ghost {
                members,
                formed_in_frame: ghost["formed_in_frame"].as_u64().unwrap_or(0),
                frame_seed: ghost["frame_seed"].as_u64().unwrap_or(0),
            });
        }
        for graph in value["role_graphs"].as_array().into_iter().flatten() {
            let roles: Vec<u8> = graph["roles"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|role| role.as_u64().unwrap_or(0) as u8)
                .collect();
            let traversed_kinds: Vec<u16> = graph["traversed_kinds"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|kind| kind.as_u64().unwrap_or(0) as u16)
                .collect();
            log.role_graphs.push(RoleGraph {
                roles,
                traversed_kinds,
                filler_sets: graph["filler_sets"].as_u64().unwrap_or(0) as u16,
            });
        }
        Ok(log)
    }

    /// OD-105's cold-frame rule: a frame carrying any `primed` light is not
    /// counted by a detector's recurrence.
    pub fn is_cold(&self) -> bool {
        !self.lights.iter().any(|light| light.primed)
    }

    /// The round-0 lighting, ascending point id, deduplicated. docs/22's member
    /// test reads this set: a later round's lit point is never a member, however
    /// the walk reached it.
    pub fn seed_points(&self) -> Vec<u32> {
        let mut points: Vec<u32> = self
            .lights
            .iter()
            .filter(|light| light.role == Role::Seed && light.round == 0)
            .map(|light| light.point)
            .collect();
        points.sort_unstable();
        points.dedup();
        points
    }
}
