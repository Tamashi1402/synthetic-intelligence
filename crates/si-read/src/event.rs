//! 04 §31 AC.1 / docs/24 §24.2: step 5b — the derived event-view index.
//!
//! One pass over committed rows' provenance at frame load. No row is appended,
//! no edge laid, no id minted: the event id **is** the `context` row's `y`, and
//! the belief side is the ordinary commit's `(prov_session, prov_step)`. The
//! index is rebuilt every frame and never stored (OD-110's shipped half), so the
//! gate can delete it with a grep instead of a migration.
//!
//! `weight = mul_q12(PROV_LINK_TUG, level_of_belief)`, where the belief's level is
//! the largest weight among its live owned edges — the same number the walk would
//! conduct with. It is read, never modulated by `a` (AC.1).

use std::collections::BTreeMap;

use anyhow::Result;

use si_core::addr::{SHELF_CONTEXT, SHELF_LINKS, SHELF_S_R_PAIRS};
use si_core::episodic::{EventView, OriginTug};
use si_core::quant::mul_q12_u16;
use si_core::record::{Record, STAMP_BYTES};
use si_store::store::Lattice;

/// 01 §2: the three L3 belief shelves. The reserved shelves hold nothing by
/// construction, and the `context` shelf is the event side of the join.
fn is_belief_shelf(shelf: u16) -> bool {
    (SHELF_S_R_PAIRS..=SHELF_LINKS).contains(&shelf)
}

/// The `context` rows' `(prov_session, prov_step)` handle — the only join key the
/// write path leaves behind. `owner_only` is §24.4: a `grimm` moment has a view
/// and no incoming belief link.
fn events_of(lattice: &Lattice, anchor: i32, owner_only: bool) -> Result<BTreeMap<(u32, i32), u32>> {
    let mut out = BTreeMap::new();
    for (_, record) in lattice.rows()? {
        let envelope = record.envelope;
        if envelope.shelf_x != SHELF_CONTEXT || !envelope.active_at(anchor) {
            continue;
        }
        if owner_only && envelope.prov_speaker != 0 {
            continue;
        }
        out.insert((envelope.prov_session, envelope.prov_step), envelope.row_id);
    }
    Ok(out)
}

/// The belief's live level: the largest weight among its live owned edges. A row
/// with no live edge has no level, and `0` is the absent sentinel — such a row
/// contributes no tug rather than a fabricated one.
fn level_of_belief(lattice: &Lattice, record: &Record) -> Result<u16> {
    let mut level = 0u16;
    for edge in lattice.edges_of(record)? {
        if edge.is_live() {
            level = level.max(edge.weight);
        }
    }
    Ok(level)
}

/// 04 §31 AC.1: `read::event_view_index` — the tugs a frame starts from.
///
/// `(event_id, point)` order is the walk's own (04 §4 note 7); the sort here is
/// by event then point set, so a cap applied downstream is a prefix of a stable
/// sequence.
pub fn event_view_index(lattice: &Lattice, anchor: i32, tug_q12: u16) -> Result<Vec<OriginTug>> {
    let events = events_of(lattice, anchor, true)?;
    let mut tugs: Vec<OriginTug> = Vec::new();
    for (_, record) in lattice.rows()? {
        let envelope = record.envelope;
        if !is_belief_shelf(envelope.shelf_x)
            || !envelope.active_at(anchor)
            || envelope.prov_speaker != 0
        {
            continue;
        }
        let Some(event_id) = events
            .get(&(envelope.prov_session, envelope.prov_step))
            .copied()
        else {
            continue;
        };
        let mut points: Vec<u32> = [record.body.key_point, record.body.val_point]
            .into_iter()
            .filter(|point| *point != 0)
            .collect();
        points.sort_unstable();
        points.dedup();
        if points.is_empty() {
            continue;
        }
        let weight_q12 = mul_q12_u16(tug_q12, level_of_belief(lattice, &record)?);
        if weight_q12 == 0 {
            continue;
        }
        tugs.push(OriginTug { event_id, points, weight_q12 });
    }
    tugs.sort_by(|left, right| (left.event_id, &left.points).cmp(&(right.event_id, &right.points)));
    Ok(tugs)
}

/// 12 §2.6: the frame's event views, for `mindlog.json`. Every `context` row in
/// the lattice gets one — grimm lines included, with an empty `belief_rows`
/// (§24.4: addressable scenery, zero authority).
pub fn event_views(lattice: &Lattice, anchor: i32) -> Result<Vec<EventView>> {
    let mut views: BTreeMap<u32, EventView> = BTreeMap::new();
    for (_, record) in lattice.rows()? {
        let envelope = record.envelope;
        if envelope.shelf_x != SHELF_CONTEXT || !envelope.active_at(anchor) {
            continue;
        }
        views.insert(
            envelope.row_id,
            EventView {
                event_id: envelope.row_id,
                belief_rows: Vec::new(),
                stamps: Vec::new(),
                a_at: 0,
                t: envelope.written_at_t,
                speaker_identity: 0,
            },
        );
    }
    if views.is_empty() {
        return Ok(Vec::new());
    }
    let events = events_of(lattice, anchor, true)?;
    for (_, record) in lattice.rows()? {
        let envelope = record.envelope;
        if !is_belief_shelf(envelope.shelf_x)
            || !envelope.active_at(anchor)
            || envelope.prov_speaker != 0
        {
            continue;
        }
        let Some(event_id) = events
            .get(&(envelope.prov_session, envelope.prov_step))
            .copied()
        else {
            continue;
        };
        let Some(view) = views.get_mut(&event_id) else {
            continue;
        };
        view.belief_rows.push(envelope.row_id);
        for edge in lattice.edges_of(&record)? {
            if !edge.is_live() {
                continue;
            }
            let stamps = lattice.stamps_of(&edge)?;
            for (index, stamp) in stamps.iter().enumerate() {
                view.stamps
                    .push(edge.charge_off + (index * STAMP_BYTES) as u32);
                if view.a_at == 0 {
                    view.a_at = stamp.a_at;
                }
            }
        }
    }
    for view in views.values_mut() {
        view.belief_rows.sort_unstable();
        view.belief_rows.dedup();
        view.stamps.sort_unstable();
        view.stamps.dedup();
    }
    Ok(views.into_values().collect())
}
