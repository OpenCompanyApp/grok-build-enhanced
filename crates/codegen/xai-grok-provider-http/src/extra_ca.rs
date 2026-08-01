//! Opt-in extra TLS roots from `GROK_EXTRA_CA_BUNDLE`.
//!
//! The bundle is additive to normal web PKI roots, capped at 1 MiB, parsed
//! once, and validated as DER before any HTTP client receives it. Invalid,
//! empty, unreadable, and oversized bundles therefore degrade to no extra
//! roots without disabling ordinary certificate verification.

use std::io::Read;
use std::sync::OnceLock;

use rustls::RootCertStore;
use rustls::pki_types::CertificateDer;
use rustls::pki_types::pem::PemObject;

pub const MAX_EXTRA_CA_BUNDLE_BYTES: u64 = 1024 * 1024;
pub const ENV_GROK_EXTRA_CA_BUNDLE: &str = "GROK_EXTRA_CA_BUNDLE";

/// Process-wide validated DER roots. Environment state is sampled once so all
/// transports in a process use one stable trust projection.
pub fn extra_root_ders() -> &'static [Vec<u8>] {
    static DERS: OnceLock<Vec<Vec<u8>>> = OnceLock::new();
    DERS.get_or_init(load_extra_root_ders).as_slice()
}

/// Apply extra roots to a reqwest 0.12 async builder.
pub fn with_extra_root_certificates(mut builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
    for der in extra_root_ders() {
        match reqwest::Certificate::from_der(der) {
            Ok(certificate) => builder = builder.add_root_certificate(certificate),
            Err(error) => tracing::warn!(
                %error,
                "GROK_EXTRA_CA_BUNDLE validated certificate was rejected by reqwest"
            ),
        }
    }
    builder
}

/// Apply extra roots to a reqwest 0.12 blocking builder.
pub fn with_extra_root_certificates_blocking(
    mut builder: reqwest::blocking::ClientBuilder,
) -> reqwest::blocking::ClientBuilder {
    for der in extra_root_ders() {
        match reqwest::Certificate::from_der(der) {
            Ok(certificate) => builder = builder.add_root_certificate(certificate),
            Err(error) => tracing::warn!(
                %error,
                "GROK_EXTRA_CA_BUNDLE validated certificate was rejected by reqwest"
            ),
        }
    }
    builder
}

fn load_extra_root_ders() -> Vec<Vec<u8>> {
    let path = match std::env::var_os(ENV_GROK_EXTRA_CA_BUNDLE) {
        Some(path) if !path.is_empty() => std::path::PathBuf::from(path),
        _ => return Vec::new(),
    };
    let bytes = match read_bundle_capped(&path) {
        Ok(bytes) => bytes,
        Err(BundleReadError::Io(error)) => {
            tracing::warn!(
                path = %path.display(),
                %error,
                "GROK_EXTRA_CA_BUNDLE unreadable; continuing without extra roots"
            );
            return Vec::new();
        }
        Err(BundleReadError::TooLarge) => {
            tracing::warn!(
                path = %path.display(),
                max_bytes = MAX_EXTRA_CA_BUNDLE_BYTES,
                "GROK_EXTRA_CA_BUNDLE exceeds size cap; continuing without extra roots"
            );
            return Vec::new();
        }
    };

    let outcome = parse_and_validate_pem(&bytes);
    if outcome.no_pem_blocks {
        tracing::warn!(
            path = %path.display(),
            "GROK_EXTRA_CA_BUNDLE contains no PEM certificates; continuing without extra roots"
        );
    } else if outcome.accepted.is_empty() {
        tracing::warn!(
            path = %path.display(),
            rejected = outcome.rejected,
            "GROK_EXTRA_CA_BUNDLE produced no usable roots; continuing without extra roots"
        );
    } else {
        tracing::info!(
            path = %path.display(),
            accepted = outcome.accepted.len(),
            rejected = outcome.rejected,
            "GROK_EXTRA_CA_BUNDLE loaded extra roots"
        );
    }
    outcome.accepted
}

#[derive(Debug)]
enum BundleReadError {
    Io(std::io::Error),
    TooLarge,
}

fn read_bundle_capped(path: &std::path::Path) -> Result<Vec<u8>, BundleReadError> {
    let file = std::fs::File::open(path).map_err(BundleReadError::Io)?;
    let mut bytes = Vec::new();
    let count = file
        .take(MAX_EXTRA_CA_BUNDLE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(BundleReadError::Io)?;
    if count as u64 > MAX_EXTRA_CA_BUNDLE_BYTES {
        return Err(BundleReadError::TooLarge);
    }
    Ok(bytes)
}

#[derive(Debug, Default)]
struct ParseOutcome {
    accepted: Vec<Vec<u8>>,
    rejected: usize,
    no_pem_blocks: bool,
}

fn parse_and_validate_pem(pem: &[u8]) -> ParseOutcome {
    let mut accepted = Vec::new();
    let mut rejected = 0;
    let mut saw_block = false;
    let mut store = RootCertStore::empty();
    for item in CertificateDer::pem_slice_iter(pem) {
        saw_block = true;
        match item {
            Ok(der) => match store.add(der.clone()) {
                Ok(()) => accepted.push(der.as_ref().to_vec()),
                Err(_) => rejected += 1,
            },
            Err(_) => rejected += 1,
        }
    }
    ParseOutcome {
        accepted,
        rejected,
        no_pem_blocks: !saw_block,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INVALID_DER_PEM: &[u8] =
        b"-----BEGIN CERTIFICATE-----\nMAMBAf8=\n-----END CERTIFICATE-----\n";

    #[test]
    fn empty_and_non_pem_inputs_add_no_roots() {
        for input in [b"".as_slice(), b"not a certificate".as_slice()] {
            let outcome = parse_and_validate_pem(input);
            assert!(outcome.accepted.is_empty());
            assert_eq!(outcome.rejected, 0);
            assert!(outcome.no_pem_blocks);
        }
    }

    #[test]
    fn invalid_der_is_rejected_before_reqwest() {
        let outcome = parse_and_validate_pem(INVALID_DER_PEM);
        assert!(outcome.accepted.is_empty());
        assert!(outcome.rejected >= 1);
        assert!(!outcome.no_pem_blocks);
    }

    #[test]
    fn capped_reader_rejects_oversized_input_and_accepts_limit() {
        let directory = tempfile::tempdir().unwrap();
        let oversized = directory.path().join("oversized.pem");
        std::fs::write(
            &oversized,
            vec![b'A'; MAX_EXTRA_CA_BUNDLE_BYTES as usize + 1],
        )
        .unwrap();
        assert!(matches!(
            read_bundle_capped(&oversized),
            Err(BundleReadError::TooLarge)
        ));

        let at_limit = directory.path().join("at-limit.pem");
        std::fs::write(&at_limit, vec![b'B'; MAX_EXTRA_CA_BUNDLE_BYTES as usize]).unwrap();
        assert_eq!(
            read_bundle_capped(&at_limit).unwrap().len(),
            MAX_EXTRA_CA_BUNDLE_BYTES as usize
        );
    }
}
