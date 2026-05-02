use std::process::Command;

fn main() {
    // Derive version from the most recent git tag, falling back to Cargo.toml.
    let version = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| std::env::var("CARGO_PKG_VERSION").unwrap_or_default());

    // Re-run build when HEAD or the current branch ref changes (new commits, branch switches).
    println!("cargo:rustc-env=TRANSCRIBER_VERSION={version}");
    println!("cargo:rerun-if-changed=.git/HEAD");
    let _ = std::fs::read_to_string(".git/HEAD")
        .ok()
        .and_then(|h| h.strip_prefix("ref: ").map(|s| s.trim().to_string()))
        .map(|ref_path| println!("cargo:rerun-if-changed=.git/{ref_path}"));
}
