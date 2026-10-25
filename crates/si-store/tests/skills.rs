//! 01 §4.2 / 12 §2.5 §2.6: the `skills` row, in bytes.
//!
//! Shelf 9 is live from B17, so the row it holds is the arc's only *stored* skill
//! fact. Three claims, each of which is a byte rather than a sentence: the body's
//! reserved u32 is zero (a reserved byte is asserted, never trusted); an
//! `arousal_at_ask` of 255 is the row's own I8 assertion that no detector read `a`
//! when it asked; and retirement moves `status` by supersede, so the retired row
//! keeps its bytes and its citation (I4).

use si_core::addr::SHELF_SKILLS;
use si_core::record::{
    AROUSAL_NONE, BODY_BYTES, Body, Envelope, Record, PATTERN_NAMELESS, ROW_VERSION_V1,
    SKILL_KIND_SHAPE,
};
use si_store::store::Lattice;

fn skill_row(t: i32, pattern_id: u16) -> (Envelope, Body) {
    let mut envelope = Envelope::live(SHELF_SKILLS, t, 1, t);
    envelope.row_version = ROW_VERSION_V1;
    let body = Body {
        pattern_id,
        key_field: SKILL_KIND_SHAPE,
        key_point: 7,
        val_point: 0,
        stim_signature: 3,
        stim_surface: 4,
        resp_surface: 0,
        target_slot: 0,
        arousal_at_ask: AROUSAL_NONE,
        status: 0,
    };
    (envelope, body)
}

#[test]
fn a_skill_row_is_written_with_zero_reserved_bytes_and_no_arousal() {
    let mut lattice = Lattice::new();
    let row_id = lattice
        .append_row(
            skill_row(5, PATTERN_NAMELESS).0,
            skill_row(5, PATTERN_NAMELESS).1,
            Vec::new(),
            &[],
            &[],
        )
        .unwrap();

    let bytes = lattice.rows_arena().read(0).unwrap();
    assert_eq!(bytes.len(), 80 + BODY_BYTES);
    assert_eq!(
        &bytes[80 + 28..80 + 32],
        &[0, 0, 0, 0],
        "the skill body's reserved u32 at offset 28 is zero (01 §4.2)"
    );
    let record = Record::decode(bytes).unwrap();
    assert_eq!(record.envelope.row_id, row_id);
    assert_eq!(record.envelope.shelf_x, SHELF_SKILLS);
    assert_eq!(record.body.arousal_at_ask, AROUSAL_NONE);
    assert_eq!(record.body.key_field, SKILL_KIND_SHAPE);
    assert_eq!(record.body.pattern_id, PATTERN_NAMELESS);

    // A reserved byte is checked, not trusted: the decoder refuses a row that
    // filled it, so a future field cannot be smuggled in as pad.
    let mut tampered = bytes.to_vec();
    tampered[80 + 28] = 1;
    assert!(Record::decode(&tampered).is_err());
}

#[test]
fn a_retired_skill_keeps_its_bytes_and_loses_only_its_activeness() {
    let mut lattice = Lattice::new();
    let (envelope, body) = skill_row(5, 12);
    let row_id = lattice.append_row(envelope, body, Vec::new(), &[], &[]).unwrap();
    let before = lattice.rows_arena().read(0).unwrap().to_vec();

    // Retirement is a supersede with a named successor (12 §2.5, I4): the store
    // moves the interval, it does not delete the row.
    let (successor_envelope, successor_body) = skill_row(9, 12);
    let successor = lattice
        .append_row(successor_envelope, successor_body, Vec::new(), &[], &[])
        .unwrap();
    lattice.age(SHELF_SKILLS, row_id, 9, successor).unwrap();

    let after = lattice.rows_arena().read(0).unwrap().to_vec();
    // Retirement moves the *activeness interval* and nothing else: every byte
    // outside `superseded_at`/`superseded_by` (envelope offsets 16..24) is the
    // byte it was when the row was written.
    assert_eq!(&before[..16], &after[..16], "identity and age stamps survive");
    assert_eq!(&before[24..], &after[24..], "the body survives untouched");
    let retired = Record::decode(&after).unwrap();
    assert_eq!(retired.envelope.superseded_at, 9);
    assert_eq!(retired.envelope.superseded_by, successor);
    assert!(!retired.envelope.is_live());
    assert!(!retired.envelope.active_at(10), "a retired row is not active");
    assert!(retired.envelope.active_at(8), "and it stays citable inside its own interval");

    let active = lattice.active_at(SHELF_SKILLS, 10).unwrap();
    assert_eq!(active.len(), 1, "only the successor is live");
    assert_eq!(active[0].1.envelope.row_id, successor);
    assert_eq!(
        lattice
            .rows()
            .unwrap()
            .iter()
            .filter(|(_, row)| row.envelope.row_id == row_id)
            .count(),
        1,
        "I4: supersede keeps the bytes citable — nothing is erased"
    );
}

#[test]
fn the_skills_shelf_is_live_and_the_registry_says_so() {
    // 12 §2.2: `strategies` and `events` are the two seats that stay
    // reserved-empty. Shelf 9 opening must not quietly make it a third.
    assert!(!si_core::shelves::is_reserved(SHELF_SKILLS));
    assert!(si_core::shelves::is_l3_live(SHELF_SKILLS));
    assert_eq!(
        si_core::shelves::SHELVES.len(),
        11,
        "eleven entries, and `skills` did not move the name list (01 §2)"
    );
    let reserved: Vec<&str> = (1..=si_core::SHELF_COUNT as u16)
        .filter(|id| si_core::shelves::is_reserved(*id))
        .map(|id| si_core::shelves::entry(id).unwrap().shelf_name)
        .collect();
    assert_eq!(reserved, ["strategies", "events"], "two reserved-empty, by name");
    assert!(si_core::shelves::registry_is_valid());
}
