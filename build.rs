use std::process::Command;

fn main() {
    // Try to get git version
    let git_version = Command::new("git")
        .args(&["describe", "--tags", "--dirty"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string());

    // Use git version if available, otherwise fall back to Cargo.toml version
    let version = git_version.unwrap_or_else(|| {
        env!("CARGO_PKG_VERSION").to_string()
    });

    println!("cargo:rustc-env=BUILD_VERSION={}", version);

    // Re-run build script if git state changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/tags");
}
