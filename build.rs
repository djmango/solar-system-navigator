//! Warn when planet textures are missing (they are gitignored; run fetch_textures.sh).

fn main() {
    let required = [
        "assets/textures/earth.jpg",
        "assets/textures/mars.jpg",
        "assets/textures/sun.jpg",
        "assets/textures/venus.jpg",
    ];

    let missing: Vec<_> = required
        .iter()
        .filter(|path| !std::path::Path::new(path).is_file())
        .copied()
        .collect();

    if !missing.is_empty() {
        println!("cargo:warning=Planet textures are missing:");
        for path in &missing {
            println!("cargo:warning=  - {path}");
        }
        println!("cargo:warning=Run: ./scripts/fetch_textures.sh");
        println!("cargo:warning=Or for release: ./scripts/build-release.sh");
    }

    for path in required {
        println!("cargo:rerun-if-changed={path}");
    }
    println!("cargo:rerun-if-changed=assets/scenarios/");
}
