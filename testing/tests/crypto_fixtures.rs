use adze_testing::crypto_fixtures::{CorruptPem, rsa_fixture, scoped_fixture_label};

#[test]
fn public_rsa_fixture_api_is_deterministic() {
    let first = rsa_fixture("testing::crypto_fixtures", "api");
    let second = rsa_fixture("testing::crypto_fixtures", "api");

    assert_eq!(
        first.private_key_pkcs8_pem(),
        second.private_key_pkcs8_pem()
    );
}

#[test]
fn public_rsa_fixture_api_exposes_negative_testing_helpers() {
    let fixture = rsa_fixture("testing::crypto_fixtures", "negative");
    let corrupted = fixture.private_key_pkcs8_pem_corrupt(CorruptPem::BadHeader);

    assert_ne!(corrupted, fixture.private_key_pkcs8_pem());
}

#[test]
fn public_scoped_fixture_label_formats_stably() {
    assert_eq!(
        scoped_fixture_label("testing::crypto_fixtures", "jwt"),
        "testing::crypto_fixtures::jwt"
    );
}
