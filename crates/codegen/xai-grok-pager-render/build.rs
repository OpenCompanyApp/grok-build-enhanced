use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsStr;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_SCHEMA_VERSION: u32 = 1;
const EXPECTED_THEME_COUNT: usize = 340;
const EXPECTED_UPSTREAM_SOURCE: &str = "https://github.com/warpdotdev/themes.git";
const EXPECTED_UPSTREAM_REVISION: &str = "b385044250f1ed3c9379ab34a8fe82f02fdffaa4";
const EXPECTED_LICENSE_SHA256: &str =
    "c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4";
const EXPECTED_VENDOR_MANIFEST_SHA256: &str =
    "dc1b04a7ea2639d0e78f6810c433f12940304c921d2b7226178a9deed871f0cc";
const MAX_PORTABLE_COMPONENT_BYTES: usize = 255;
const MAX_PORTABLE_PATH_BYTES: usize = 4_096;
const MAX_THEME_BYTES: u64 = 1024 * 1024;
const MAX_LICENSE_BYTES: u64 = 256 * 1024;
const MAX_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
const CATEGORIES: &[(&str, usize)] = &[
    ("base16", 178),
    ("standard", 134),
    ("special_edition", 8),
    ("stradicat", 1),
    ("warp_bundled", 19),
];
const ROOT_FILES: &[&str] = &[
    "LICENSE",
    "README.md",
    "UPSTREAM_REVISION",
    "VENDOR_MANIFEST.json",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VendorManifest {
    schema_version: u32,
    source: String,
    revision: String,
    theme_count: usize,
    category_counts: BTreeMap<String, usize>,
    license: ManifestFile,
    files: Vec<ManifestFile>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestFile {
    path: String,
    bytes: u64,
    sha256: String,
}

#[derive(Debug)]
struct ThemeAsset {
    category: String,
    stem: String,
    filename: String,
    path: PathBuf,
}

fn main() {
    if let Err(error) = validate_and_generate() {
        panic!("vendored Warp theme validation failed: {error}");
    }
}

fn validate_and_generate() -> Result<(), String> {
    let manifest_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR")
            .ok_or_else(|| "CARGO_MANIFEST_DIR is not set".to_owned())?,
    );
    // Keep the corpus below the crate root so `cargo package` includes every
    // source consumed by this build script. Generated includes remain
    // manifest-relative and therefore do not capture the build checkout's
    // absolute path.
    let catalog_root = manifest_dir.join("assets/warp-themes");
    println!("cargo:rerun-if-changed={}", catalog_root.display());

    let inventory = inventory_catalog(&catalog_root)?;
    let revision_path = catalog_root.join("UPSTREAM_REVISION");
    let revision_bytes = read_regular_file(&revision_path, 256, "revision marker")?;
    let expected_revision_marker = format!("{EXPECTED_UPSTREAM_REVISION}\n");
    if revision_bytes != expected_revision_marker.as_bytes() {
        return Err(format!(
            "{} must contain exactly audited revision {} followed by one newline",
            revision_path.display(),
            EXPECTED_UPSTREAM_REVISION
        ));
    }

    let manifest_path = catalog_root.join("VENDOR_MANIFEST.json");
    let manifest_bytes =
        read_regular_file(&manifest_path, MAX_MANIFEST_BYTES, "VENDOR_MANIFEST.json")?;
    validate_vendor_manifest_hash(&manifest_bytes)?;
    let manifest: VendorManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("failed to parse {}: {error}", manifest_path.display()))?;
    validate_manifest_header(&manifest)?;

    let expected_category_counts: BTreeMap<String, usize> = CATEGORIES
        .iter()
        .map(|(category, count)| ((*category).to_owned(), *count))
        .collect();
    if manifest.category_counts != expected_category_counts {
        return Err(format!(
            "manifest category_counts {:?} do not match audited counts {:?}",
            manifest.category_counts, expected_category_counts
        ));
    }
    if manifest.theme_count != EXPECTED_THEME_COUNT {
        return Err(format!(
            "manifest theme_count is {}; audited count is {EXPECTED_THEME_COUNT}",
            manifest.theme_count
        ));
    }
    if manifest.files.len() != EXPECTED_THEME_COUNT {
        return Err(format!(
            "manifest has {} file records; audited count is {EXPECTED_THEME_COUNT}",
            manifest.files.len()
        ));
    }

    validate_license(&catalog_root, &manifest.license)?;
    validate_theme_manifest(&inventory, &manifest.files, &expected_category_counts)?;
    generate_catalog(inventory)
}

fn validate_vendor_manifest_hash(bytes: &[u8]) -> Result<(), String> {
    let actual = sha256_hex(bytes);
    if actual != EXPECTED_VENDOR_MANIFEST_SHA256 {
        return Err(format!(
            "VENDOR_MANIFEST.json SHA-256 is {actual}; audited canonical SHA-256 is {EXPECTED_VENDOR_MANIFEST_SHA256}"
        ));
    }
    Ok(())
}

fn validate_manifest_header(manifest: &VendorManifest) -> Result<(), String> {
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(format!(
            "manifest schema_version is {}; expected {MANIFEST_SCHEMA_VERSION}",
            manifest.schema_version
        ));
    }
    if manifest.source != EXPECTED_UPSTREAM_SOURCE {
        return Err(format!(
            "manifest source is {:?}; expected {:?}",
            manifest.source, EXPECTED_UPSTREAM_SOURCE
        ));
    }
    if !is_full_lower_hex(&manifest.revision, 40) {
        return Err(
            "manifest revision must be an exact 40-character lowercase hexadecimal commit ID"
                .to_owned(),
        );
    }
    if manifest.revision != EXPECTED_UPSTREAM_REVISION {
        return Err(format!(
            "manifest revision {} does not match audited revision {}",
            manifest.revision, EXPECTED_UPSTREAM_REVISION
        ));
    }
    Ok(())
}

fn validate_license(catalog_root: &Path, record: &ManifestFile) -> Result<(), String> {
    validate_manifest_record(record, "manifest license")?;
    if record.path != "LICENSE" {
        return Err(format!(
            "manifest license path is {:?}; expected \"LICENSE\"",
            record.path
        ));
    }
    if record.bytes == 0 || record.bytes > MAX_LICENSE_BYTES {
        return Err(format!(
            "manifest LICENSE size {} is outside 1..={MAX_LICENSE_BYTES}",
            record.bytes
        ));
    }
    if record.sha256 != EXPECTED_LICENSE_SHA256 {
        return Err(format!(
            "manifest LICENSE SHA-256 {} does not match audited license SHA-256 {}",
            record.sha256, EXPECTED_LICENSE_SHA256
        ));
    }

    let license_path = catalog_root.join("LICENSE");
    let bytes = read_regular_file(&license_path, MAX_LICENSE_BYTES, "Warp themes LICENSE")?;
    if bytes.len() as u64 != record.bytes {
        return Err(format!(
            "{} has {} bytes; manifest records {}",
            license_path.display(),
            bytes.len(),
            record.bytes
        ));
    }
    let actual_hash = sha256_hex(&bytes);
    if actual_hash != record.sha256 {
        return Err(format!(
            "{} SHA-256 is {}; manifest records {}",
            license_path.display(),
            actual_hash,
            record.sha256
        ));
    }
    Ok(())
}

fn validate_theme_manifest(
    inventory: &BTreeMap<String, ThemeAsset>,
    records: &[ManifestFile],
    expected_category_counts: &BTreeMap<String, usize>,
) -> Result<(), String> {
    let mut manifest_paths = BTreeSet::new();
    let mut manifest_category_counts: BTreeMap<String, usize> = CATEGORIES
        .iter()
        .map(|(category, _)| ((*category).to_owned(), 0))
        .collect();
    let mut previous_path: Option<&str> = None;

    for (index, record) in records.iter().enumerate() {
        validate_manifest_record(record, &format!("manifest files[{index}]"))?;
        if record.bytes == 0 || record.bytes > MAX_THEME_BYTES {
            return Err(format!(
                "manifest size for {:?} is outside 1..={MAX_THEME_BYTES}: {}",
                record.path, record.bytes
            ));
        }
        if previous_path.is_some_and(|previous| previous >= record.path.as_str()) {
            return Err(format!(
                "manifest file records must be strictly sorted; {:?} follows {:?}",
                record.path, previous_path
            ));
        }
        previous_path = Some(&record.path);

        let (category, _filename, _stem) = validate_theme_path(&record.path, "manifest")?;
        let count = manifest_category_counts
            .get_mut(category)
            .ok_or_else(|| format!("manifest uses non-allowlisted category {category:?}"))?;
        *count += 1;
        if !manifest_paths.insert(record.path.clone()) {
            return Err(format!(
                "manifest contains duplicate path {:?}",
                record.path
            ));
        }

        let asset = inventory.get(&record.path).ok_or_else(|| {
            format!(
                "manifest records missing filesystem theme {:?}",
                record.path
            )
        })?;
        let bytes = read_regular_file(&asset.path, MAX_THEME_BYTES, "Warp theme YAML")?;
        std::str::from_utf8(&bytes).map_err(|error| {
            format!(
                "{} is not valid UTF-8 required by include_str!: {error}",
                asset.path.display()
            )
        })?;
        if bytes.len() as u64 != record.bytes {
            return Err(format!(
                "{} has {} bytes; manifest records {}",
                asset.path.display(),
                bytes.len(),
                record.bytes
            ));
        }
        let actual_hash = sha256_hex(&bytes);
        if actual_hash != record.sha256 {
            return Err(format!(
                "{} SHA-256 is {}; manifest records {}",
                asset.path.display(),
                actual_hash,
                record.sha256
            ));
        }
    }

    if manifest_category_counts != *expected_category_counts {
        return Err(format!(
            "manifest file paths yield category counts {:?}; audited counts are {:?}",
            manifest_category_counts, expected_category_counts
        ));
    }

    let inventory_paths: BTreeSet<_> = inventory.keys().cloned().collect();
    if manifest_paths != inventory_paths {
        let missing: Vec<_> = manifest_paths
            .difference(&inventory_paths)
            .cloned()
            .collect();
        let extra: Vec<_> = inventory_paths
            .difference(&manifest_paths)
            .cloned()
            .collect();
        return Err(format!(
            "manifest/filesystem path sets differ; missing on disk: {missing:?}; unrecorded on disk: {extra:?}"
        ));
    }
    Ok(())
}

fn is_bidi_control(character: char) -> bool {
    matches!(
        character as u32,
        0x061c | 0x200e | 0x200f | 0x202a..=0x202e | 0x2066..=0x2069
    )
}

fn validate_portable_component(component: &str, label: &str) -> Result<(), String> {
    if component.is_empty() || component == "." || component == ".." {
        return Err(format!(
            "{label} contains an empty or traversal component: {component:?}"
        ));
    }
    if component.chars().any(is_bidi_control) {
        return Err(format!(
            "{label} component contains a bidi control: {component:?}"
        ));
    }
    if component.chars().any(char::is_control) {
        return Err(format!(
            "{label} component contains a control character: {component:?}"
        ));
    }
    if component
        .chars()
        .any(|character| matches!(character, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
    {
        return Err(format!(
            "{label} component contains a Windows-reserved character: {component:?}"
        ));
    }
    if component.ends_with('.') || component.ends_with(' ') {
        return Err(format!(
            "{label} component has a Windows-unsafe trailing dot or space: {component:?}"
        ));
    }
    let windows_stem = component
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    if matches!(
        windows_stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    ) {
        return Err(format!(
            "{label} component uses a Windows-reserved device name: {component:?}"
        ));
    }
    // Requiring this portable ASCII subset also guarantees NFC without adding
    // normalization behavior that could silently rename an audited asset.
    if !component
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(format!(
            "{label} component must use only portable ASCII letters, digits, '.', '_', or '-': {component:?}"
        ));
    }
    if component.len() > MAX_PORTABLE_COMPONENT_BYTES {
        return Err(format!(
            "{label} component exceeds {MAX_PORTABLE_COMPONENT_BYTES} bytes: {component:?}"
        ));
    }
    Ok(())
}

fn validate_portable_relative_path(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > MAX_PORTABLE_PATH_BYTES {
        return Err(format!(
            "{label} path must contain 1..={MAX_PORTABLE_PATH_BYTES} bytes: {value:?}"
        ));
    }
    let bytes = value.as_bytes();
    let has_windows_prefix = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    if value.starts_with('/') || value.contains('\\') || has_windows_prefix {
        return Err(format!(
            "{label} contains an absolute or backslash path: {value:?}"
        ));
    }
    for component in value.split('/') {
        validate_portable_component(component, label)?;
    }
    Ok(())
}

fn validate_manifest_record(record: &ManifestFile, label: &str) -> Result<(), String> {
    if !is_full_lower_hex(&record.sha256, 64) {
        return Err(format!(
            "{label} SHA-256 for {:?} must be 64 lowercase hexadecimal characters",
            record.path
        ));
    }
    validate_portable_relative_path(&record.path, label)
}

fn inventory_catalog(catalog_root: &Path) -> Result<BTreeMap<String, ThemeAsset>, String> {
    let root_metadata = fs::symlink_metadata(catalog_root)
        .map_err(|error| format!("failed to inspect {}: {error}", catalog_root.display()))?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(format!(
            "catalog root must be a real directory, not a symlink: {}",
            catalog_root.display()
        ));
    }

    let allowed_categories: BTreeSet<_> = CATEGORIES.iter().map(|(name, _)| *name).collect();
    let allowed_root_files: BTreeSet<_> = ROOT_FILES.iter().copied().collect();
    let mut observed_categories = BTreeSet::new();
    let mut observed_root_files = BTreeSet::new();

    let root_entries = fs::read_dir(catalog_root)
        .map_err(|error| format!("failed to read {}: {error}", catalog_root.display()))?;
    for entry in root_entries {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read a directory entry below {}: {error}",
                catalog_root.display()
            )
        })?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "catalog root contains a non-UTF-8 filename".to_owned())?;
        validate_portable_component(&name, "catalog root")?;
        let metadata = fs::symlink_metadata(entry.path())
            .map_err(|error| format!("failed to inspect {}: {error}", entry.path().display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("catalog root rejects symlink {name:?}"));
        }
        if metadata.is_dir() {
            if !allowed_categories.contains(name.as_str()) {
                return Err(format!(
                    "catalog root contains non-allowlisted directory {name:?}"
                ));
            }
            observed_categories.insert(name);
        } else if metadata.is_file() {
            if !allowed_root_files.contains(name.as_str()) {
                return Err(format!(
                    "catalog root contains non-allowlisted file {name:?}"
                ));
            }
            observed_root_files.insert(name);
        } else {
            return Err(format!("catalog root contains special entry {name:?}"));
        }
    }

    let expected_categories: BTreeSet<_> = allowed_categories
        .iter()
        .map(|category| (*category).to_owned())
        .collect();
    if observed_categories != expected_categories {
        return Err(format!(
            "catalog categories {:?} do not match required categories {:?}",
            observed_categories, expected_categories
        ));
    }
    let expected_root_files: BTreeSet<_> = allowed_root_files
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    if observed_root_files != expected_root_files {
        return Err(format!(
            "catalog root files {:?} do not match required files {:?}",
            observed_root_files, expected_root_files
        ));
    }

    let mut inventory = BTreeMap::new();
    let mut case_insensitive_paths: BTreeMap<String, String> = BTreeMap::new();
    let mut case_insensitive_ids: BTreeMap<String, String> = BTreeMap::new();
    for (category, expected_count) in CATEGORIES {
        let dir = catalog_root.join(category);
        let entries = fs::read_dir(&dir)
            .map_err(|error| format!("failed to read {}: {error}", dir.display()))?;
        let category_start = inventory.len();
        for entry in entries {
            let entry = entry.map_err(|error| {
                format!(
                    "failed to read a directory entry below {}: {error}",
                    dir.display()
                )
            })?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(format!(
                    "category directories may contain only regular YAML/YML files: {}",
                    path.display()
                ));
            }
            if metadata.len() == 0 || metadata.len() > MAX_THEME_BYTES {
                return Err(format!(
                    "{} size {} is outside 1..={MAX_THEME_BYTES}",
                    path.display(),
                    metadata.len()
                ));
            }
            let filename = entry
                .file_name()
                .into_string()
                .map_err(|_| format!("{} contains a non-UTF-8 filename", dir.display()))?;
            let relative = format!("{category}/{filename}");
            let (_validated_category, _validated_filename, stem) =
                validate_theme_path(&relative, "filesystem")?;

            let path_key = relative.to_ascii_lowercase();
            if let Some(previous) = case_insensitive_paths.insert(path_key, relative.clone()) {
                return Err(format!(
                    "catalog has a case-insensitive path collision: {previous:?} and {relative:?}"
                ));
            }
            let id_key = format!("{category}/{stem}").to_ascii_lowercase();
            if let Some(previous) = case_insensitive_ids.insert(id_key, relative.clone()) {
                return Err(format!(
                    "catalog has a case-insensitive theme ID collision: {previous:?} and {relative:?}"
                ));
            }
            if inventory
                .insert(
                    relative.clone(),
                    ThemeAsset {
                        category: (*category).to_owned(),
                        stem: stem.to_owned(),
                        filename,
                        path,
                    },
                )
                .is_some()
            {
                return Err(format!("catalog contains duplicate path {relative:?}"));
            }
        }
        let actual_count = inventory.len() - category_start;
        if actual_count != *expected_count {
            return Err(format!(
                "category {category} contains {actual_count} themes; audited count is {expected_count}"
            ));
        }
    }
    if inventory.len() != EXPECTED_THEME_COUNT {
        return Err(format!(
            "catalog contains {} themes; audited count is {EXPECTED_THEME_COUNT}",
            inventory.len()
        ));
    }
    Ok(inventory)
}

fn validate_theme_path<'a>(
    relative: &'a str,
    label: &str,
) -> Result<(&'a str, &'a str, &'a str), String> {
    validate_portable_relative_path(relative, label)?;
    let mut parts = relative.split('/');
    let category = parts.next().unwrap_or_default();
    let filename = parts.next().unwrap_or_default();
    if category.is_empty() || filename.is_empty() || parts.next().is_some() {
        return Err(format!(
            "{label} theme path must have category/filename form: {relative:?}"
        ));
    }
    if !CATEGORIES.iter().any(|(allowed, _)| *allowed == category) {
        return Err(format!(
            "{label} theme path uses non-allowlisted category: {relative:?}"
        ));
    }
    let path = Path::new(filename);
    let extension = path.extension().and_then(OsStr::to_str);
    if !matches!(extension, Some("yaml" | "yml")) {
        return Err(format!(
            "{label} theme path must have a lowercase .yaml or .yml extension: {relative:?}"
        ));
    }
    let stem = path
        .file_stem()
        .and_then(OsStr::to_str)
        .filter(|stem| !stem.is_empty())
        .ok_or_else(|| format!("{label} theme filename has no UTF-8 stem: {relative:?}"))?;
    Ok((category, filename, stem))
}

fn read_regular_file(path: &Path, limit: u64, label: &str) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("failed to inspect {label} {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "{label} must be a regular file, not a symlink or special file: {}",
            path.display()
        ));
    }
    if metadata.len() > limit {
        return Err(format!(
            "{label} {} is {} bytes; limit is {limit}",
            path.display(),
            metadata.len()
        ));
    }
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read {label} {}: {error}", path.display()))?;
    if bytes.len() as u64 != metadata.len() {
        return Err(format!(
            "{label} {} changed while it was being read",
            path.display()
        ));
    }
    Ok(bytes)
}

fn generate_catalog(inventory: BTreeMap<String, ThemeAsset>) -> Result<(), String> {
    let mut assets: Vec<_> = inventory.into_values().collect();
    assets.sort_by(|left, right| (&left.category, &left.stem).cmp(&(&right.category, &right.stem)));

    let mut generated =
        String::from("pub static EMBEDDED_WARP_THEMES: &[EmbeddedWarpThemeSource] = &[\n");
    for asset in assets {
        let id = format!("{}/{}", asset.category, asset.stem);
        let relative = format!("assets/warp-themes/{}/{}", asset.category, asset.filename);
        let include_suffix = format!("/{relative}");
        println!("cargo:rerun-if-changed={}", asset.path.display());
        generated.push_str("    EmbeddedWarpThemeSource {\n");
        writeln!(generated, "        id: {id:?},").expect("write to String");
        writeln!(generated, "        category: {:?},", asset.category).expect("write to String");
        writeln!(generated, "        stem: {:?},", asset.stem).expect("write to String");
        writeln!(
            generated,
            "        yaml: include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), {include_suffix:?})),"
        )
        .expect("write to String");
        generated.push_str("    },\n");
    }
    generated.push_str("];\n");

    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or_else(|| "OUT_DIR is not set".to_owned())?)
        .join("warp_theme_catalog.rs");
    fs::write(&out, generated)
        .map_err(|error| format!("failed to write {}: {error}", out.display()))?;

    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        write!(output, "{byte:02x}").expect("write to String");
    }
    output
}

fn is_full_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audited_manifest_hash_accepts_only_canonical_bytes() {
        let canonical = include_bytes!("assets/warp-themes/VENDOR_MANIFEST.json");
        validate_vendor_manifest_hash(canonical).expect("canonical manifest hash");

        let mut tampered = canonical.to_vec();
        tampered.push(b'\n');
        assert!(validate_vendor_manifest_hash(&tampered).is_err());
    }

    #[test]
    fn portable_paths_reject_hostile_cross_platform_names() {
        for accepted in [
            "LICENSE",
            "base16/base16_3024.yaml",
            "special_edition/first_light.yaml",
        ] {
            validate_portable_relative_path(accepted, "test").expect(accepted);
        }

        for rejected in [
            "base16/cafe\u{301}.yaml",
            "base16/caf\u{e9}.yaml",
            "base16/evil\u{202e}yaml.yaml",
            "base16/CON.yaml",
            "base16/nul.txt.yaml",
            "base16/has:colon.yaml",
            "base16/has space.yaml",
            "base16/trailing.",
            "base16/trailing ",
            "../base16/theme.yaml",
            "C:/base16/theme.yaml",
            "base16\\theme.yaml",
        ] {
            assert!(
                validate_portable_relative_path(rejected, "test").is_err(),
                "accepted hostile path {rejected:?}"
            );
        }
    }
}
