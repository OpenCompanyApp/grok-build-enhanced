//! Installed grok CLI version, lockstepped with shipping binaries.

use semver::Version;

pub const TEST_VERSION_ENV: &str = "GROK_TEST_VERSION";

/// User-facing fork identity. Protocol, executable, and storage identities stay
/// `grok`; this constant is only for distribution branding.
pub const ENHANCED_PRODUCT_NAME: &str = "Grok Build Enhanced";

/// Distribution subtitle used on branded startup and about surfaces.
pub const ENHANCED_SUBTITLE: &str = "The unofficial daily-driver fork of Grok Build.";

/// User-facing name for the source distribution this fork tracks.
pub const UPSTREAM_PRODUCT_NAME: &str = "Grok Build";

/// Minimum client compatibility version required by the Sol, Terra, and Luna
/// entries in the public OpenAI Codex model catalog audited at upstream commit
/// `f737605606c14e3aa59a4c17be80d338f164dff5`. Review that source again before
/// changing this backend-protocol value; it is not the Grok Build app version.
pub const OPENAI_CODEX_COMPATIBILITY_VERSION: &str = "0.144.0";

/// Installed Grok Build Enhanced release version. Distribution builds may
/// override this with the fork release tag through `GROK_VERSION`.
pub const VERSION: &str = match option_env!("GROK_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

/// Audited Grok Build package version this fork was based on. Unlike
/// [`VERSION`], this is never replaced by a downstream release tag.
pub const UPSTREAM_BASE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// [`TEST_VERSION_ENV`] override first, then [`VERSION`]. Trimmed so
/// non-semver-aware callers can pass the result straight into parsing.
pub fn installed() -> String {
    std::env::var(TEST_VERSION_ENV)
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|_| VERSION.to_string())
}

pub fn installed_semver() -> Result<Version, semver::Error> {
    Version::parse(&installed())
}

/// Format the compiled version with a channel label for user-facing display.
///
/// `channel_label` is a pre-formatted suffix such as `" [alpha]"`, `" [stable]"`,
/// or `""` (empty when no cached pointer is available). Obtain it from
/// `xai_grok_update::channel_label()`.
///
/// Example: `"0.2.5 [stable]"` or `"0.2.5 [alpha]"`.
pub fn display_version(channel_label: &str) -> String {
    format!("{}{}", VERSION, channel_label)
}

/// Format a version-with-commit string with a channel label.
///
/// Same semantics as [`display_version`] but for the full
/// `"0.2.5 (abc1234)"` string.
pub fn display_version_with_commit(version_with_commit: &str, channel_label: &str) -> String {
    format!("{}{}", version_with_commit, channel_label)
}

/// Return a compiled fork revision when one is available.
///
/// Build scripts use `"unknown"` when the checkout identity cannot be read.
/// Treating that sentinel (and blank values) as absent keeps user-facing
/// surfaces from presenting fabricated metadata.
pub fn fork_revision(revision: &str) -> Option<&str> {
    let revision = revision.trim();
    (!revision.is_empty() && !revision.eq_ignore_ascii_case("unknown")).then_some(revision)
}

/// Format the additive one-line identity used by `grok --version`.
///
/// The Enhanced name and Codex compatibility version are labels only and never
/// flow into protocol headers. The Enhanced release version, audited upstream
/// base version, and fork checkout revision are deliberately separate inputs:
/// neither the release tag nor checkout revision is upstream provenance.
pub fn enhanced_cli_version(
    enhanced_version: &str,
    upstream_base_version: &str,
    fork_revision: Option<&str>,
    channel_label: &str,
) -> String {
    let mut identity = format!(
        "{ENHANCED_PRODUCT_NAME} {enhanced_version} · upstream base {upstream_base_version}"
    );
    let channel_label = channel_label.trim();
    if !channel_label.is_empty() {
        identity.push_str(" · Enhanced updates ");
        identity.push_str(channel_label);
    }
    if let Some(revision) = fork_revision.and_then(self::fork_revision) {
        identity.push_str(" · fork ");
        identity.push_str(revision);
    }
    identity.push_str(" · Codex compat ");
    identity.push_str(OPENAI_CODEX_COMPATIBILITY_VERSION);
    identity
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Display formatting invariant matrix — verifies label appending
    /// works correctly across all label states (alpha, stable, empty).
    #[test]
    fn test_display_version_formatting_matrix() {
        let cases: &[(&str, &str, &str)] = &[
            // (version_with_commit,    label,        expected_suffix)
            ("0.2.5 (abc1234)", " [alpha]", "0.2.5 (abc1234) [alpha]"),
            ("0.2.5 (abc1234)", " [stable]", "0.2.5 (abc1234) [stable]"),
            ("0.2.5 (abc1234)", "", "0.2.5 (abc1234)"),
            (
                "0.1.220-alpha.2 (def0)",
                " [alpha]",
                "0.1.220-alpha.2 (def0) [alpha]",
            ),
        ];
        for (vwc, label, expected) in cases {
            assert_eq!(
                display_version_with_commit(vwc, label),
                *expected,
                "display_version_with_commit({:?}, {:?})",
                vwc,
                label,
            );
        }
        // display_version uses compiled VERSION — just verify the label appends
        assert_eq!(display_version(""), VERSION);
        assert!(display_version(" [stable]").ends_with("[stable]"));
    }

    #[test]
    fn distribution_copy_is_stable() {
        assert_eq!(ENHANCED_PRODUCT_NAME, "Grok Build Enhanced");
        assert_eq!(
            ENHANCED_SUBTITLE,
            "The unofficial daily-driver fork of Grok Build."
        );
        assert_eq!(UPSTREAM_PRODUCT_NAME, "Grok Build");
    }

    #[test]
    fn fork_revision_omits_unavailable_metadata() {
        assert_eq!(fork_revision("abc1234"), Some("abc1234"));
        assert_eq!(fork_revision(" abc1234 \n"), Some("abc1234"));
        assert_eq!(fork_revision(""), None);
        assert_eq!(fork_revision("unknown"), None);
        assert_eq!(fork_revision("UNKNOWN"), None);
    }

    #[test]
    fn enhanced_cli_version_labels_each_compatibility_layer() {
        let rendered = enhanced_cli_version("1.4.0", "0.2.5", Some("fork123"), " [stable]");
        assert_eq!(
            rendered,
            "Grok Build Enhanced 1.4.0 · upstream base 0.2.5 · Enhanced updates [stable] · fork fork123 · Codex compat 0.144.0"
        );

        let without_revision = enhanced_cli_version("1.4.0", "0.2.5", Some("unknown"), "");
        assert!(!without_revision.contains(" · fork "));
        assert!(!without_revision.contains("Enhanced updates"));
        assert!(without_revision.contains("Grok Build Enhanced 1.4.0 · upstream base 0.2.5"));
    }
}
