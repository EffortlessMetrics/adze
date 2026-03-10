//! Deterministic cryptographic fixtures for tests.
//!
//! Backed by `uselesskey`, this module lets tests generate RSA key material at
//! runtime instead of checking PEM or DER blobs into the repository.
//!
//! Use `module_path!()` for the `scope` argument so labels stay stable and do
//! not collide across crates:
//!
//! ```
//! use adze_testing::crypto_fixtures::{CorruptPem, rsa_fixture};
//!
//! let keypair = rsa_fixture(module_path!(), "jwt-issuer");
//! let bad_pem = keypair.private_key_pkcs8_pem_corrupt(CorruptPem::BadHeader);
//!
//! assert_ne!(bad_pem, keypair.private_key_pkcs8_pem());
//! ```

use std::sync::OnceLock;

use uselesskey::{Factory, RsaFactoryExt, RsaKeyPair, RsaSpec, Seed};

pub use uselesskey::Error as CryptoFixtureError;
pub use uselesskey::TempArtifact;
pub use uselesskey::negative::CorruptPem;

/// Environment variable used to override the shared deterministic seed.
pub const ADZE_USELESSKEY_SEED_ENV: &str = "ADZE_USELESSKEY_SEED";

const DEFAULT_USELESSKEY_SEED: &str = "adze-testing-uselesskey-fixtures-v1";

static CRYPTO_FIXTURE_FACTORY: OnceLock<Factory> = OnceLock::new();

/// Return the shared deterministic `uselesskey` factory for this process.
///
/// The factory is cached so repeated test calls reuse `uselesskey`'s internal
/// artifact cache instead of regenerating the same RSA material.
pub fn crypto_fixture_factory() -> &'static Factory {
    CRYPTO_FIXTURE_FACTORY.get_or_init(|| Factory::deterministic(configured_master_seed()))
}

/// Build a stable label namespace for a test fixture.
pub fn scoped_fixture_label(scope: &str, label: &str) -> String {
    format!("{scope}::{label}")
}

/// Generate a deterministic RSA fixture using the standard RS256 profile.
///
/// Pass `module_path!()` for `scope` so different tests do not accidentally
/// share labels.
pub fn rsa_fixture(scope: &str, label: &str) -> RsaKeyPair {
    rsa_fixture_with_spec(scope, label, RsaSpec::rs256())
}

/// Generate a deterministic RSA fixture with an explicit RSA spec.
pub fn rsa_fixture_with_spec(scope: &str, label: &str, spec: RsaSpec) -> RsaKeyPair {
    crypto_fixture_factory().rsa(scoped_fixture_label(scope, label), spec)
}

fn configured_master_seed() -> Seed {
    match std::env::var(ADZE_USELESSKEY_SEED_ENV) {
        Ok(raw) => Seed::from_env_value(&raw).unwrap_or_else(|err| {
            panic!("{ADZE_USELESSKEY_SEED_ENV} must be a valid uselesskey seed: {err}")
        }),
        Err(_) => {
            Seed::from_env_value(DEFAULT_USELESSKEY_SEED).expect("default uselesskey seed is valid")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn rsa_fixture_is_stable_for_same_scope_and_label() {
        let first = rsa_fixture(module_path!(), "issuer");
        let second = rsa_fixture(module_path!(), "issuer");

        assert_eq!(
            first.private_key_pkcs8_pem(),
            second.private_key_pkcs8_pem()
        );
        assert_eq!(first.public_key_spki_pem(), second.public_key_spki_pem());
    }

    #[test]
    fn rsa_fixture_changes_when_label_changes() {
        let first = rsa_fixture(module_path!(), "issuer-a");
        let second = rsa_fixture(module_path!(), "issuer-b");

        assert_ne!(
            first.private_key_pkcs8_pem(),
            second.private_key_pkcs8_pem()
        );
    }

    #[test]
    fn rsa_fixture_supports_negative_pem_variants() {
        let fixture = rsa_fixture(module_path!(), "negative");
        let corrupted = fixture.private_key_pkcs8_pem_corrupt(CorruptPem::BadHeader);

        assert_ne!(corrupted, fixture.private_key_pkcs8_pem());
        assert!(!corrupted.contains("-----BEGIN PRIVATE KEY-----"));
    }

    #[test]
    fn rsa_fixture_writes_tempfile_artifacts() {
        let fixture = rsa_fixture(module_path!(), "tempfile");
        let pem_file = fixture
            .write_private_key_pkcs8_pem()
            .expect("private key tempfile should be created");
        let content = fs::read_to_string(pem_file.path()).expect("tempfile should be readable");

        assert_eq!(content, fixture.private_key_pkcs8_pem());
    }

    #[test]
    fn scoped_fixture_label_namespaces_labels() {
        assert_eq!(
            scoped_fixture_label("crate::module", "issuer"),
            "crate::module::issuer"
        );
    }
}
