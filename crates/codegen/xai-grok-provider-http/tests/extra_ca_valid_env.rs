#[test]
fn valid_bundle_loads_one_root_and_builds_clients() {
    const CERTIFICATE: &str = "-----BEGIN CERTIFICATE-----\n\
MIIDFTCCAf2gAwIBAgIUT2czXTuxSAjDjEh92UMB1OVahZYwDQYJKoZIhvcNAQEL\n\
BQAwGjEYMBYGA1UEAwwPdGVzdC1leHRyYS1jYS0xMB4XDTI2MDcyOTE4MzUwNFoX\n\
DTM2MDcyNjE4MzUwNFowGjEYMBYGA1UEAwwPdGVzdC1leHRyYS1jYS0xMIIBIjAN\n\
BgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA1gNk2BQwUy+n5cCaTFtGpSzVQv//\n\
d7QD+3QWeE411wIGJzp3nrd7np55X8JHxeg/pRhspQvLQAF7bt55LSkL/+sSth3S\n\
QTbBqhftic9CXik3llAwbdQkAM9srz5zXWW9KVjZ57dxjjxrS15SCXu/UmvGZy98\n\
faJcS++TRkczsNFzwQEqeDYARVc/no0C0I++NhGLPaNMfFAevvnu6Kt3CYMI5ls4\n\
KCFgnlau4CjgRCMSfRDCRcwEwUAp+DyX9IU+tvDAQY1ncVoa/05tvaEvw7pQ+UgW\n\
0wRG0lk7PLlcWmUkLcFpO+sL5GRkC8RoWM4cFbIOiXoVxUFks/z2y0GCEQIDAQAB\n\
o1MwUTAdBgNVHQ4EFgQU+lyC70W5aR6BIf4VNtjfiWMNzzkwHwYDVR0jBBgwFoAU\n\
+lyC70W5aR6BIf4VNtjfiWMNzzkwDwYDVR0TAQH/BAUwAwEB/zANBgkqhkiG9w0B\n\
AQsFAAOCAQEA02972nA7LshRgubz6BwXbh1gA5pLzTd5KEae+94Hq6mP2zJ1T0gk\n\
x+me0NtSgG4BJLdBIylUzo2UmsfB/sz+ght6WX1uB38Vc2UQsp0sRPeeiMovSd6n\n\
I7xZyuZEF3noYJVBBlKQ8XsCUIBNIROlyKlNjNcWY8tGqPh9cepvtZYkBgRZr1vW\n\
hJAE3EOL2ZddrMPF64QeU9UhvCm0Ch+Ceqa1ZWE0MygccggX5s2yQwtXO2ovJdjH\n\
6vW0I02r8sE+NX0d1u8rIPJEKlp89UwCwniD7SxHTNw8bbsTCWz+AMod7vC7De3X\n\
4Daxme+vD8adOfCeOIu5vNrlXLNST2yaTw==\n\
-----END CERTIFICATE-----\n";
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("root.pem");
    std::fs::write(&path, CERTIFICATE).unwrap();
    // This integration test has its own process and resolves the cache once.
    unsafe { std::env::set_var(xai_grok_provider_http::ENV_GROK_EXTRA_CA_BUNDLE, &path) };

    assert_eq!(xai_grok_provider_http::extra_root_ders().len(), 1);
    xai_grok_provider_http::with_extra_root_certificates(reqwest::Client::builder())
        .build()
        .unwrap();
    xai_grok_provider_http::with_extra_root_certificates_blocking(
        reqwest::blocking::Client::builder(),
    )
    .build()
    .unwrap();
}
