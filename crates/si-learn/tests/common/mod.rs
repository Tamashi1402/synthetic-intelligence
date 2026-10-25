//! Shared fixtures for the gate tests: one empty lattice, one owner context and
//! a candidate builder. No test here touches `si-session`; the gate is exercised
//! as a function of (store, candidate, context).

use si_affect::params::Params;
use si_core::record::Envelope;
use si_core::seed::derive_temperament;
use si_core::strings::Source;
use si_core::world::SpeakerClass;
use si_learn::gate::{self, Candidate, WriteContext};
use si_store::store::Lattice;

pub fn params() -> Params {
    Params::from_temperament(&derive_temperament(0x51))
}

pub fn ctx(t: i32) -> WriteContext {
    WriteContext {
        t,
        prov_session: 1,
        source: Source::Owner,
        class: SpeakerClass::Owner,
        stage: 0,
        a_formation: 0,
        thought_id: 1,
        signal_ref: t as u32,
    }
}

pub fn candidate(
    store: &mut Lattice,
    prov_kind: u8,
    key: &str,
    value: &str,
    t: i32,
) -> Candidate {
    let key_point = store.points_mut().ensure_term(key, key);
    let val_point = store.points_mut().ensure_term(value, value);
    let pattern = si_core::patterns::PatternId::PQuestA;
    let stim_surface = store.interner_mut().intern(&format!("what color is {key}"));
    let resp_surface = store.interner_mut().intern(value);
    let stim_signature = gate::intern_signature(store, key, value);
    let text_ref = store.interner_mut().intern(value);
    Candidate {
        shelf: si_core::addr::SHELF_S_R_PAIRS,
        pattern_id: pattern.index() as u16,
        key_field: Candidate::key_field_for(pattern, 0),
        key_point,
        val_point,
        stim_signature,
        stim_surface,
        resp_surface,
        target_slot: 0,
        prov_kind,
        prov_speaker: gate::SPEAKER_OWNER,
        bind_output_id: 0,
        question_id: 0,
        text_ref,
        prov_step: t,
        marker: false,
        attempts_at_slot: 0,
        terms: Vec::new(),
        derived_from: Vec::new(),
        stimulus: format!("{key} {value}"),
        response: value.to_string(),
    }
}

pub fn live(shelf: u16, store: &Lattice) -> Vec<si_core::record::Record> {
    store
        .active_at(shelf, i32::MAX)
        .unwrap()
        .into_iter()
        .map(|(_, record)| record)
        .collect()
}

pub fn envelope(shelf: u16, _row: u32, t: i32) -> Envelope {
    Envelope::live(shelf, t, 1, t)
}
