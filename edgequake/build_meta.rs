// Build metadata shared by the workspace binary and `edgequake-api`.
//
// Included from each package's `build.rs`. Cargo treats a missing
// `rerun-if-changed` path as dirty on every invocation. The git checkout
// lives at the repository root, not inside this Cargo workspace, so a
// hard-coded `.git/HEAD` never exists and both crates recompiled on every
// `cargo run` (including the second one `make dev` starts after migrate).

use std::path::{Path, PathBuf};
use std::process::Command;

/// Emit git hash, branch, and build time. Re-run only when this checkout's
/// HEAD actually moves. Never emit a path that is not on disk.
pub fn emit_build_metadata() {
    let build_timestamp = utc_now();
    println!("cargo:rustc-env=EDGEQUAKE_BUILD_TIMESTAMP={build_timestamp}");

    let git_hash = git_line(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=EDGEQUAKE_GIT_HASH={git_hash}");

    let git_branch =
        git_line(&["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=EDGEQUAKE_GIT_BRANCH={git_branch}");

    let build_number = build_timestamp
        .replace(['-', ':'], "")
        .replace('T', ".")
        .replace('Z', "");
    println!("cargo:rustc-env=EDGEQUAKE_BUILD_NUMBER={build_number}");

    for path in git_watch_paths() {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    // Keeps the script from scanning the whole package when git is unavailable.
    println!("cargo:rerun-if-changed=build.rs");
}

/// Files whose mtime means the embedded git hash changed.
///
/// `HEAD` moves on checkout. `logs/HEAD` is appended on commit, reset, and
/// rebase. The shared `refs/` tree is intentionally not watched: another
/// branch's update would rebuild this checkout, and a missing path would
/// rebuild it forever.
fn git_watch_paths() -> Vec<PathBuf> {
    let Some(git_dir) = git_line(&["rev-parse", "--absolute-git-dir"]) else {
        return Vec::new();
    };
    existing_git_watch_paths(Path::new(&git_dir))
}

fn existing_git_watch_paths(git_dir: &Path) -> Vec<PathBuf> {
    let mut tracked = Vec::new();
    let head = git_dir.join("HEAD");
    if head.is_file() {
        tracked.push(head);
    }
    let reflog = git_dir.join("logs").join("HEAD");
    if reflog.is_file() {
        tracked.push(reflog);
    }
    tracked
}

fn utc_now() -> String {
    let output = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output();
    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => "unknown".to_string(),
    }
}

fn git_line(args: &[&str]) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        cmd.current_dir(dir);
    }
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let line = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if line.is_empty() {
        None
    } else {
        Some(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch_paths_skip_files_that_are_not_on_disk() {
        let dir = std::env::temp_dir().join(format!("eq-git-watch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("logs")).unwrap();
        std::fs::write(dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();

        let head_only = existing_git_watch_paths(&dir);
        assert_eq!(head_only, vec![dir.join("HEAD")]);

        std::fs::write(dir.join("logs").join("HEAD"), "log\n").unwrap();
        let both = existing_git_watch_paths(&dir);
        assert_eq!(both, vec![dir.join("HEAD"), dir.join("logs").join("HEAD")]);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
