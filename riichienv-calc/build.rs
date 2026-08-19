use std::{fs, process::Command};

fn main() {
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");
    println!("cargo:rerun-if-env-changed=SOURCE_GIT_SHA");
    println!("cargo:rerun-if-changed=SOURCE_GIT_SHA");
    println!("cargo:rerun-if-changed=../.git/HEAD");
    let sha = std::env::var("SOURCE_GIT_SHA")
        .ok()
        .or_else(|| {
            fs::read_to_string("SOURCE_GIT_SHA")
                .ok()
                .map(|value| value.trim().to_owned())
                .filter(|value| value != "unknown" && !value.is_empty())
        })
        .or_else(|| std::env::var("GITHUB_SHA").ok())
        .or_else(|| {
            Command::new("git")
                .args(["rev-parse", "HEAD"])
                .output()
                .ok()
                .filter(|output| output.status.success())
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|value| value.trim().to_owned())
        });
    if let Some(sha) = sha {
        println!("cargo:rustc-env=RIICHIENV_CORE_GIT_SHA={sha}");
    }
}
