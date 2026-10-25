//! 04 §4 C.1–C.6: round-0 lighting.
//!
//! One pulse per token in word order (`PHRASE_BATCHING` is off, a LAW): the
//! frame hyperpoint first when a frame matched, then each token's term
//! hyperpoint at the round-0 quantum. A token with no hyperpoint fires no
//! lighting and is recorded as a novelty event (04 §4.2); an alias set lights
//! **every** member with the shared activation, never one member silently
//! (OD-55, R-04).

use si_walk::{Lit, LIGHT_QUANTUM};

/// The session's handle lookup. `aliases` returns every hyperpoint whose folded
/// key equals `key`: one member is the ordinary case, several are R2's alias
/// candidates. An empty answer means the token lands where nothing exists.
pub trait Resolve {
    fn aliases(&self, key: &str) -> Vec<u32>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LightRequest {
    /// Folded term keys, in word order.
    pub keys: Vec<String>,
    /// The frame hyperpoint, when the input decoded to one.
    pub frame_point: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lighting {
    pub lit: Vec<Lit>,
    /// Indices into `keys` that lit nothing (04 §4.2's novelty events).
    pub novel: Vec<usize>,
    /// `(token index, every member lit)`, for the trace's alias note (OD-55).
    pub aliases: Vec<(usize, Vec<u32>)>,
}

pub fn light(request: &LightRequest, resolve: &dyn Resolve) -> Lighting {
    let mut lit = Vec::new();
    let mut novel = Vec::new();
    let mut aliases = Vec::new();

    // 04 §4.1: order must prime — the frame lights before the content arrives.
    if let Some(point) = request.frame_point {
        lit.push(Lit { point, activation: LIGHT_QUANTUM });
    }
    for (index, key) in request.keys.iter().enumerate() {
        let members = resolve.aliases(key);
        if members.is_empty() {
            novel.push(index);
            continue;
        }
        // Reciprocal multiply on the grid: the token's quantum is shared, so the
        // alias set carries the same total activation as a single member.
        let share = LIGHT_QUANTUM / members.len() as i32;
        for member in &members {
            lit.push(Lit { point: *member, activation: share });
        }
        aliases.push((index, members));
    }
    Lighting { lit, novel, aliases }
}
