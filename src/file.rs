//! Files managed under the typewriter system

use serde::Deserialize;

/// File in typewriter config that should be tracked and updated
/// appropriately on apply.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TrackedFile {
    // Source file to read from
    file: String,

    // Destination location to write to
    destination: String,
}
