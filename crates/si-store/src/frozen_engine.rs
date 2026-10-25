//! 07 §9 W.1 / 00 §6.2: the loader body. `layers_frozen` holds the guard and the
//! type; this file is the only place in the crate that opens a sealed artifact,
//! and it exists only in a build compiled with `engine`. A shipped CONFIG SI
//! binary has no path to `open`, which is the build half of G5c — the runtime
//! half is the guard in the entry, and the third is the file's physical absence
//! from a release artifact (00 §6.3).
//!
//! The v1 body reads the artifact once with `fs::read` and keeps the decoded
//! canonical file. What 07 §9 forbids by name is materializing a *merged* edge
//! arena per wake — that copy is the 5.5·10⁸-byte figure the load budget is about
//! — and the merged web is never built here: W.3's walk reads two address spaces
//! through cursors and writes nothing. Persistently mapped `HOST_VISIBLE |
//! HOST_COHERENT` buffers over exactly these bytes are `si-vulkan`'s
//! `genome_residency` (W.5, 02 §7, **OD-44**: the v1 walker is CPU), so the
//! mapping arrives with the viewer stage rather than being faked here.

use std::path::Path;

use si_core::genome::GenomeHeader;

use crate::genome_file::{check_seal, GenomeFile, Provenance};
use crate::layers_frozen::LoadError;
use crate::opens;

/// W.1: the read-only open, the seal check and the tag comparison. The artifact
/// path and the pinned tag arrive as two arguments, and the header, batch table
/// and file go back the same way, so that the frozen path is named in exactly one
/// file (05 §20's config guard) and this body holds no manifest type.
pub fn open(
    artifact: &Path,
    pinned_tag: u64,
) -> Result<(GenomeHeader, Vec<Provenance>, GenomeFile), LoadError> {
    let text = artifact.display().to_string();
    opens::record(&text, "open-ro");
    let bytes = std::fs::read(artifact)
        .map_err(|error| LoadError::Artifact(format!("{text}: {error}")))?;
    // A10 before anything else: an artifact edited after sealing is not decoded,
    // let alone walked (07 §13).
    let (header, provenance, file) =
        check_seal(&bytes).map_err(|error| LoadError::Artifact(error.to_string()))?;
    if header.version_tag != pinned_tag {
        return Err(LoadError::TagMismatch { pinned: pinned_tag, sealed: header.version_tag });
    }
    Ok((header, provenance, file))
}
