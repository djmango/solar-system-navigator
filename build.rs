//! Download planet textures before build when missing (gitignored assets).

use std::path::{Path, PathBuf};
use std::process::Command;

const REQUIRED_TEXTURES: &[&str] = &[
    "assets/textures/earth.jpg",
    "assets/textures/mars.jpg",
    "assets/textures/mercury.jpg",
    "assets/textures/moon.jpg",
    "assets/textures/sun.jpg",
    "assets/textures/venus.jpg",
];

fn manifest_dir() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"))
}

fn textures_ready(root: &Path) -> bool {
    REQUIRED_TEXTURES.iter().all(|rel| root.join(rel).is_file())
}

fn fetch_textures(root: &Path) {
    if std::env::var("SOLAR_SKIP_TEXTURE_FETCH").ok().as_deref() == Some("1") {
        println!("cargo:warning=SOLAR_SKIP_TEXTURE_FETCH=1 — textures not downloaded");
        return;
    }

    let script = root.join("scripts/fetch_textures.sh");
    if !script.is_file() {
        println!("cargo:warning=Missing scripts/fetch_textures.sh — cannot auto-fetch textures");
        return;
    }

    println!("cargo:warning=Downloading planet textures (first build may take a minute)...");
    let status = Command::new("bash").arg(&script).current_dir(root).status();

    match status {
        Ok(s) if s.success() => {
            if textures_ready(root) {
                println!("cargo:warning=Planet textures ready in assets/textures/");
            } else {
                println!(
                    "cargo:warning=Texture fetch finished but required files are still missing"
                );
            }
        }
        Ok(s) => {
            println!(
                "cargo:warning=Texture fetch failed (exit {}). Run: ./scripts/fetch_textures.sh",
                s.code().unwrap_or(-1)
            );
        }
        Err(err) => {
            println!("cargo:warning=Could not run fetch_textures.sh: {err}");
        }
    }
}

fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH").ok().as_deref() == Some("wasm32") {
        println!("cargo:warning=WASM build — skipping texture fetch (bundle assets/ for Trunk)");
        return;
    }

    let root = manifest_dir();

    println!("cargo:rerun-if-changed=scripts/fetch_textures.sh");
    println!("cargo:rerun-if-changed=assets/scenarios/");

    for rel in REQUIRED_TEXTURES {
        let path = root.join(rel);
        println!("cargo:rerun-if-changed={rel}");
        if !path.is_file() {
            fetch_textures(&root);
            break;
        }
    }

    if !textures_ready(&root) {
        println!("cargo:warning=Required planet textures still missing after fetch attempt.");
        for rel in REQUIRED_TEXTURES {
            if !root.join(rel).is_file() {
                println!("cargo:warning=  - {rel}");
            }
        }
    }
}
