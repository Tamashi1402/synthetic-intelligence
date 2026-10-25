p = 'crates/si-learn/src/gate.rs'
s = open(p, encoding='utf-8').read()

old = """pub const ASK_ORIGIN_SENSE: u8 = 0;
pub const ASK_ORIGIN_IDLE: u8 = 1;
"""
new = """pub const ASK_ORIGIN_SENSE: u8 = 0;
pub const ASK_ORIGIN_IDLE: u8 = 1;
/// engineering/12 §2.4: the arc adds one origin value in `key_field`'s low byte
/// and two gap kinds in its high byte — no new column, no envelope byte.
pub const ASK_ORIGIN_NOMINATION: u8 = 2;
pub const GAP_CLUSTER_NOMINATION: u8 = 5;
pub const GAP_MOVE_NOMINATION: u8 = 6;
"""
assert old in s
s = s.replace(old, new, 1)

old = """impl AskRow {
    /// The id a resolving row is stored under (`resolved_by` rides
    /// `bind_output_id` on the resolution's version row, 06 §2).
    pub fn resolved_by(&self) -> u32 {
        self.record.envelope.bind_output_id
    }
}
"""
new = """impl AskRow {
    /// The id a resolving row is stored under (`resolved_by` rides
    /// `bind_output_id` on the resolution's version row, 06 §2).
    pub fn resolved_by(&self) -> u32 {
        self.record.envelope.bind_output_id
    }

    /// `key_field`'s low byte — the lane, derived and never stored separately
    /// (12 §2.4: OD-128's "lane field" is this half).
    pub fn origin(&self) -> u8 {
        (self.record.body.key_field & 0x00FF) as u8
    }

    /// `key_field`'s high byte — the gap kind that asked.
    pub fn gap_kind(&self) -> u8 {
        (self.record.body.key_field >> 8) as u8
    }
}
"""
assert old in s
s = s.replace(old, new, 1)

old = """pub fn latest_open_ask(store: &Lattice, now: i32) -> Result<Option<AskRow>> {
    let open = open_asks(store, now)?;
    Ok(open
        .into_iter()
        .max_by_key(|row| (row.asked_at, row.row_id)))
}
"""
new = """pub fn latest_open_ask(store: &Lattice, now: i32) -> Result<Option<AskRow>> {
    let open = open_asks(store, now)?;
    // OD-128 / 12 §2.4: an `origin = 2` nomination ask is answered only by the
    // owner lane that mints its point (B17). Until that lane exists it must not
    // capture a bare reply that was answering an ordinary ask, or a detector's
    // proposal would silently satisfy itself.
    Ok(open
        .into_iter()
        .filter(|row| row.origin() != ASK_ORIGIN_NOMINATION)
        .max_by_key(|row| (row.asked_at, row.row_id)))
}
"""
assert old in s
s = s.replace(old, new, 1)

open(p, 'w', encoding='utf-8').write(s)
print('ok')
