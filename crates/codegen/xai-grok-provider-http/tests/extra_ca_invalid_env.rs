#[test]
fn invalid_or_unreadable_bundle_adds_no_roots() {
    unsafe {
        std::env::set_var(
            xai_grok_provider_http::ENV_GROK_EXTRA_CA_BUNDLE,
            "/nonexistent/grok-enhanced-extra-ca.pem",
        )
    };
    assert!(xai_grok_provider_http::extra_root_ders().is_empty());
    xai_grok_provider_http::with_extra_root_certificates(reqwest::Client::builder())
        .build()
        .unwrap();
}
