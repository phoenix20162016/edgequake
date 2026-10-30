//! Build script for EdgeQuake — embeds git hash and build time.
//!
//! The health API reads the same variables from the `edgequake-api` build
//! script. This copy keeps the binary package on the same rebuild rule so a
//! missing `.git` path cannot force `edgequake` to compile on every `cargo run`.

include!("build_meta.rs");

fn main() {
    emit_build_metadata();
}
