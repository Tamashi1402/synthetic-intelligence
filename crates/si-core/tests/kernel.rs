//! B0 DoD: the kernel's structural laws as executable assertions.

use si_core::consts::{self, Class};
use si_core::container::{Container, Format};
use si_core::hash::{
    encode_rng, hex, CoreBlock, HashInputs, StateHash, VersionTags, CORE_BLOCK_BYTES, HASH_FORMAT,
    EMPTY_STATE_MD5_SEED_ZERO,
};
use si_core::quant::{map_fraction, mul_q12_u16, q12_to_u8, snap_to_grid, Q12_ONE};
use si_core::rng::{birth_streams, Rng};
use si_core::seed;
use si_core::shelves::{self, Mutability};
use si_core::temperament::DERIV_ORDER_LEN;
use si_core::{
    aligned, layout_is_valid, EDGE_RECORD_BYTES, GENOME_REC_BYTES, RECORD_FIXED_BYTES,
    SHELF_COUNT, STAMP_BYTES,
};

#[test]
fn every_record_length_is_aligned() {
    assert!(layout_is_valid());
    for len in [
        RECORD_FIXED_BYTES,
        EDGE_RECORD_BYTES,
        STAMP_BYTES,
        CORE_BLOCK_BYTES,
        GENOME_REC_BYTES[0],
        GENOME_REC_BYTES[1],
        GENOME_REC_BYTES[2],
    ] {
        assert!(aligned(len), "{len} is not a multiple of four");
    }
    assert_eq!(RECORD_FIXED_BYTES, 80 + 32);
    assert_eq!(CORE_BLOCK_BYTES, 36);
}

#[test]
fn quant_rounds_half_up_without_division() {
    // (2048 · 4096)/4096 = 2048 exactly.
    assert_eq!(mul_q12_u16(2048, Q12_ONE), 2048);
    // A tie below one grid step rounds up, and one less does not.
    assert_eq!(mul_q12_u16(2048, 1), 1);
    assert_eq!(mul_q12_u16(2047, 1), 0);
    // Saturates rather than wrapping.
    assert_eq!(mul_q12_u16(65535, 65535), 65535);
    // 1/255 grid conversions.
    assert_eq!(q12_to_u8(Q12_ONE), 255);
    assert_eq!(q12_to_u8(0), 0);
    // Creation-time snapping is the same round-half-up rule.
    assert_eq!(snap_to_grid(1, 2), 2);
    assert_eq!(snap_to_grid(2, 2), 2);
    assert_eq!(snap_to_grid(3, 2), 4);
    // The P.2 mapping never leaves the row's legal range.
    for fraction in (0..Q12_ONE).step_by(7) {
        let value = map_fraction(fraction, 1, 64, 1);
        assert!((1..=64).contains(&value));
    }
    assert_eq!(map_fraction(0, 5, 9, 1), 5);
    assert_eq!(map_fraction(4095, 5, 9, 1), 9);
}

#[test]
fn one_seed_two_identical_streams() {
    let mut a = Rng::seeded(0x51);
    let mut b = Rng::seeded(0x51);
    for _ in 0..128 {
        assert_eq!(a.next_u32(), b.next_u32());
    }
    let mut c = Rng::seeded(0x52);
    let mut d = Rng::seeded(0x51);
    let different = (0..16).any(|_| c.next_u32() != d.next_u32());
    assert!(different, "different seeds produced identical streams");

    // T and R are separate keys from the same seed: drawing on T does not move R.
    let (mut t, mut r) = birth_streams(0x51);
    let before = r.state();
    for _ in 0..DERIV_ORDER_LEN {
        t.next_q12();
    }
    assert_eq!(r.state(), before, "stream R moved because T was read");
    let (_, mut r_control) = birth_streams(0x51);
    assert_eq!(r.next_u32(), r_control.next_u32());
}

#[test]
fn seeded_derivation_is_pure_and_closed() {
    assert!(seed::order_is_closed());
    assert_eq!(DERIV_ORDER_LEN, consts::temperament_count());
    assert_eq!(seed::derive_temperament(7), seed::derive_temperament(7));
    assert_eq!(seed::temperament_digest(7), seed::temperament_digest(7));
    assert_ne!(seed::temperament_digest(7), seed::temperament_digest(8));
    // Determinism is byte-exact, not approximate.
    assert_eq!(
        seed::derive_temperament(0x51).canonical_bytes(),
        seed::derive_temperament(0x51).canonical_bytes()
    );
}

#[test]
fn core_block_round_trips_and_checks_its_reserved_bytes() {
    let block = CoreBlock::at_birth(0x51);
    let bytes = block.encode();
    assert_eq!(bytes.len(), CORE_BLOCK_BYTES);
    assert_eq!(bytes[11], 0, "pad at offset 11 must be zero");
    assert_eq!(bytes[33], 0, "pad2 low byte must be zero");
    assert_eq!(bytes[34], 0, "pad2 high byte must be zero");
    assert_eq!(bytes[35], 0x01, "block_version");
    assert_eq!(CoreBlock::decode(&bytes).unwrap(), block);
    assert!(block.digest_matches_seed());

    let mut mutated = bytes;
    mutated[11] = 1;
    assert!(CoreBlock::decode(&mutated).is_err());
    let mut mutated = bytes;
    mutated[33] = 1;
    assert!(CoreBlock::decode(&mutated).is_err());
    let mut mutated = bytes;
    mutated[35] = 2;
    assert!(CoreBlock::decode(&mutated).is_err());

    let mut other = block;
    other.birth_seed = 0x52;
    assert!(!other.digest_matches_seed(), "digest must be tied to the seed");
}

#[test]
fn empty_state_hash_is_pinned() {
    let digest = StateHash::compose(&HashInputs::empty_at_birth(0));
    assert_eq!(
        digest.hex(),
        EMPTY_STATE_MD5_SEED_ZERO,
        "the empty-state hash changed; if that is intended, bump HASH_FORMAT and re-pin"
    );
    // The seed is inside item 9 and shapes item 7, so two individuals cannot
    // share an empty-state hash.
    let other = StateHash::compose(&HashInputs::empty_at_birth(1));
    assert_ne!(digest.hex(), other.hex());
    assert_eq!(HASH_FORMAT, VersionTags::at_birth(0).hash_format);
    assert_eq!(hex(&digest.digest()), digest.hex());
    {
        use md5::{Digest, Md5};
        let mut canonical = [0u8; 16];
        canonical.copy_from_slice(Md5::digest(si_core::hash::encode_trajectory(&[])).as_slice());
        assert_eq!(digest.sub_digest(si_core::hash::HashItem::Trajectory), canonical);
        assert_ne!(canonical, <[u8; 16]>::from(Md5::digest([])));
    }
}

#[test]
fn rng_item_records_state_and_advance_history() {
    let state = si_core::rng::Rng::seeded(3).state();
    let empty = encode_rng(&state, &[]);
    let with = encode_rng(&state, &[(1, 2)]);
    assert_eq!(empty.len(), 20 + 4);
    assert_eq!(with.len(), 20 + 4 + 8);
    assert_ne!(empty, with, "advance history is hashed");
}

#[test]
fn container_round_trips_byte_exactly() {
    let header = serde_json::json!({"shelf_versions": {"facts": 1}, "core_block": CoreBlock::at_birth(4).encode().to_vec()});
    let container = Container::new(Format::StateSnapshot, header, vec![1, 2, 3, 4, 5]);
    let bytes = container.encode().unwrap();
    let decoded = Container::decode(&bytes).unwrap();
    assert_eq!(decoded, container);
    assert_eq!(decoded.encode().unwrap(), bytes);
    assert_eq!(Format::from_magic(b"SIst"), Some(Format::StateSnapshot));
    assert_eq!(Format::from_magic(b"SIxx"), None);
}

#[test]
fn placeholders_are_the_registry_midpoints() {
    let swept: Vec<_> = consts::swept_species_rows().collect();
    // A published sweep replaces its row's placeholder; the two buckets are
    // disjoint and together cover every swept species row (11 §4 DoD 2).
    assert_eq!(swept.len(), consts::placeholder_count() + consts::published_swept_count());
    for row in consts::published_swept_rows() {
        assert_eq!(consts::placeholder_value(row), None, "{} is published and still placed", row.name);
    }
    for row in swept.iter().filter(|row| consts::swept_value(row).is_none()) {
        let row = *row;
        let placeholder = consts::placeholder_value(row).expect("swept species row has one");
        // OD-86: lo + round_half_up((hi - lo)/2), then snapped to the row's grid.
        let span = row.hi - row.lo;
        let midpoint = row.lo + (span + 1) / 2;
        let expected = snap_to_grid(midpoint, row.grid).clamp(row.lo, row.hi);
        assert_eq!(placeholder, expected, "{} placeholder is not the midpoint", row.name);
        assert!((row.lo..=row.hi).contains(&placeholder));
    }
    assert!(consts::placeholders_match_registry());
    for placeholder in consts::placeholders() {
        assert_eq!(placeholder.tag, consts::PLACEHOLDER_TAG);
    }
    // TEMPERAMENT rows are drawn, never placed.
    for row in consts::temperament_rows() {
        assert_eq!(consts::placeholder_value(row), None, "{} must not be placeheld", row.name);
    }
    // A LAW row with a value is never placed.
    for row in consts::CONST_ROWS.iter().filter(|r| r.class != Class::Species) {
        if row.law_value.is_some() {
            assert_eq!(consts::placeholder_value(row), None);
        }
    }
}

#[test]
fn registry_is_closed() {
    assert_eq!(shelves::SHELVES.len(), SHELF_COUNT);
    assert_eq!(SHELF_COUNT, 11);
    assert!(shelves::registry_is_valid());
    // 12 §2.2: shelf 9 opens with the skills build, so the reserved seats are
    // `strategies` and `events`. The count is the registry's own constant, and
    // the names are asserted because "which two" is the claim, not "how many".
    assert_eq!(
        shelves::SHELVES.iter().filter(|s| s.mutability == Mutability::Reserved).count(),
        shelves::RESERVED_SHELF_COUNT
    );
    assert_eq!(shelves::RESERVED_SHELF_COUNT, 2);
    let reserved: Vec<&str> = shelves::SHELVES
        .iter()
        .filter(|s| s.mutability == Mutability::Reserved)
        .map(|s| s.shelf_name)
        .collect();
    assert_eq!(reserved, ["strategies", "events"]);
    assert!(shelves::is_l3_live(si_core::addr::SHELF_SKILLS));
    assert!(si_core::patterns::table_is_closed());
    assert_eq!(si_core::patterns::PATTERN_TABLE.len(), 10);
    assert!(si_core::patterns::PATTERN_TABLE.iter().any(|p| *p == si_core::patterns::PatternId::PTeach));
    assert_eq!(consts::temperament_count(), 18);
    assert_eq!(consts::CONST_ROWS.iter().filter(|r| r.name == "DERIV_ORDER").count(), 1);
    assert_eq!(si_core::CONF_MAX, 99);
    assert_eq!(si_core::SIGNAL_KIND_COUNT, 6);
    assert_eq!(si_core::DOOR_COUNT, 6);
}

#[test]
fn string_allowlist_is_the_spec_table() {
    assert_eq!(si_core::strings::FUNCTION_WORDS.len(), 15);
    assert_eq!(si_core::strings::CORRECTION_MARKERS.len(), 6);
    assert_eq!(si_core::strings::PATTERN_FRAME_NAMES.len(), 9);
    assert_eq!(si_core::SOURCE_LITERALS, ["owner", "grimm"]);
    assert_eq!(si_core::TEACH_LITERALS, ["/teach", "=>", "owner-teach"]);
    let frames: Vec<_> = si_core::patterns::PATTERN_TABLE
        .iter()
        .filter(|p| **p != si_core::patterns::PatternId::Undecoded)
        .map(|p| p.name())
        .collect();
    assert_eq!(frames, si_core::strings::PATTERN_FRAME_NAMES);
    assert!(si_core::strings::vocabulary().contains(&"/teach"));
    assert!(si_core::strings::vocabulary().contains(&"=>"));
}
