//! 11 §B14 DoD 1's graph snapshot / 12 §1.1–§1.2.
//!
//! `check_crystallize_no_commit` already denies the *forbidden* manifest names at
//! the stage gate, and a `cargo test` that re-implemented that denial would be a
//! second copy of the same sentence. What nothing asserted yet is the other half
//! of "exactly the edges": the positive set, `si-silt`'s manifest (which that
//! check does not scan), and the four reciprocal rows — the thought path, the
//! write path and the surface may not ask the arc for a decision, so no crate
//! other than `si-session` may name the three.
//!
//! The bytes are read at compile time, so a manifest edited to break the law
//! breaks the build of this test rather than quietly widening a graph.

/// The `si-*` crates one manifest section declares, in file order.
fn si_edges(manifest: &str, section: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut current = String::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            current = line.trim_matches(|c| c == '[' || c == ']').to_string();
            continue;
        }
        if current != section {
            continue;
        }
        let Some(key) = line.split_once('=').map(|(key, _)| key.trim()) else { continue };
        let key = key.strip_suffix(".workspace").unwrap_or(key);
        if key.starts_with("si-") {
            found.push(key.to_string());
        }
    }
    found
}

#[test]
fn the_three_reader_packages_declare_exactly_the_edges_of_12_1_1() {
    // core, store (read-only) — NO si-learn, NO si-affect, NO si-walk.
    assert_eq!(
        si_edges(include_str!("../../si-crystallize/Cargo.toml"), "dependencies"),
        ["si-core".to_string(), "si-store".to_string()]
    );
    // core only — the table keys point handles, so it cannot name a row at all.
    assert_eq!(
        si_edges(include_str!("../../si-silt/Cargo.toml"), "dependencies"),
        ["si-core".to_string()]
    );
    assert_eq!(
        si_edges(include_str!("../../si-skills/Cargo.toml"), "dependencies"),
        ["si-core".to_string(), "si-store".to_string()]
    );
    // A dev-dependency is still an edge at the point the law says "only".
    for manifest in [
        include_str!("../../si-crystallize/Cargo.toml"),
        include_str!("../../si-silt/Cargo.toml"),
        include_str!("../../si-skills/Cargo.toml"),
    ] {
        assert!(si_edges(manifest, "dev-dependencies").is_empty(), "a reader tests with a writer");
    }
}

#[test]
fn nothing_but_the_session_can_ask_the_arc_for_a_decision() {
    const ARC: [&str; 3] = ["si-crystallize", "si-silt", "si-skills"];
    // 12 §1.1's reciprocal rows, one manifest each.
    for (name, manifest) in [
        ("si-walk", include_str!("../../si-walk/Cargo.toml")),
        ("si-learn", include_str!("../../si-learn/Cargo.toml")),
        ("si-front", include_str!("../../si-front/Cargo.toml")),
        ("si-read", include_str!("../../si-read/Cargo.toml")),
    ] {
        for section in ["dependencies", "dev-dependencies"] {
            let declared = si_edges(manifest, section);
            for reacher in &ARC {
                assert!(
                    !declared.iter().any(|edge| edge == reacher),
                    "{name} declares `{reacher}`; the loop would be asking a detector to decide (12 §1.1)"
                );
            }
        }
    }
    // `si-session` already imports everything in the loop, so it is the one caller.
    let session = si_edges(include_str!("../../si-session/Cargo.toml"), "dependencies");
    for member in ARC {
        assert!(session.iter().any(|edge| edge == member), "{member} has no caller");
    }
}
