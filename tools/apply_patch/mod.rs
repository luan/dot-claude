mod apply;
mod diff;
mod parser;
mod seek_sequence;

pub use apply::{ApplyPatchError, ChangeType, apply};
pub(crate) use seek_sequence::seek_sequence;

/// Maximum accepted patch body length. Guards against unbounded allocation
/// from either `ct tool apply-patch` (stdin) or the MCP handler (JSON body).
pub const MAX_PATCH_SIZE_BYTES: usize = 16 * 1024 * 1024;
