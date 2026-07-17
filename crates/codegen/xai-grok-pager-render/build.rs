use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_THEME_COUNT: usize = 340;
const EXPECTED_UPSTREAM_REVISION: &str = "b385044250f1ed3c9379ab34a8fe82f02fdffaa4";
const CATEGORIES: &[(&str, usize)] = &[
    ("base16", 178),
    ("standard", 134),
    ("special_edition", 8),
    ("stradicat", 1),
    ("warp_bundled", 19),
];

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    // Keep the corpus below the crate root so `cargo package` includes every
    // source consumed by this build script. Generated includes remain
    // manifest-relative and therefore do not capture the build checkout's
    // absolute path.
    let catalog_root = manifest_dir.join("assets/warp-themes");
    println!("cargo:rerun-if-changed={}", catalog_root.display());
    let revision_path = catalog_root.join("UPSTREAM_REVISION");
    println!("cargo:rerun-if-changed={}", revision_path.display());
    let revision = fs::read_to_string(&revision_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", revision_path.display()));
    assert_eq!(
        revision.trim(),
        EXPECTED_UPSTREAM_REVISION,
        "vendored Warp revision marker and audited build baseline differ"
    );

    let mut entries = Vec::new();
    for (category, expected_count) in CATEGORIES {
        let dir = catalog_root.join(category);
        let read_dir = fs::read_dir(&dir).unwrap_or_else(|e| {
            panic!("failed to read Warp theme directory {}: {e}", dir.display())
        });
        let category_start = entries.len();
        for item in read_dir {
            let path = item.expect("Warp theme directory entry").path();
            let metadata = fs::symlink_metadata(&path)
                .unwrap_or_else(|error| panic!("failed to inspect {}: {error}", path.display()));
            assert!(
                metadata.is_file() && !metadata.file_type().is_symlink() && is_yaml(&path),
                "Warp category directories may contain only regular YAML/YML files: {}",
                path.display()
            );
            let stem = path
                .file_stem()
                .and_then(OsStr::to_str)
                .expect("Warp theme filename must be UTF-8")
                .to_owned();
            entries.push((
                format!("{category}/{stem}"),
                (*category).to_owned(),
                stem,
                path.file_name()
                    .and_then(OsStr::to_str)
                    .expect("Warp theme filename must be UTF-8")
                    .to_owned(),
            ));
        }
        assert_eq!(
            entries.len() - category_start,
            *expected_count,
            "vendored Warp category {category} must contain exactly {expected_count} YAML themes"
        );
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        entries.len(),
        EXPECTED_THEME_COUNT,
        "vendored Warp catalog must contain exactly {EXPECTED_THEME_COUNT} YAML themes"
    );

    let mut generated =
        String::from("pub static EMBEDDED_WARP_THEMES: &[EmbeddedWarpThemeSource] = &[\n");
    for (id, category, stem, filename) in entries {
        let relative = format!("assets/warp-themes/{category}/{filename}");
        let include_suffix = format!("/{relative}");
        let path = manifest_dir.join(&relative);
        println!("cargo:rerun-if-changed={}", path.display());
        generated.push_str("    EmbeddedWarpThemeSource {\n");
        generated.push_str(&format!("        id: {id:?},\n"));
        generated.push_str(&format!("        category: {category:?},\n"));
        generated.push_str(&format!("        stem: {stem:?},\n"));
        generated.push_str(&format!(
            "        yaml: include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), {include_suffix:?})),\n"
        ));
        generated.push_str("    },\n");
    }
    generated.push_str("];\n");

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("warp_theme_catalog.rs");
    fs::write(&out, generated).unwrap_or_else(|e| panic!("failed to write {}: {e}", out.display()));
}

fn is_yaml(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "yaml" | "yml"))
}
