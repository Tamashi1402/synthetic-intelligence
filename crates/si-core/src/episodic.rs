//! docs/24 §24.2 / engineering/04 §31 AC.1–AC.2, 12 §2.6: the event view and
//! the derived origin tug.
//!
//! Neither type is stored. `event_id` **is** the `context` row's `y`, so there is
//! no new id space, no new shelf and no `prov-link` row (OD-110 ships derived);
//! `SHELF_EVENTS` stays reserved at `row_schema_version = 0`. The index and the
//! views are assembled at frame load from bytes the write path already laid, and
//! their types live here so `si-read`, `si-walk` and the harness name one shape.

/// One derived provenance edge: a belief row's live level, read over the
/// `context` row that formed it.
///
/// `points` are the belief row's nonzero endpoints, ascending (a belief may be a
/// relation over one point, so never assume two). `weight_q12` is
/// `mul_q12(PROV_LINK_TUG, level_of_belief)` computed by the index builder, so
/// the walk applies it without reading a constant of its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OriginTug {
    /// The `context` row's `y` — 04 §31 AC.1's one id space.
    pub event_id: u32,
    pub points: Vec<u32>,
    pub weight_q12: u16,
}

/// 12 §2.6's event view: one per sense line, assembled at frame load, hashed
/// nowhere — every field is already in the hash's items 1–3.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct EventView {
    /// The sense row's `y`; the same value `OriginTug::event_id` carries.
    pub event_id: u32,
    /// Live belief rows whose `(prov_session, prov_step)` names this line. Empty
    /// for a `grimm` line: his moments have views and no incoming belief link
    /// (docs/24 §24.4, G17(e)).
    pub belief_rows: Vec<u32>,
    /// The stamp-arena offsets the commit laid, ascending by arena order.
    pub stamps: Vec<u32>,
    /// The `a` those stamps recorded at write time.
    pub a_at: u8,
    /// The line's tick.
    pub t: i32,
    /// OD-123's field: a framed identity, absent (`0`) until B18 frames one.
    pub speaker_identity: u64,
}
