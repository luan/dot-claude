mod apply;
mod diff;
mod parser;
mod seek_sequence;
pub mod telemetry;
#[cfg(test)]
mod tests;

pub use apply::{
    ApplyFailure, ApplyOutcome, ApplyPatchError, ChangeType, FileChange, HunkFuzzy, HunkRegion,
    apply,
};
pub use telemetry::{
    AnchorAttempt, CallRecord, FileCallEntry, Fingerprint, Telemetry, enrich, sha1_hex,
};

/// Maximum accepted patch body length. Guards against unbounded allocation
/// from either `ct tool apply-patch` (stdin) or the MCP handler (JSON body).
pub const MAX_PATCH_SIZE_BYTES: usize = 16 * 1024 * 1024;
