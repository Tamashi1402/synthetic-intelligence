//! 04 §21 U.2: the claim label is a pure function of the settled thought and the
//! cited rows; first match wins, and the hop cap is claim-side.

use si_core::consts;
use si_front::{
    keys, label, Caps, ClaimLabel, Flavor, LabelInput, Reason, Rendered, SourceTag, HOP_CAP_ROW,
};
use si_read::{Activation, Cited, RouteCounts, StopReason, Thought};

fn thought(stop_reason: StopReason) -> Thought {
    Thought {
        settle_set: vec![Activation { point: 20, activation: 4000, first_lit: 0 }],
        rounds: Vec::new(),
        a_trajectory: Vec::new(),
        stop_reason,
        lit_round0: Vec::new(),
        relit: Vec::new(),
        oscillated: false,
        frame_log: Default::default(),
    }
}

fn cited(row_id: u32, surface: &str) -> Cited {
    Cited {
        row_id,
        kind: 0,
        confidence: 95,
        active_at_anchor: true,
        key_point: 10,
        val_point: 20 + row_id,
        surface: surface.to_string(),
        nameless: false,
    }
}

fn rendered(surface: &str, containment: bool, leaked: &[&str]) -> Rendered {
    Rendered {
        surface: surface.to_string(),
        content: keys(surface),
        citations: Vec::new(),
        leaked: leaked.iter().map(|key| key.to_string()).collect(),
        containment,
    }
}

fn route(owned: u32, base: u32, hops: u32) -> RouteCounts {
    RouteCounts { owned, base, hops }
}

struct Case {
    thought: Thought,
    cited: Vec<Cited>,
    rendered: Rendered,
    question_shaped: bool,
    named_dark: bool,
    route: RouteCounts,
    caps: Caps,
}

fn case() -> Case {
    Case {
        thought: thought(StopReason::Settled),
        cited: Vec::new(),
        rendered: rendered("", true, &[]),
        question_shaped: true,
        named_dark: false,
        route: route(0, 0, 0),
        caps: Caps { max_owned_hops: 2, engine: false },
    }
}

fn run(case: Case) -> si_front::Labeled {
    label(&LabelInput {
        thought: &case.thought,
        caps: &case.caps,
        cited: &case.cited,
        rendered: &case.rendered,
        question_shaped: case.question_shaped,
        named_dark: case.named_dark,
        route: case.route,
    })
}

#[test]
fn a_settled_row_that_carries_the_whole_answer_is_a_fact() {
    let mut scenario = case();
    scenario.cited = vec![cited(3, "red")];
    scenario.rendered = rendered("red", true, &[]);
    scenario.route = route(1, 0, 1);
    let labeled = run(scenario);
    assert_eq!(labeled.label, ClaimLabel::Fact);
    assert_eq!(labeled.flavor, Flavor::Plain);
    assert_eq!(labeled.tag, SourceTag::Owned);
    assert_eq!(labeled.reason, Reason::SettledFact);
    assert_eq!(labeled.citation, vec![3]);
    assert_eq!(labeled.surface(), "FACT");
}

#[test]
fn two_cited_rows_compose_into_an_inference_inside_the_cap() {
    let mut scenario = case();
    scenario.cited = vec![cited(3, "red"), cited(4, "green")];
    scenario.rendered = rendered("red green", true, &[]);
    scenario.route = route(2, 0, 2);
    let labeled = run(scenario);
    assert_eq!(labeled.label, ClaimLabel::Inference);
    assert_eq!(labeled.reason, Reason::Composed);
    assert_eq!(labeled.citation, vec![3, 4]);
    assert_eq!(labeled.surface(), "INFERENCE");
}

#[test]
fn an_over_deep_route_is_the_uncertainty_surface_with_the_full_route_cited() {
    let mut scenario = case();
    scenario.cited = vec![cited(3, "red"), cited(4, "green")];
    scenario.rendered = rendered("red green", true, &[]);
    scenario.route = route(3, 0, 3);
    let labeled = run(scenario);
    assert_eq!(labeled.label, ClaimLabel::Speculation);
    assert_eq!(labeled.flavor, Flavor::Uncertainty);
    assert_eq!(labeled.reason, Reason::Depth);
    assert_eq!(labeled.surface(), "UNCERTAINTY");
    assert_eq!(labeled.citation, vec![3, 4]);
    assert!(labeled.is_uncertain());
}

#[test]
fn a_maxk_stop_is_the_uncertainty_surface_without_a_claim() {
    let mut scenario = case();
    scenario.thought = thought(StopReason::MaxK);
    let labeled = run(scenario);
    assert_eq!(labeled.label, ClaimLabel::Speculation);
    assert_eq!(labeled.flavor, Flavor::Uncertainty);
    assert_eq!(labeled.reason, Reason::MaxK);
    assert_eq!(labeled.surface(), "UNCERTAINTY");
}

#[test]
fn a_named_dark_spot_is_i_dont_know_and_cites_the_ask_back() {
    let mut scenario = case();
    scenario.thought = thought(StopReason::TauMiss);
    scenario.named_dark = true;
    let labeled = run(scenario);
    assert_eq!(labeled.label, ClaimLabel::DontKnow);
    assert_eq!(labeled.reason, Reason::NamedDark);
    assert_eq!(labeled.tag, SourceTag::None);
    assert_eq!(labeled.surface(), "I-DON'T-KNOW");
}

#[test]
fn an_unnamed_dark_spot_gets_the_nonce_guess_label() {
    let mut scenario = case();
    scenario.thought = thought(StopReason::TauMiss);
    let labeled = run(scenario);
    assert_eq!(labeled.label, ClaimLabel::Speculation);
    assert_eq!(labeled.flavor, Flavor::Plain);
    assert_eq!(labeled.reason, Reason::Novel);
    assert_eq!(labeled.tag, SourceTag::None);
    assert_eq!(labeled.surface(), "SPECULATION");
}

#[test]
fn a_statement_with_no_route_is_i_dont_know() {
    let mut scenario = case();
    scenario.thought = thought(StopReason::TauMiss);
    scenario.question_shaped = false;
    let labeled = run(scenario);
    assert_eq!(labeled.label, ClaimLabel::DontKnow);
    assert_eq!(labeled.reason, Reason::NoSlot);
}

#[test]
fn a_genome_only_route_is_a_capped_inference_in_engine_mode() {
    let mut scenario = case();
    scenario.caps = Caps { max_owned_hops: 2, engine: true };
    scenario.route = route(0, 1, 0);
    let labeled = run(scenario);
    assert_eq!(labeled.label, ClaimLabel::Inference);
    assert_eq!(labeled.reason, Reason::BaseOnly);
    assert_eq!(labeled.tag, SourceTag::Base);
    // CONFIG SI may not label a genome path INFERENCE.
    let mut config = case();
    config.route = route(0, 1, 0);
    assert_ne!(run(config).label, ClaimLabel::Inference);
}

#[test]
fn a_token_absent_from_the_cited_row_demotes_the_fact() {
    let mut scenario = case();
    scenario.cited = vec![cited(3, "red")];
    scenario.rendered = rendered("red green", false, &["green"]);
    scenario.route = route(1, 0, 1);
    let labeled = run(scenario);
    assert_ne!(labeled.label, ClaimLabel::Fact);
    assert_eq!(labeled.surface(), "UNCERTAINTY");
    assert_eq!(labeled.reason, Reason::Novel);
}

#[test]
fn the_cap_is_read_from_the_registry_value() {
    assert_eq!(HOP_CAP_ROW, "INFERENCE_MAX_HOPS_OWNED");
    let row = consts::row(HOP_CAP_ROW).expect("the hop cap is a registry row");
    let expected = consts::registry_value(row).expect("the row is a swept scalar SPECIES row");
    assert_eq!(Caps::registry().max_owned_hops as i64, expected);
    assert!(!Caps::registry().engine);
}
