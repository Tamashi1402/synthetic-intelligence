//! 04 §21 U.1 / 02 §7: the renderer copies, never creates. Containment is
//! measured against the cited surfaces; v1's copy makes it a byte-subset by
//! construction, and the labeler is what demotes a non-subset.

use si_front::{keys, render_monologue, reverse_decode};
use si_read::{Activation, Cited, StopReason, Thought};

fn thought(entries: &[(u32, i32)], stop_reason: StopReason) -> Thought {
    Thought {
        settle_set: entries
            .iter()
            .map(|(point, activation)| Activation {
                point: *point,
                activation: *activation,
                first_lit: 0,
            })
            .collect(),
        rounds: Vec::new(),
        a_trajectory: Vec::new(),
        stop_reason,
        lit_round0: Vec::new(),
        relit: Vec::new(),
        oscillated: false,
        frame_log: Default::default(),
    }
}

fn cited(row_id: u32, val_point: u32, surface: &str) -> Cited {
    Cited {
        row_id,
        kind: 0,
        confidence: 95,
        active_at_anchor: true,
        key_point: 10,
        val_point,
        surface: surface.to_string(),
        nameless: false,
    }
}

#[test]
fn the_answer_is_the_top_cited_rows_surface_byte_for_byte() {
    let thought = thought(&[(20, 100), (21, 300)], StopReason::Settled);
    let rows = vec![cited(7, 20, "it is red"), cited(9, 21, "It is GREEN")];
    let rendered = reverse_decode(&thought, &rows);
    assert_eq!(rendered.surface, "It is GREEN");
    assert_eq!(rendered.citations, vec![7, 9]);
    assert!(rendered.containment);
    assert!(rendered.leaked.is_empty());
}

#[test]
fn a_tie_breaks_to_the_lower_row_id() {
    let thought = thought(&[(20, 250), (21, 250)], StopReason::Settled);
    let rows = vec![cited(9, 21, "second"), cited(4, 20, "first")];
    let rendered = reverse_decode(&thought, &rows);
    assert_eq!(rendered.surface, "first");
}

#[test]
fn every_content_token_is_in_the_cited_surfaces() {
    let thought = thought(&[(20, 300), (21, 100)], StopReason::Settled);
    let rows = vec![cited(1, 20, "the apple is red"), cited(2, 21, "green")];
    let rendered = reverse_decode(&thought, &rows);
    assert_eq!(rendered.content, keys("the apple is red"));
    let union: Vec<String> =
        rows.iter().flat_map(|row| keys(&row.surface)).collect();
    for key in &rendered.content {
        assert!(union.contains(key), "content token `{key}` is in no cited surface");
    }
    assert!(rendered.containment);
}

#[test]
fn no_cited_row_renders_nothing_and_claims_nothing() {
    let thought = thought(&[(20, 300)], StopReason::Settled);
    let rendered = reverse_decode(&thought, &[]);
    assert!(rendered.surface.is_empty());
    assert!(rendered.content.is_empty());
    assert!(rendered.citations.is_empty());
    assert!(rendered.containment);
}

#[test]
fn the_monologue_copies_display_order_and_cites_nothing() {
    let thought = thought(&[(20, 100), (21, 300), (22, 200)], StopReason::Settled);
    let words: Vec<(u32, String)> =
        vec![(20, "red".to_string()), (21, "blue".to_string()), (22, "green".to_string())];
    let rendered = render_monologue(&thought, &words, 2);
    assert_eq!(rendered.surface, "blue green");
    assert!(rendered.citations.is_empty());
    assert!(rendered.containment);
    assert_eq!(render_monologue(&thought, &words, 1).surface, "blue");
}

#[test]
fn a_point_with_no_word_is_skipped_not_invented() {
    let thought = thought(&[(20, 300), (21, 200)], StopReason::Settled);
    let words: Vec<(u32, String)> = vec![(20, "red".to_string())];
    let rendered = render_monologue(&thought, &words, 2);
    assert_eq!(rendered.surface, "red");
    assert_eq!(rendered.content, keys("red"));
}
