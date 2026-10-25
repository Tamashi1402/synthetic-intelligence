//! 04 §12 step 0b / §33 AE.2–AE.3 (B18, engineering/11 §B18 DoD item 1): the class
//! arm answers **before** steps 1–10 with zero bytes, and §15's new class axis
//! scopes the conflict list ahead of the tier table.

mod common;

use common::{candidate, ctx, params};
use si_core::addr::{SHELF_S_R_PAIRS, SHELF_FACTS};
use si_core::world::SpeakerClass;
use si_learn::gate::{self, Conflict, Outcome, WriteContext};
use si_store::store::Lattice;

fn scoped(t: i32, class: SpeakerClass) -> WriteContext {
    let mut c = ctx(t);
    c.class = class;
    c
}

fn arenas(store: &Lattice) -> (usize, usize, usize) {
    (
        store.rows_arena().len(),
        store.edges_arena().len(),
        store.stamps_arena().len(),
    )
}

#[test]
fn a_trusted_belief_write_is_refused_with_zero_bytes() {
    let mut store = Lattice::new();
    let rows_before = store.rows_arena().len();
    let mut stats = gate::GateStats::default();
    let c = candidate(&mut store, gate::PROV_STATEMENT, "apple", "red", 1);
    let report = gate::write(&mut store, &c, &scoped(1, SpeakerClass::Trusted), &params(), &mut stats)
        .unwrap();
    assert_eq!(
        report.outcome,
        Outcome::Refused { class: SpeakerClass::Trusted, reason: "SpeakerNotRepresentable" },
        "AE.2: the honest answer while OD-135 is open is a refusal, not a fourth prov_speaker"
    );
    // Steps 1–10 never ran: no novelty call, no salience read, no stats moved.
    assert_eq!(report.salience_calls, 0);
    assert_eq!(report.novelty.delta_q12, 0);
    assert_eq!(stats, gate::GateStats::default(), "a refusal commits and confirms nothing");
    assert_eq!(store.rows_arena().len(), rows_before);
    assert!(common::live(SHELF_S_R_PAIRS, &store).is_empty());
    assert!(common::live(SHELF_FACTS, &store).is_empty());
    // one log line, and it names the arm and the reason.
    assert_eq!(report.events.len(), 1, "{:?}", report.events);
    let body = &report.events[0].body;
    assert!(body.contains("\"arm\":\"class\""), "{body}");
    assert!(body.contains("SpeakerNotRepresentable"), "{body}");
    assert!(body.contains("\"trusted\""), "{body}");
}

#[test]
fn the_owner_and_grimm_arms_are_unchanged() {
    // The class arm is additive: an Owner line still commits and a Grimm line
    // still answers at step 0's tag guard with its own reason string (B9's law,
    // G12's assertion), not with the B18 refusal.
    let mut store = Lattice::new();
    let mut stats = gate::GateStats::default();
    let c = candidate(&mut store, gate::PROV_OWNER_TEACH, "apple", "red", 1);
    let report = gate::write(&mut store, &c, &ctx(1), &params(), &mut stats).unwrap();
    assert!(matches!(report.outcome, Outcome::Committed { .. }), "{:?}", report.outcome);

    let mut grimm = ctx(2);
    grimm.source = si_core::strings::Source::Grimm;
    grimm.class = SpeakerClass::Grimm;
    let repeat = candidate(&mut store, gate::PROV_OWNER_TEACH, "banana", "yellow", 2);
    let report = gate::write(&mut store, &repeat, &grimm, &params(), &mut stats).unwrap();
    assert!(
        matches!(report.outcome, Outcome::Inert { reason: "source_not_owner" }),
        "{:?}",
        report.outcome
    );
}

#[test]
fn a_stranger_write_moves_no_arena_byte() {
    // AE.3: a stranger's line can never raise `repeats` or a confidence. The
    // session's step-0 tag guard answers it; the gate's arenas do not move.
    let mut store = Lattice::new();
    let mut stats = gate::GateStats::default();
    let seed = candidate(&mut store, gate::PROV_OWNER_TEACH, "apple", "red", 1);
    gate::write(&mut store, &seed, &ctx(1), &params(), &mut stats).unwrap();
    let before = common::live(SHELF_S_R_PAIRS, &store)[0].clone();
    let bytes = arenas(&store);
    let mut stranger = ctx(2);
    stranger.source = si_core::strings::Source::Grimm;
    stranger.class = SpeakerClass::Stranger;
    let echo = candidate(&mut store, gate::PROV_OWNER_TEACH, "apple", "red", 2);
    let report = gate::write(&mut store, &echo, &stranger, &params(), &mut stats).unwrap();
    assert!(matches!(report.outcome, Outcome::Inert { .. }), "{:?}", report.outcome);
    assert_eq!(arenas(&store), bytes, "a stranger bought nothing durable (R-35)");
    let after = common::live(SHELF_S_R_PAIRS, &store)[0].clone();
    assert_eq!(after.envelope.repeats, before.envelope.repeats);
    assert_eq!(after.envelope.confidence, before.envelope.confidence);
}

fn conflict(speaker: u8, row_id: u32) -> Conflict {
    Conflict {
        shelf: SHELF_S_R_PAIRS,
        row_id,
        val_point: 9,
        prov_kind: gate::PROV_STATEMENT,
        prov_speaker: speaker,
        confidence: 50,
        written_at: 1,
        prov_step: 1,
    }
}

#[test]
fn the_class_axis_is_strictly_above_the_tier_axis() {
    let conflicts = [
        conflict(gate::SPEAKER_OWNER, 1),
        conflict(gate::SPEAKER_GRIMM, 2),
        conflict(gate::SPEAKER_SYSTEM_INFERRED, 3),
    ];
    // An Owner candidate keeps §15's rows 1–4 exactly as built.
    assert_eq!(gate::class_scope(SpeakerClass::Owner, &conflicts).len(), 3);
    // A Trusted correction supersedes Trusted rows only — and there are none,
    // because its belief write is refused — so it may age no row at all, and
    // above all no owner row (§33 AE.3).
    assert!(gate::class_scope(SpeakerClass::Trusted, &conflicts).is_empty());
    // A Grimm- or Stranger-class line never reaches arbitration.
    assert!(gate::class_scope(SpeakerClass::Grimm, &conflicts).is_empty());
    assert!(gate::class_scope(SpeakerClass::Stranger, &conflicts).is_empty());
    // There is no code path that orders Trusted above Owner.
    assert!(SpeakerClass::Owner > SpeakerClass::Trusted);
}

#[test]
fn an_origin_two_ask_answers_only_in_the_owner_lane_whatever_the_class() {
    // 04 §12 step 4 / AE.4: the lane is `key_field`'s origin half, and no class
    // value moves it. A granted or stranger line cannot capture a nomination ask
    // any more than the bare reply that was answering an ordinary sense ask.
    use si_core::patterns::PatternId;
    let mut store = Lattice::new();
    let sense = gate::QuestionSpec {
        pattern_id: PatternId::PQuestA.index() as u16,
        gap_kind: 4,
        origin: gate::ASK_ORIGIN_SENSE,
        key_point: 11,
        target_slot: 0,
        text_ref: 5,
        prov_step: 1,
        arousal_at_ask: 0,
    };
    let nomination = gate::QuestionSpec {
        pattern_id: PatternId::PQuestA.index() as u16,
        gap_kind: 6,
        origin: gate::ASK_ORIGIN_NOMINATION,
        key_point: 12,
        target_slot: 0,
        text_ref: 6,
        prov_step: 1,
        arousal_at_ask: 0,
    };
    let sense_row = gate::append_question(&mut store, &sense, &ctx(1)).unwrap();
    let nomination_row = gate::append_question(&mut store, &nomination, &ctx(1)).unwrap();

    for class in [
        SpeakerClass::Owner,
        SpeakerClass::Grimm,
        SpeakerClass::Trusted,
        SpeakerClass::Stranger,
    ] {
        let scoped = scoped(2, class);
        // The bare reply's lookup: the sense ask, and never the nomination.
        let matched = gate::latest_open_ask(&store, scoped.t).unwrap();
        assert_eq!(matched.map(|row| row.row_id), Some(sense_row), "{class}");
        // The owner lane's lookup: the nomination, and not by a sense reply.
        let lane = gate::latest_open_nomination(&store, scoped.t, 6).unwrap();
        assert_eq!(lane.map(|row| row.row_id), Some(nomination_row), "{class}");
        assert_eq!(
            gate::latest_open_nomination(&store, scoped.t, 4).unwrap(),
            None,
            "{class}: the nomination lane answers kind 6, not the sense ask"
        );
    }
}
