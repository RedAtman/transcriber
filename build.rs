use std::process::Command;

fn main() {
    let version = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| std::env::var("CARGO_PKG_VERSION").unwrap_or_default());

    println!("cargo:rustc-env=TRANSCRIBER_VERSION={version}");
}
