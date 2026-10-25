//! 01 §2: the closed shelf registry. A twelfth `x` is a design change (R-12).

use crate::SHELF_COUNT;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    L1Frozen,
    L2Frozen,
    L3Live,
    Reserved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mutability {
    ReVersioned,
    AppendOnly,
    Reserved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShelfEntry {
    pub shelf_id: u16,
    pub shelf_name: &'static str,
    pub row_schema_version: u16,
    pub layer: Layer,
    pub mutability: Mutability,
}

pub const SHELVES: [ShelfEntry; SHELF_COUNT] = [
    ShelfEntry { shelf_id: 1, shelf_name: "dictionary", row_schema_version: 1, layer: Layer::L1Frozen, mutability: Mutability::ReVersioned },
    ShelfEntry { shelf_id: 2, shelf_name: "relations", row_schema_version: 1, layer: Layer::L2Frozen, mutability: Mutability::ReVersioned },
    ShelfEntry { shelf_id: 3, shelf_name: "rules", row_schema_version: 1, layer: Layer::L2Frozen, mutability: Mutability::ReVersioned },
    ShelfEntry { shelf_id: 4, shelf_name: "s_r_pairs", row_schema_version: 1, layer: Layer::L3Live, mutability: Mutability::AppendOnly },
    ShelfEntry { shelf_id: 5, shelf_name: "facts", row_schema_version: 1, layer: Layer::L3Live, mutability: Mutability::AppendOnly },
    ShelfEntry { shelf_id: 6, shelf_name: "links", row_schema_version: 1, layer: Layer::L3Live, mutability: Mutability::AppendOnly },
    ShelfEntry { shelf_id: 7, shelf_name: "questions", row_schema_version: 1, layer: Layer::L3Live, mutability: Mutability::AppendOnly },
    ShelfEntry { shelf_id: 8, shelf_name: "context", row_schema_version: 1, layer: Layer::L3Live, mutability: Mutability::AppendOnly },
    // 12 §2.2: shelf 9 goes live with the skills build. The id was always
    // registered, so `SHELF_NAMES` does not move (docs/25 §25.11 note 2).
    ShelfEntry { shelf_id: 9, shelf_name: "skills", row_schema_version: 1, layer: Layer::L3Live, mutability: Mutability::AppendOnly },
    ShelfEntry { shelf_id: 10, shelf_name: "strategies", row_schema_version: 0, layer: Layer::Reserved, mutability: Mutability::Reserved },
    ShelfEntry { shelf_id: 11, shelf_name: "events", row_schema_version: 0, layer: Layer::Reserved, mutability: Mutability::Reserved },
];

/// 12 §2.2: `strategies` and `events` are the two seats that stay reserved-empty;
/// a thirteenth `x` fails the build (R-12).
pub const RESERVED_SHELF_COUNT: usize = 2;

/// 01 §2: the registry is the only answer to "may a row be appended here?". A
/// store that hard-coded a range would drift the moment a reserved seat opens.
pub const fn is_l3_live(shelf_id: u16) -> bool {
    let mut index = 0;
    while index < SHELVES.len() {
        if SHELVES[index].shelf_id == shelf_id {
            return matches!(SHELVES[index].layer, Layer::L3Live);
        }
        index += 1;
    }
    false
}

pub fn entry(shelf_id: u16) -> Option<&'static ShelfEntry> {
    SHELVES.iter().find(|e| e.shelf_id == shelf_id)
}

pub fn is_reserved(shelf_id: u16) -> bool {
    entry(shelf_id).map(|e| e.mutability == Mutability::Reserved).unwrap_or(false)
}

pub fn registry_is_valid() -> bool {
    if SHELVES.len() != SHELF_COUNT {
        return false;
    }
    let mut ids: Vec<u16> = SHELVES.iter().map(|e| e.shelf_id).collect();
    ids.sort_unstable();
    ids.iter().enumerate().all(|(i, id)| *id as usize == i + 1)
        && SHELVES.iter().filter(|e| e.mutability == Mutability::Reserved).count() == RESERVED_SHELF_COUNT
}
