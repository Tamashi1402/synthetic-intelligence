//! 11 §B18 DoD 2 / 01 §9: the `SIro` container's version bump is the arc's one
//! byte, and it moves nothing else. A B18 inbox reads as the frames it carries, a
//! pre-B18 inbox refuses by name, a lane cut by a dead viewer reads as the frames
//! before the cut (08 §2.2), and the same lines driven through the transport
//! hash exactly as the same lines driven without it — identity is metadata on
//! sense, so it cannot reach the nine hash items.

use si_core::addr::{SHELF_CONTEXT, SHELF_FACTS, SHELF_S_R_PAIRS};
use si_core::container::{Container, Format, CONTAINER_VERSION};
use si_core::eventlog::{encode_sense_body, Event};
use si_core::strings::Source;
use si_core::world::{SpeakerClass, IDENTITY_OWNER};
use si_session::{InboxOutcome, OpenOutcome, Output, SenseFrame, Session};

fn frames(texts: &[(&str, u64, u32, u8)]) -> Vec<u8> {
    let mut payload = Vec::new();
    for (seq, (text, identity, conversation, source)) in texts.iter().enumerate() {
        let event = Event {
            seq: seq as u64,
            t: seq as i32,
            source: *source,
            kind: 0,
            body: encode_sense_body(text.as_bytes(), *identity, *conversation).unwrap(),
        };
        let frame = event.encode();
        payload.extend_from_slice(&(frame.len() as u32).to_le_bytes());
        payload.extend_from_slice(&frame);
    }
    Container::new(Format::StreamRecord, serde_json::json!({ "lane": "sense_inbox" }), payload)
        .encode()
        .unwrap()
}

#[test]
fn a_b18_inbox_reads_as_frames_and_a_pre_b18_inbox_refuses_by_name() {
    let bytes = frames(&[("apple is red", IDENTITY_OWNER, 0, 0), ("sky is blue", 7, 0, 1)]);
    let container = Container::decode(&bytes).unwrap();
    assert_eq!(container.version, Format::StreamRecord.container_version());
    assert_eq!(container.version, CONTAINER_VERSION + 1, "one bump, and only the stream");

    let outcome = Session::read_sense_inbox(&bytes).unwrap();
    let InboxOutcome::Accepted { frames: read, torn_tail_bytes: 0 } = outcome else {
        panic!("a B18 inbox reads: {outcome:?}");
    };
    assert_eq!(read.len(), 2);
    assert_eq!(read[0].text, "apple is red");
    assert_eq!(read[0].identity, IDENTITY_OWNER);
    assert_eq!(read[0].source, Source::Owner);
    assert_eq!(read[1].identity, 7);
    assert_eq!(read[1].source, Source::Grimm);

    // The same lane as it was written before the transport byte: same magic, older
    // version. 00 §5's mismatch law answers with the number it found.
    let older = Container { version: CONTAINER_VERSION, ..container };
    let outcome = Session::read_sense_inbox(&older.encode().unwrap()).unwrap();
    let InboxOutcome::Refused(refusal) = outcome else {
        panic!("a pre-B18 inbox refuses: {outcome:?}");
    };
    let named = refusal.to_string();
    assert!(named.contains("1") && named.contains("2"), "{named}");
    assert!(named.to_lowercase().contains("version"), "{named}");
}

#[test]
fn a_lane_cut_mid_frame_is_the_frames_before_it_and_not_a_refusal() {
    // [08 §2.2]'s dead-window row says the effect of a viewer killed while appending is
    // **no effect**, and [08 §1]'s HEADLESS law is the same sentence from the loop's
    // side. So the tail of a frame that never finished being written is a tail that was
    // never written: the lane ends at the last whole frame and the cut is named.
    let a = Event {
        seq: 0,
        t: 0,
        source: 0,
        kind: 0,
        body: encode_sense_body(b"apple is red", IDENTITY_OWNER, 0).unwrap(),
    };
    let b = Event {
        seq: 1,
        t: 1,
        source: 1,
        kind: 0,
        body: encode_sense_body(b"sky is blue", 7, 0).unwrap(),
    };
    let b_bytes = b.encode();
    let bytes = frames(&[("apple is red", IDENTITY_OWNER, 0, 0), ("sky is blue", 7, 0, 1)]);
    assert_eq!(
        Container::decode(&bytes).unwrap().payload.len(),
        8 + a.encode().len() + b_bytes.len(),
        "the lane's payload is `[4][a][4][b]` — the byte maths below assumes nothing else"
    );

    // Cut the second frame's body in half. The payload behind the header is
    // `[4][a][4][b]`, so the lane now ends `b.len()/2` bytes early.
    let half = b_bytes.len() / 2;
    let cut = &bytes[..bytes.len() - half];
    let outcome = Session::read_sense_inbox(cut).unwrap();
    let InboxOutcome::Accepted { frames, torn_tail_bytes } = outcome else {
        panic!("a cut lane is a read with a named tail, not a refusal: {outcome:?}");
    };
    assert_eq!(frames.len(), 1, "the frame before the cut is heard");
    assert_eq!(frames[0].text, "apple is red");
    assert_eq!(
        torn_tail_bytes,
        4 + b_bytes.len() - half,
        "the cut is counted from the last whole frame, length prefix included"
    );

    // A cut so deep that not even a length prefix survives is the same case from the
    // other side: no frame starts in the stub, and the stub is named as the tail.
    let stub = &bytes[..bytes.len() - (4 + b_bytes.len() - 2)];
    let outcome = Session::read_sense_inbox(stub).unwrap();
    let InboxOutcome::Accepted { frames, torn_tail_bytes } = outcome else {
        panic!("a stub lane still reads: {outcome:?}");
    };
    assert_eq!(frames.len(), 1);
    assert_eq!(torn_tail_bytes, 2, "a stub shorter than a length prefix is the whole tail");
}

#[test]
fn a_body_without_the_transport_pair_is_refused_not_guessed() {
    // A v2 container is a promise about its bodies: a bare-text body is a shape
    // mismatch, named by the frame's own seq rather than read as 12 characters of
    // the sentence.
    let event = Event { seq: 3, t: 0, source: 0, kind: 0, body: b"apple is red".to_vec() };
    let frame = event.encode();
    let mut payload = Vec::new();
    payload.extend_from_slice(&(frame.len() as u32).to_le_bytes());
    payload.extend_from_slice(&frame);
    let bytes = Container::new(
        Format::StreamRecord,
        serde_json::json!({ "lane": "sense_inbox" }),
        payload,
    )
    .encode()
    .unwrap();
    let outcome = Session::read_sense_inbox(&bytes).unwrap();
    let InboxOutcome::Refused(refusal) = outcome else {
        panic!("a short body refuses: {outcome:?}");
    };
    assert!(refusal.to_string().contains('3'), "{refusal}");
}

#[test]
fn the_transport_pair_moves_no_hash_byte_and_a_b18_container_resumes() {
    let lines = ["what color is apple?", "apple is red", "banana is yellow"];
    let mut plain = Session::new(0x51).unwrap();
    for line in &lines {
        plain.tick(line, Source::Owner).unwrap();
    }

    let mut framed = Session::new(0x51).unwrap();
    for line in &lines {
        framed
            .tick_frame(&SenseFrame {
                text: line.to_string(),
                identity: IDENTITY_OWNER,
                conversation: 0,
                source: Source::Owner,
            })
            .unwrap();
    }
    assert_eq!(
        framed.state_hash().unwrap().hex(),
        plain.state_hash().unwrap().hex(),
        "01 §9: identity is on sense, never in a hash item"
    );
    assert_eq!(framed.conversation(), plain.conversation());
    assert_eq!(framed.class_of(IDENTITY_OWNER, framed.now()), plain.class_of(IDENTITY_OWNER, plain.now()));

    // DoD 2's resume half: a session that took the bumped container saves and
    // re-opens at the same seed with the same digest.
    let path = std::env::temp_dir().join(format!(
        "si-b18-session-{}-resume.SIck",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    framed.save(&path).unwrap();
    let opened = match Session::open(&path, 0x51).unwrap() {
        OpenOutcome::Opened(session) => session,
        OpenOutcome::Refused(refusal) => panic!("the B18 container refuses its own seed: {refusal}"),
    };
    assert_eq!(opened.state_hash().unwrap().hex(), framed.state_hash().unwrap().hex());
    let _ = std::fs::remove_file(&path);
}

fn shelf_rows(session: &Session, shelf: u16) -> usize {
    session
        .store()
        .rows()
        .unwrap()
        .into_iter()
        .filter(|(_, record)| record.envelope.shelf_x == shelf && record.envelope.is_live())
        .count()
}

#[test]
fn a_stranger_line_pays_its_context_row_and_writes_no_belief() {
    // 05 §23.5(a) / DoD 1: a stranger's line is not silenced at the sense-input
    // floor. The `context` row is what the substrate keeps of the line; the belief
    // shelves see nothing of it.
    let mut session = Session::new(0x51).unwrap();
    let before = (
        shelf_rows(&session, SHELF_S_R_PAIRS),
        shelf_rows(&session, SHELF_FACTS),
        shelf_rows(&session, SHELF_CONTEXT),
    );
    session
        .tick_frame(&SenseFrame {
            text: "apple is red".to_string(),
            identity: 7,
            conversation: 0,
            source: Source::Grimm,
        })
        .unwrap();
    assert_eq!(session.class_of(7, session.now()), SpeakerClass::Stranger);
    let after = (
        shelf_rows(&session, SHELF_S_R_PAIRS),
        shelf_rows(&session, SHELF_FACTS),
        shelf_rows(&session, SHELF_CONTEXT),
    );
    assert_eq!(after.0, before.0);
    assert_eq!(after.1, before.1);
    assert_eq!(after.2, before.2 + 1, "a stranger is heard, and believed nothing");
}

#[test]
fn a_granted_line_is_refused_at_the_class_arm_after_its_context_row() {
    // 04 §12 step 0b end-to-end, and G19(c)'s shape: the grant makes the next
    // derivation `Trusted`, the belief write returns the named refusal with zero
    // bytes, and the revocation ends the class on the line after it.
    use si_core::world::{GrantRow, Right};
    let mut session = Session::new(0x51).unwrap();
    session.grant(GrantRow {
        subject: 7,
        right: Right::Teach,
        granted_by: IDENTITY_OWNER,
        granted_at_t: session.now(),
        revoked_at_t: None,
    });
    assert_eq!(session.class_of(7, session.now()), SpeakerClass::Trusted);

    let before = (
        shelf_rows(&session, SHELF_S_R_PAIRS),
        shelf_rows(&session, SHELF_CONTEXT),
    );
    let report = session
        .tick_frame(&SenseFrame {
            text: "apple is red".to_string(),
            identity: 7,
            conversation: 0,
            source: Source::Grimm,
        })
        .unwrap();
    assert!(
        matches!(&report.output, Output::Refused { reason } if reason.contains("SpeakerNotRepresentable")),
        "{:?}",
        report.output
    );
    let after = (
        shelf_rows(&session, SHELF_S_R_PAIRS),
        shelf_rows(&session, SHELF_CONTEXT),
    );
    assert_eq!(after.0, before.0, "AE.2: the refusal buys zero bytes");
    assert_eq!(after.1, before.1 + 1);

    // I4: revocation is a mark, and the identity is a stranger again.
    session.revoke(7, Right::Teach, session.now() + 1);
    assert_eq!(session.class_of(7, session.now() + 1), SpeakerClass::Stranger);
}

#[test]
fn a_window_suppresses_the_idle_tick_and_costs_nothing_else() {
    // OD-126 / AE.4 end-to-end. docs/26 §26.4's interval is the one in which quiet
    // threads **may** ignite, so a tick outside it is denied and a tick inside it
    // is granted nothing it did not already have: no window can cause an ignition.
    use si_core::world::Window;
    let mut open = Session::new(0x51).unwrap();
    let mut shut = Session::new(0x51).unwrap();
    // Declared entirely ahead of this run's clock: the busy server's window.
    shut.set_window(Some(Window { open_from_t: 1_000_000, open_through_t: 1_000_001 }));
    let idle_open = open.idle_tick().unwrap();
    let idle_shut = shut.idle_tick().unwrap();
    assert!(!idle_open.suppressed_by_window);
    assert!(idle_shut.suppressed_by_window);
    // What the window did not do: no ignition, no sleep pass, no clock difference,
    // and the quiet run still counts on its own law.
    assert_eq!(shut.sleeps_completed(), open.sleeps_completed());
    assert_eq!(shut.now(), open.now());
    assert_eq!(shut.quiet_run(), open.quiet_run(), "the quiet law still holds on its own");
    // The same session with the window standing open: nothing is caused, and the
    // step-12 gate is exactly the pre-B18 predicate.
    shut.set_window(Some(Window { open_from_t: 0, open_through_t: i32::MAX }));
    assert!(!shut.idle_tick().unwrap().suppressed_by_window, "an open window makes nothing happen");
    shut.set_window(None);
    assert!(!shut.idle_tick().unwrap().suppressed_by_window);
}

#[test]
fn a_declared_owner_preemption_is_untouched_by_the_window() {
    // G19(g)'s second half: the owner's line runs its full tick while the window
    // stands shut, because a window can say "not now" only to step 12.
    use si_core::world::Window;
    let mut session = Session::new(0x51).unwrap();
    session.set_window(Some(Window { open_from_t: 1_000_000, open_through_t: 1_000_001 }));
    let report = session.tick("apple is red", Source::Owner).unwrap();
    assert!(!matches!(&report.output, Output::Refused { .. }), "{:?}", report.output);
    assert!(shelf_rows(&session, SHELF_CONTEXT) >= 1);
}
