use std::process::Command;

fn main() {
    let pkg_version = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();

    // Use CARGO_PKG_VERSION as the version prefix, with git commit count + hash suffix.
    // This avoids `git describe` picking the wrong tag (nearest by distance, not by version).
    // Output format: v<version>-<count>-g<hash>[-dirty]
    let version = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty", "--long"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .map(|desc| {
            // desc: <tag>-<count>-g<hash>[-dirty]
            // Replace tag prefix with CARGO_PKG_VERSION
            if let Some((_, rest)) = desc.split_once('-') {
                // Strip "-0-g<hash>" when on a clean tag (no dirty, count=0)
                if rest.starts_with("0-") && !rest.ends_with("-dirty") {
                    format!("v{pkg_version}")
                } else {
                    format!("v{pkg_version}-{rest}")
                }
            } else {
                format!("v{pkg_version}")
            }
        })
        .unwrap_or(pkg_version);

    println!("cargo:rustc-env=TRANSCRIBER_VERSION={version}");
}
