//! 08 §5.1: the layout question, answered as a pure function.
//!
//! Positions come from exactly one place in this build — identity. A layout learned
//! from the corpus's own vectors is R-14 with a camera attached, and a force-directed
//! relaxation over the live web is a second computation the loop does not do, so
//! the viewer would be *deriving* structure and paying a per-frame cost the
//! design never budgeted. What is left is [`pos`]: one mix of the point handle,
//! three rotations of it, and a shelf-derived shell. Stable across runs, zero
//! stored bytes, nothing learned, and a replay puts every point exactly where it
//! was.
//!
//! The consequence the spec states rather than hides: **the picture is
//! structural, not semantic.** Nearby points are nearby because the registry and
//! the hash say so, not because their meaning is close.

use si_core::shelves::SHELVES;

/// Grid extent per axis, in quantized units. A display constant, not a schema
/// one: it is the radius the camera is written against, and inverting this
/// mapping back into state is what §5.2's LUT rule forbids.
///
/// Design target, **UNMEASURED** against any device.
pub const EXTENT: i32 = 1 << 14;

/// The shell bands: shelf `x` lands in band `x % SHELLS`. The registry is the
/// source of the enumeration, so a reserved shelf still has a shell and O2's
/// "a reserved shelf renders as reserved, v2" is the same fact on both panes.
pub const SHELLS: usize = 12;

/// The innermost shell radius, in the same quantized units as [`EXTENT`]. A
/// shelf-derived band has to be wide enough that two points of the same shelf
/// are legibly in it, and narrow enough that the outer shells do not vanish.
pub const SHELL_BAND: i32 = EXTENT / (SHELLS as i32 + 2);

/// One mix, the same idiom [01 §1] reduces `coords_of` to: a multiply, a xor-shift,
/// a multiply. Identity, not security — nothing here infers similarity, and no
/// distance in this picture is a distance in meaning (I1).
const fn mix(value: u64) -> u64 {
    let mut x = value ^ (value >> 30);
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

/// A point's place in the preview: three rotations of one mix ([01 §1]'s
/// `coords_of` shape), folded into the shell band its shelf owns.
///
/// `shelf` is `None` for a point the viewer knows only as a walk handle — a lit
/// node of a round that has no row yet, or a dark slot. Those draw in the shell
/// the mix alone gives them, which is the honest rendering: the loop has not said
/// which shelf owns it, so the picture may not either.
pub fn pos(handle: u32, shelf: Option<u16>) -> [i32; 3] {
    let base = mix(u64::from(handle).wrapping_add(1));
    let band = shelf.map_or(0, |x| shell_of(x));
    let spread = EXTENT - band;
    [
        offset(base, 0, band, spread),
        offset(base, 21, band, spread),
        offset(base, 42, band, spread),
    ]
}

/// The shell radius a shelf id owns, in grid units from the origin.
pub fn shell_of(shelf_x: u16) -> i32 {
    let shells = SHELVES.len().max(SHELLS) as i32;
    let index = shelf_x as i32 % shells;
    SHELL_BAND + index * SHELL_BAND
}

/// §6 O2's row label: the registry's own name, and the reserved shelves named as
/// reserved rather than skipped — R-12's alarm is not allowed to be patched
/// around by a pane that quietly lists eight rows where the registry has eleven.
pub fn shelf_label(shelf_x: u16) -> String {
    match si_core::shelves::entry(shelf_x) {
        Some(entry) if si_core::shelves::is_reserved(shelf_x) => {
            format!("{} (reserved, v2)", entry.shelf_name)
        }
        Some(entry) => entry.shelf_name.to_string(),
        None => format!("shelf {shelf_x} (not in the registry)"),
    }
}

/// `axis` is the rotation: 0, 21 and 42 bits of the same 64-bit mix, so the three
/// coordinates of one point are three views of one number and nothing else.
fn offset(base: u64, axis: u32, band: i32, spread: i32) -> i32 {
    let rotated = base >> axis ^ base.rotate_left(axis);
    let within = (rotated % (spread.max(1) as u64)) as i32;
    band + within - EXTENT / 2
}

/// The walk's own adjacency, drawn as lines: an edge is a pair of [`pos`]
/// results, never an interpolation between them. §5.2's per-round draw is one
/// line per traversed edge at the loop's cadence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Segment {
    pub from: u32,
    pub to: u32,
    /// The traversal's effective weight as the round paid it (`weight_eff_q12`),
    /// a grid int. The display LUT maps it to an intensity; the *number* lives in
    /// the inspector row, because a mapping that faded a live-but-dim edge into
    /// invisibility would hide "unroutable but still citable" on the one screen
    /// the owner watches (§5.2).
    pub weight_q12: i32,
    /// OWNED or BASE, from the record's provenance bit — never from a name.
    pub owned: bool,
}

impl Segment {
    /// The minimum visible intensity a dimmed edge still draws at, in the same
    /// units as a q12 weight. Presentation only, and the reason it is a constant
    /// rather than a curve is §5.2's rule: the floor of a LUT is not the floor of
    /// the schema.
    pub const MIN_VISIBLE_Q12: i32 = 1;

    /// The LUT, forward only. `None` says the value is outside the display range
    /// and the panel prints the number instead of guessing a colour.
    pub fn intensity(&self) -> Option<u8> {
        if self.weight_q12 < 0 {
            return None;
        }
        let scaled = self.weight_q12.max(Self::MIN_VISIBLE_Q12).min(i32::from(u8::MAX));
        Some(scaled as u8)
    }
}
