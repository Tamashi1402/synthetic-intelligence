//! 00 §6.1 / 07 §12 A11–A12: the open register.
//!
//! It is an in-process register of the paths this build can open for frozen-layer
//! or pinned-source work, not an `strace` of the OS: what it can prove is exactly
//! the claim the gates make — that `Config::Si` returns before a path is formed
//! and that the converter touches no lattice, no `runs/` and no gate directory.
//! Enabled by `SI_TRACE_OPENS=1` (or `--trace-opens`, which sets it) so a normal
//! run pays nothing; the audit legs set it and read the log back.

use std::sync::Mutex;
use std::sync::OnceLock;

static LOG: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn log() -> &'static Mutex<Vec<String>> {
    LOG.get_or_init(Default::default)
}

/// Tracing is on whenever the environment says so; recording is cheap and the
/// only consumer is an audit leg.
pub fn enabled() -> bool {
    std::env::var_os("SI_TRACE_OPENS").is_some_and(|value| value != "0")
}

/// Called by every file-open site in the frozen loader body and the converter.
pub fn record(path: &str, mode: &str) {
    if !enabled() {
        return;
    }
    if let Ok(mut guard) = log().lock() {
        guard.push(format!("{mode} {path}"));
    }
}

/// The trace so far, in open order.
pub fn trace() -> Vec<String> {
    log().lock().map(|guard| guard.clone()).unwrap_or_default()
}

pub fn clear() {
    if let Ok(mut guard) = log().lock() {
        guard.clear();
    }
}

/// The audit's own question, asked of the trace: did anything name a frozen-layer
/// path? Only the loader entry can name one, so the caller supplies that path
/// rather than re-deriving a filename here (00 §6.3).
pub fn frozen_opens(artifact_path: &str) -> Vec<String> {
    trace()
        .into_iter()
        .filter(|line| line.split_once(' ').map(|(_, path)| path).unwrap_or("") == artifact_path)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_register_records_clears_and_filters() {
        let previous = std::env::var_os("SI_TRACE_OPENS");
        // Required so set_var and remove_var cannot race with another test.
        std::env::set_var("SI_TRACE_OPENS", "1");
        clear();
        record("data/genome/fixture/base.sigen", "open-ro");
        record("runs/session/SIck", "open-rw");
        let trace = trace();
        assert_eq!(trace.len(), 2);
        assert_eq!(frozen_opens("data/genome/fixture/base.sigen").len(), 1);
        assert!(frozen_opens("nothing/here").is_empty());
        clear();
        assert!(super::trace().is_empty());
        match previous {
            Some(value) => std::env::set_var("SI_TRACE_OPENS", value),
            None => std::env::remove_var("SI_TRACE_OPENS"),
        }
    }
}
