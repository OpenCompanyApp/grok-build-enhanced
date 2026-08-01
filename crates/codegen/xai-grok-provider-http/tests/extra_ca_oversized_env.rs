#[test]
fn oversized_bundle_adds_no_roots() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("oversized.pem");
    std::fs::write(
        &path,
        vec![b'A'; xai_grok_provider_http::MAX_EXTRA_CA_BUNDLE_BYTES as usize + 1],
    )
    .unwrap();
    unsafe { std::env::set_var(xai_grok_provider_http::ENV_GROK_EXTRA_CA_BUNDLE, &path) };
    assert!(xai_grok_provider_http::extra_root_ders().is_empty());
}
